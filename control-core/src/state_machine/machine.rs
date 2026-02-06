use crate::state_machine::actions::{ActionRegistry, GuardRegistry};
use crate::state_machine::types::{
    ExecutionState, MachineContext, StateConfig, StateMachineConfig, StateType,
};
use anyhow::{anyhow, Result};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

/// Máquina de estados ejecutable
pub struct StateMachine {
    /// Configuración de la máquina
    config: StateMachineConfig,
    /// Estado actual
    current_state: String,
    /// Estado anterior
    previous_state: Option<String>,
    /// Contexto de ejecución
    context: MachineContext,
    /// Registro de acciones
    actions: ActionRegistry,
    /// Registro de guards
    guards: GuardRegistry,
    /// Cola de eventos pendientes
    event_queue: VecDeque<String>,
    /// Timestamp de la última transición
    last_transition: u64,
}

impl StateMachine {
    /// Crea una nueva máquina de estados desde una configuración
    pub fn new(config: StateMachineConfig) -> Result<Self> {
        let initial_state = config.initial.clone();

        // Validar que el estado inicial existe
        if !config.states.contains_key(&initial_state) {
            return Err(anyhow!(
                "Initial state '{}' not found in states",
                initial_state
            ));
        }

        let mut machine = Self {
            config,
            current_state: initial_state,
            previous_state: None,
            context: MachineContext::new(),
            actions: ActionRegistry::new(),
            guards: GuardRegistry::new(),
            event_queue: VecDeque::new(),
            last_transition: Self::now_millis(),
        };

        // Ejecutar entry actions del estado inicial
        machine.execute_entry_actions(&machine.current_state.clone())?;

        Ok(machine)
    }

    /// Crea una máquina desde JSON
    pub fn from_json(json: &str) -> Result<Self> {
        let config: StateMachineConfig = serde_json::from_str(json)?;
        Self::new(config)
    }

    /// Obtiene el registro de acciones (mutable para registrar nuevas)
    pub fn actions_mut(&mut self) -> &mut ActionRegistry {
        &mut self.actions
    }

    /// Obtiene el registro de guards (mutable para registrar nuevos)
    pub fn guards_mut(&mut self) -> &mut GuardRegistry {
        &mut self.guards
    }

    /// Obtiene el contexto (mutable)
    pub fn context_mut(&mut self) -> &mut MachineContext {
        &mut self.context
    }

    /// Obtiene el contexto (inmutable)
    pub fn context(&self) -> &MachineContext {
        &self.context
    }

    /// Obtiene el estado actual
    pub fn current_state(&self) -> &str {
        &self.current_state
    }

    /// Obtiene el estado anterior
    pub fn previous_state(&self) -> Option<&str> {
        self.previous_state.as_deref()
    }

    /// Obtiene el estado de ejecución actual
    pub fn execution_state(&self) -> ExecutionState {
        ExecutionState {
            current_state: self.current_state.clone(),
            previous_state: self.previous_state.clone(),
            available_events: self.available_events(),
            timestamp: self.last_transition,
        }
    }

    /// Obtiene los eventos disponibles desde el estado actual
    pub fn available_events(&self) -> Vec<String> {
        let state = self.get_current_state_config();
        
        match &state.on {
            Some(transitions) => transitions.keys().cloned().collect(),
            None => Vec::new(),
        }
    }

    /// Verifica si la máquina está en un estado final
    pub fn is_final(&self) -> bool {
        let state = self.get_current_state_config();
        matches!(state.state_type, Some(StateType::Final))
    }

    /// Envía un evento a la máquina
    pub fn send(&mut self, event: impl Into<String>) -> Result<bool> {
        let event = event.into();
        
        // No procesar eventos si estamos en estado final
        if self.is_final() {
            tracing::warn!(
                "Cannot process event '{}' in final state '{}'",
                event,
                self.current_state
            );
            return Ok(false);
        }

        let state_config = self.get_current_state_config();

        // Buscar transición para este evento
        let transition = match &state_config.on {
            Some(transitions) => transitions.get(&event),
            None => {
                tracing::debug!(
                    "No transitions defined for state '{}'",
                    self.current_state
                );
                return Ok(false);
            }
        };

        let transition = match transition {
            Some(t) => t,
            None => {
                tracing::debug!(
                    "No transition for event '{}' in state '{}'",
                    event,
                    self.current_state
                );
                return Ok(false);
            }
        };

        // Evaluar guard si existe
        if let Some(guard_name) = transition.guard() {
            let can_transition = self.guards.evaluate(guard_name, &self.context)?;
            if !can_transition {
                tracing::debug!(
                    "Guard '{}' prevented transition from '{}'",
                    guard_name,
                    self.current_state
                );
                return Ok(false);
            }
        }

        // Clone transition data to avoid borrowing conflicts
        let target = transition.target().to_string();
        let actions = transition.actions().map(|a| a.to_vec());

        // Ejecutar transición
        self.transition(&target, actions.as_deref())?;

        Ok(true)
    }

    /// Cola un evento para procesamiento posterior
    pub fn queue_event(&mut self, event: impl Into<String>) {
        self.event_queue.push_back(event.into());
    }

    /// Procesa todos los eventos en cola
    pub fn process_queue(&mut self) -> Result<usize> {
        let mut processed = 0;

        while let Some(event) = self.event_queue.pop_front() {
            if self.send(event)? {
                processed += 1;
            }
        }

        Ok(processed)
    }

