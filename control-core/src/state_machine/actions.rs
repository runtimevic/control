use crate::state_machine::types::{ContextValue, MachineContext};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Trait para implementar acciones personalizadas
pub trait Action: Send + Sync {
    /// Ejecuta la acción con el contexto dado
    fn execute(&self, context: &mut MachineContext) -> Result<()>;
    
    /// Nombre de la acción
    fn name(&self) -> &str;
}

/// Trait para implementar guards personalizados
pub trait Guard: Send + Sync {
    /// Evalúa el guard con el contexto dado
    fn evaluate(&self, context: &MachineContext) -> Result<bool>;
    
    /// Nombre del guard
    fn name(&self) -> &str;
}

/// Registro de acciones disponibles
#[derive(Clone)]
pub struct ActionRegistry {
    actions: HashMap<String, Arc<dyn Action>>,
}

impl ActionRegistry {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    /// Registra una nueva acción
    pub fn register(&mut self, action: Arc<dyn Action>) {
        self.actions.insert(action.name().to_string(), action);
    }

    /// Ejecuta una acción por nombre
    pub fn execute(&self, name: &str, context: &mut MachineContext) -> Result<()> {
        match self.actions.get(name) {
            Some(action) => action.execute(context),
            None => {
                tracing::warn!("Action '{}' not found, skipping", name);
                Ok(())
            }
        }
    }

    /// Verifica si una acción existe
    pub fn has_action(&self, name: &str) -> bool {
        self.actions.contains_key(name)
    }
}

impl Default for ActionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registro de guards disponibles
#[derive(Clone)]
pub struct GuardRegistry {
    guards: HashMap<String, Arc<dyn Guard>>,
}

impl GuardRegistry {
    pub fn new() -> Self {
        Self {
            guards: HashMap::new(),
        }
    }

    /// Registra un nuevo guard
    pub fn register(&mut self, guard: Arc<dyn Guard>) {
        self.guards.insert(guard.name().to_string(), guard);
    }

    /// Evalúa un guard por nombre
    pub fn evaluate(&self, name: &str, context: &MachineContext) -> Result<bool> {
        match self.guards.get(name) {
            Some(guard) => guard.evaluate(context),
            None => {
                tracing::warn!("Guard '{}' not found, defaulting to true", name);
                Ok(true)
            }
        }
    }

    /// Verifica si un guard existe
    pub fn has_guard(&self, name: &str) -> bool {
        self.guards.contains_key(name)
    }
}

impl Default for GuardRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Acciones predefinidas
// ============================================================================

/// Acción que registra un mensaje
pub struct LogAction {
    message: String,
}

impl LogAction {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Action for LogAction {
    fn execute(&self, _context: &mut MachineContext) -> Result<()> {
        tracing::info!("StateMachine: {}", self.message);
        Ok(())
    }

    fn name(&self) -> &str {
        "log"
    }
}

/// Acción que establece una variable en el contexto
pub struct SetVariableAction {
    name: String,
    key: String,
    value: ContextValue,
}

impl SetVariableAction {
    pub fn new(name: String, key: String, value: ContextValue) -> Self {
        Self { name, key, value }
    }
}

impl Action for SetVariableAction {
    fn execute(&self, context: &mut MachineContext) -> Result<()> {
        context.variables.insert(self.key.clone(), self.value.clone());
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// ============================================================================
// Guards predefinidos
// ============================================================================

/// Guard que verifica si una variable booleana es verdadera
pub struct BoolGuard {
    name: String,
    key: String,
}

impl BoolGuard {
    pub fn new(name: String, key: String) -> Self {
        Self { name, key }
    }
}

impl Guard for BoolGuard {
    fn evaluate(&self, context: &MachineContext) -> Result<bool> {
        Ok(context.get_bool(&self.key).unwrap_or(false))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Guard que siempre devuelve verdadero
pub struct AlwaysTrueGuard;

impl Guard for AlwaysTrueGuard {
    fn evaluate(&self, _context: &MachineContext) -> Result<bool> {
        Ok(true)
    }

    fn name(&self) -> &str {
        "alwaysTrue"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_registry() {
        let mut registry = ActionRegistry::new();
        let log_action = Arc::new(LogAction::new("test message"));
        registry.register(log_action);

        assert!(registry.has_action("log"));

        let mut context = MachineContext::new();
        registry.execute("log", &mut context).unwrap();
    }

    #[test]
    fn test_guard_registry() {
        let mut registry = GuardRegistry::new();
        let always_true = Arc::new(AlwaysTrueGuard);
        registry.register(always_true);

        assert!(registry.has_guard("alwaysTrue"));

        let context = MachineContext::new();
        let result = registry.evaluate("alwaysTrue", &context).unwrap();
        assert!(result);
    }

    #[test]
    fn test_bool_guard() {
        let guard = BoolGuard::new("testGuard".to_string(), "ready".to_string());
        let mut context = MachineContext::new();

        // Initially false
        assert!(!guard.evaluate(&context).unwrap());

        // Set to true
        context.set_bool("ready", true);
        assert!(guard.evaluate(&context).unwrap());
    }
}
