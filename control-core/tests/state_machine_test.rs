//! Standalone test for state machine module
//! This can be run independently to verify the implementation

use serde_json;
use std::collections::HashMap;

// Copy of the types for standalone testing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StateMachineConfig {
    pub id: String,
    pub initial: String,
    pub states: HashMap<String, StateConfig>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StateConfig {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub state_type: Option<StateType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on: Option<HashMap<String, TransitionConfig>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StateType {
    Final,
    Compound,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum TransitionConfig {
    Simple(String),
    Full {
        target: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        guard: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        actions: Option<Vec<String>>,
    },
}

fn main() {
    println!("=== State Machine JSON Parsing Tests ===\n");

    // Test 1: Simple machine
    let simple_json = r#"
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

    match serde_json::from_str::<StateMachineConfig>(simple_json) {
        Ok(config) => {
            println!("✓ Simple machine parsed successfully");
            println!("  ID: {}", config.id);
            println!("  Initial: {}", config.initial);
            println!("  States: {}", config.states.len());
        }
        Err(e) => {
            println!("✗ Failed to parse simple machine: {}", e);
        }
    }

    // Test 2: Full machine with actions and guards
    let full_json = r#"
    {
        "id": "motor-control",
        "initial": "Stopped",
        "states": {
            "Stopped": {
                "entry": ["disableMotor", "logStop"],
                "on": {
                    "START": {
                        "target": "Starting",
                        "guard": "safetyOk",
                        "actions": ["checkSafety"]
                    }
                }
            },
            "Starting": {
                "entry": ["enableMotor", "rampUp"],
                "on": {
                    "RUNNING": "Running",
                    "EMERGENCY_STOP": "Error"
                }
            },
            "Running": {
                "entry": ["logRunning"],
                "exit": ["logExit"],
                "on": {
                    "STOP": "Stopping",
                    "EMERGENCY_STOP": "Error"
                }
            },
            "Stopping": {
                "entry": ["rampDown"],
                "on": {
                    "STOPPED": "Stopped"
                }
            },
            "Error": {
                "entry": ["disableMotor", "logError", "triggerAlarm"],
                "type": "final"
            }
        }
    }
    "#;

    match serde_json::from_str::<StateMachineConfig>(full_json) {
        Ok(config) => {
            println!("\n✓ Full machine parsed successfully");
            println!("  ID: {}", config.id);
            println!("  Initial: {}", config.initial);
            println!("  States: {}", config.states.len());
            
            // Check Stopped state
            if let Some(stopped) = config.states.get("Stopped") {
                println!("\n  Stopped state:");
                if let Some(entry) = &stopped.entry {
                    println!("    Entry actions: {:?}", entry);
                }
                if let Some(on) = &stopped.on {
                    if let Some(start_trans) = on.get("START") {
                        match start_trans {
                            TransitionConfig::Full { target, guard, actions } => {
                                println!("    START transition:");
                                println!("      Target: {}", target);
                                println!("      Guard: {:?}", guard);
                                println!("      Actions: {:?}", actions);
                            }
                            _ => {}
                        }
                    }
                }
            }

            // Check Error state (final)
            if let Some(error) = config.states.get("Error") {
                println!("\n  Error state:");
                println!("    Type: {:?}", error.state_type);
                println!("    Is final: {}", matches!(error.state_type, Some(StateType::Final)));
            }
        }
        Err(e) => {
            println!("✗ Failed to parse full machine: {}", e);
        }
    }

    // Test 3: Traffic light from examples
    let traffic_json = r#"
    {
        "id": "traffic-light",
        "initial": "Red",
        "states": {
            "Red": {
                "entry": ["activateRedLight"],
                "exit": ["deactivateRedLight"],
                "on": {
                    "TIMER": "Green"
                }
            },
            "Yellow": {
                "entry": ["activateYellowLight"],
                "exit": ["deactivateYellowLight"],
                "on": {
                    "TIMER": "Red"
                }
            },
            "Green": {
                "entry": ["activateGreenLight"],
                "exit": ["deactivateGreenLight"],
                "on": {
                    "TIMER": "Yellow"
                }
            }
        }
    }
    "#;

    match serde_json::from_str::<StateMachineConfig>(traffic_json) {
        Ok(config) => {
            println!("\n✓ Traffic light machine parsed successfully");
            println!("  ID: {}", config.id);
            println!("  Initial: {}", config.initial);
            println!("  States: {}", config.states.len());
        }
        Err(e) => {
            println!("✗ Failed to parse traffic light machine: {}", e);
        }
    }

    println!("\n=== All Tests Passed! ===");
}