    /// Realiza una transición a un nuevo estado
    fn transition(&mut self, target: &str, transition_actions: Option<&[String]>) -> Result<()> {
        // Validar que el estado destino existe
        if !self.config.states.contains_key(target) {
            return Err(anyhow!("Target state '{}' not found", target));
        }

        tracing::info!(
            "Transitioning from '{}' to '{}'",
            self.current_state,
            target
        );

        // Ejecutar exit actions del estado actual
        self.execute_exit_actions(&self.current_state.clone())?;

        // Ejecutar acciones de la transición
        if let Some(actions) = transition_actions {
            for action_name in actions {
                self.actions.execute(action_name, &mut self.context)?;
            }
        }

        // Actualizar estados
        self.previous_state = Some(self.current_state.clone());
        self.current_state = target.to_string();
        self.last_transition = Self::now_millis();

        // Ejecutar entry actions del nuevo estado
        self.execute_entry_actions(target)?;

        Ok(())
    }

    /// Ejecuta las entry actions de un estado
    fn execute_entry_actions(&mut self, state_name: &str) -> Result<()> {
        let state = self.config.states.get(state_name).ok_or_else(|| {
            anyhow!("State '{}' not found", state_name)
        })?;

        if let Some(entry_actions) = &state.entry {
            for action_name in entry_actions {
                self.actions.execute(action_name, &mut self.context)?;
            }
        }

        Ok(())
    }

    /// Ejecuta las exit actions de un estado
    fn execute_exit_actions(&mut self, state_name: &str) -> Result<()> {
        let state = self.config.states.get(state_name).ok_or_else(|| {
            anyhow!("State '{}' not found", state_name)
        })?;

        if let Some(exit_actions) = &state.exit {
            for action_name in exit_actions {
                self.actions.execute(action_name, &mut self.context)?;
            }
        }

        Ok(())
    }

    /// Obtiene la configuración del estado actual
    fn get_current_state_config(&self) -> &StateConfig {
        self.config.states.get(&self.current_state).unwrap()
    }

    /// Obtiene el timestamp actual en milisegundos
    fn now_millis() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    /// Reinicia la máquina al estado inicial
    pub fn reset(&mut self) -> Result<()> {
        let initial = self.config.initial.clone();
        
        // No ejecutar exit actions al reiniciar
        self.previous_state = Some(self.current_state.clone());
        self.current_state = initial.clone();
        self.last_transition = Self::now_millis();
        self.context = MachineContext::new();
        self.event_queue.clear();

        // Ejecutar entry actions del estado inicial
        self.execute_entry_actions(&initial)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_machine::actions::{AlwaysTrueGuard, LogAction};
    use std::sync::Arc;

    #[test]
    fn test_simple_machine() {
        let json = r#"
        {
            "id": "test",
            "initial": "idle",
            "states": {
                "idle": {
                    "on": {
                        "START": "running"
                    }
                },
                "running": {
                    "on": {
                        "STOP": "idle"
                    }
                }
            }
        }
        "#;

        let mut machine = StateMachine::from_json(json).unwrap();
        assert_eq!(machine.current_state(), "idle");

        // Transition to running
        assert!(machine.send("START").unwrap());
        assert_eq!(machine.current_state(), "running");

        // Transition back to idle
        assert!(machine.send("STOP").unwrap());
        assert_eq!(machine.current_state(), "idle");
    }

    #[test]
    fn test_final_state() {
        let json = r#"
        {
            "id": "test",
            "initial": "running",
            "states": {
                "running": {
                    "on": {
                        "FINISH": "done"
                    }
                },
                "done": {
                    "type": "final"
                }
            }
        }
        "#;

        let mut machine = StateMachine::from_json(json).unwrap();
        assert!(!machine.is_final());

        machine.send("FINISH").unwrap();
        assert!(machine.is_final());

        // Cannot transition from final state
        assert!(!machine.send("ANYTHING").unwrap());
    }

    #[test]
    fn test_with_actions() {
        let json = r#"
        {
            "id": "test",
            "initial": "idle",
            "states": {
                "idle": {
                    "entry": ["log"],
                    "on": {
                        "START": "running"
                    }
                },
                "running": {
                    "exit": ["log"]
                }
            }
        }
        "#;

        let mut machine = StateMachine::from_json(json).unwrap();
        
        // Register log action
        machine.actions_mut().register(Arc::new(LogAction::new("test")));

        assert_eq!(machine.current_state(), "idle");
        machine.send("START").unwrap();
        assert_eq!(machine.current_state(), "running");
    }

    #[test]
    fn test_event_queue() {
        let json = r#"
        {
            "id": "test",
            "initial": "a",
            "states": {
                "a": { "on": { "NEXT": "b" } },
                "b": { "on": { "NEXT": "c" } },
                "c": { "on": { "NEXT": "a" } }
            }
        }
        "#;

        let mut machine = StateMachine::from_json(json).unwrap();
        
        // Queue multiple events
        machine.queue_event("NEXT");
        machine.queue_event("NEXT");
        machine.queue_event("NEXT");

        let processed = machine.process_queue().unwrap();
        assert_eq!(processed, 3);
        assert_eq!(machine.current_state(), "a");
    }

    #[test]
    fn test_reset() {
        let json = r#"
        {
            "id": "test",
            "initial": "a",
            "states": {
                "a": { "on": { "NEXT": "b" } },
                "b": { "on": { "NEXT": "c" } },
                "c": {}
            }
        }
        "#;

        let mut machine = StateMachine::from_json(json).unwrap();
        
        machine.send("NEXT").unwrap();
        machine.send("NEXT").unwrap();
        assert_eq!(machine.current_state(), "c");

        machine.reset().unwrap();
        assert_eq!(machine.current_state(), "a");
    }
}
