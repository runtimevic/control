use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuración completa de una máquina de estados compatible con XState
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachineConfig {
    /// ID único de la máquina
    pub id: String,
    /// Nombre del estado inicial
    pub initial: String,
    /// Mapa de estados
    pub states: HashMap<String, StateConfig>,
}

/// Configuración de un estado individual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateConfig {
    /// Tipo de estado (final, compound, etc.)
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub state_type: Option<StateType>,
    /// Acciones a ejecutar al entrar al estado
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry: Option<Vec<String>>,
    /// Acciones a ejecutar al salir del estado
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit: Option<Vec<String>>,
    /// Transiciones desde este estado
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on: Option<HashMap<String, TransitionConfig>>,
}

/// Tipo de estado
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StateType {
    /// Estado final
    Final,
    /// Estado compuesto (puede tener sub-estados)
    Compound,
}

/// Configuración de una transición
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransitionConfig {
    /// Transición simple: solo nombre del estado destino
    Simple(String),
    /// Transición completa con guards y actions
    Full {
        /// Estado destino
        target: String,
        /// Condición que debe cumplirse (guard)
        #[serde(skip_serializing_if = "Option::is_none")]
        guard: Option<String>,
        /// Acciones a ejecutar durante la transición
        #[serde(skip_serializing_if = "Option::is_none")]
        actions: Option<Vec<String>>,
    },
}

impl TransitionConfig {
    /// Obtiene el estado destino de la transición
    pub fn target(&self) -> &str {
        match self {
            TransitionConfig::Simple(target) => target,
            TransitionConfig::Full { target, .. } => target,
        }
    }

    /// Obtiene el guard si existe
    pub fn guard(&self) -> Option<&str> {
        match self {
            TransitionConfig::Simple(_) => None,
            TransitionConfig::Full { guard, .. } => guard.as_deref(),
        }
    }

    /// Obtiene las acciones de la transición
    pub fn actions(&self) -> Option<&[String]> {
        match self {
            TransitionConfig::Simple(_) => None,
            TransitionConfig::Full { actions, .. } => actions.as_deref(),
        }
    }
}

/// Estado de ejecución actual de la máquina
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionState {
    /// Estado actual
    pub current_state: String,
    /// Estado anterior (si existe)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_state: Option<String>,
    /// Eventos disponibles desde el estado actual
    pub available_events: Vec<String>,
    /// Timestamp de la última transición (en milisegundos)
    pub timestamp: u64,
}

/// Contexto de ejecución para guards y actions
#[derive(Debug, Clone, Default)]
pub struct MachineContext {
    /// Variables del contexto
    pub variables: HashMap<String, ContextValue>,
}

/// Valores que pueden almacenarse en el contexto
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContextValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

impl MachineContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_bool(&mut self, key: impl Into<String>, value: bool) {
        self.variables.insert(key.into(), ContextValue::Bool(value));
    }

    pub fn set_int(&mut self, key: impl Into<String>, value: i64) {
        self.variables.insert(key.into(), ContextValue::Int(value));
    }

    pub fn set_float(&mut self, key: impl Into<String>, value: f64) {
        self.variables.insert(key.into(), ContextValue::Float(value));
    }

    pub fn set_string(&mut self, key: impl Into<String>, value: String) {
        self.variables.insert(key.into(), ContextValue::String(value));
    }

    pub fn get(&self, key: &str) -> Option<&ContextValue> {
        self.variables.get(key)
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.get(key) {
            Some(ContextValue::Bool(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        match self.get(key) {
            Some(ContextValue::Int(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn get_float(&self, key: &str) -> Option<f64> {
        match self.get(key) {
            Some(ContextValue::Float(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        match self.get(key) {
            Some(ContextValue::String(v)) => Some(v.as_str()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_machine() {
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

        let config: StateMachineConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.id, "test");
        assert_eq!(config.initial, "idle");
        assert_eq!(config.states.len(), 2);
    }

    #[test]
    fn test_parse_full_transition() {
        let json = r#"
        {
            "id": "test",
            "initial": "idle",
            "states": {
                "idle": {
                    "entry": ["log", "init"],
                    "exit": ["cleanup"],
                    "on": {
                        "START": {
                            "target": "running",
                            "guard": "isReady",
                            "actions": ["startMotor", "logStart"]
                        }
                    }
                },
                "running": {
                    "type": "final"
                }
            }
        }
        "#;

        let config: StateMachineConfig = serde_json::from_str(json).unwrap();
        let idle_state = config.states.get("idle").unwrap();
        
        assert_eq!(idle_state.entry.as_ref().unwrap().len(), 2);
        assert_eq!(idle_state.exit.as_ref().unwrap().len(), 1);

        let transition = idle_state.on.as_ref().unwrap().get("START").unwrap();
        assert_eq!(transition.target(), "running");
        assert_eq!(transition.guard(), Some("isReady"));
        assert_eq!(transition.actions().unwrap().len(), 2);
    }
}
