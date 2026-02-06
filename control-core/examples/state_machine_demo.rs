//! Example demonstrating state machine usage
//!
//! This example shows how to create and use a state machine
//! for motor control with safety checks.

use control_core::state_machine::{
    Action, ActionRegistry, BoolGuard, Guard, GuardRegistry, LogAction, MachineContext,
    StateMachine,
};
use anyhow::Result;
use std::sync::Arc;

// Custom action to simulate motor control
struct MotorAction {
    name: String,
    enabled: bool,
}

impl MotorAction {
    fn new(name: impl Into<String>, enabled: bool) -> Self {
        Self {
            name: name.into(),
            enabled,
        }
    }
}

impl Action for MotorAction {
    fn execute(&self, context: &mut MachineContext) -> Result<()> {
        println!("Motor action: {} (enabled: {})", self.name, self.enabled);
        context.set_bool("motor_enabled", self.enabled);
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// Custom guard to check safety conditions
struct SafetyGuard;

impl Guard for SafetyGuard {
    fn evaluate(&self, context: &MachineContext) -> Result<bool> {
        // Check multiple safety conditions
        let emergency_stop = context.get_bool("emergency_stop").unwrap_or(false);
        let door_open = context.get_bool("door_open").unwrap_or(false);
        
        let safe = !emergency_stop && !door_open;
        println!("Safety check: {}", if safe { "OK" } else { "FAILED" });
        
        Ok(safe)
    }

    fn name(&self) -> &str {
        "safetyOk"
    }
}

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Motor control state machine JSON
    let machine_json = r#"
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
                "on": {
                    "RESET": {
                        "target": "Stopped",
                        "guard": "errorCleared",
                        "actions": ["clearError"]
                    }
                }
            }
        }
    }
    "#;

    // Create the state machine
    let mut machine = StateMachine::from_json(machine_json)?;

    // Register custom actions
    let actions = machine.actions_mut();
    actions.register(Arc::new(LogAction::new("Log message")));
    actions.register(Arc::new(MotorAction::new("disableMotor", false)));
    actions.register(Arc::new(MotorAction::new("enableMotor", true)));
    actions.register(Arc::new(MotorAction::new("rampUp", true)));
    actions.register(Arc::new(MotorAction::new("rampDown", false)));
    actions.register(Arc::new(MotorAction::new("logStop", false)));
    actions.register(Arc::new(MotorAction::new("logRunning", true)));
    actions.register(Arc::new(MotorAction::new("logExit", false)));
    actions.register(Arc::new(MotorAction::new("logError", false)));
    actions.register(Arc::new(MotorAction::new("triggerAlarm", false)));
    actions.register(Arc::new(MotorAction::new("checkSafety", false)));
    actions.register(Arc::new(MotorAction::new("clearError", false)));

    // Register custom guards
    let guards = machine.guards_mut();
    guards.register(Arc::new(SafetyGuard));
    guards.register(Arc::new(BoolGuard::new(
        "errorCleared".to_string(),
        "error_cleared".to_string(),
    )));

    // Set initial context - safety OK
    machine.context_mut().set_bool("emergency_stop", false);
    machine.context_mut().set_bool("door_open", false);

    println!("\n=== Motor Control State Machine Demo ===\n");
    println!("Initial state: {}", machine.current_state());
    println!("Available events: {:?}\n", machine.available_events());

    // Try to start the motor
    println!(">>> Sending START event");
    if machine.send("START")? {
        println!("Transitioned to: {}\n", machine.current_state());
    }

    // Simulate motor reaching running speed
    println!(">>> Sending RUNNING event");
    if machine.send("RUNNING")? {
        println!("Transitioned to: {}\n", machine.current_state());
    }

    // Stop the motor
    println!(">>> Sending STOP event");
    if machine.send("STOP")? {
        println!("Transitioned to: {}\n", machine.current_state());
    }

    // Complete stop
    println!(">>> Sending STOPPED event");
    if machine.send("STOPPED")? {
        println!("Transitioned to: {}\n", machine.current_state());
    }

    println!("\n=== Testing Emergency Stop ===\n");

    // Start again
    println!(">>> Sending START event");
    machine.send("START")?;
    machine.send("RUNNING")?;
    println!("Current state: {}\n", machine.current_state());

    // Emergency stop
    println!(">>> Sending EMERGENCY_STOP event");
    if machine.send("EMERGENCY_STOP")? {
        println!("Transitioned to: {}\n", machine.current_state());
    }

    // Try to reset without clearing error (should fail)
    println!(">>> Sending RESET event (error not cleared)");
    if !machine.send("RESET")? {
        println!("Reset blocked by guard\n");
    }

    // Clear error and reset
    machine.context_mut().set_bool("error_cleared", true);
    println!(">>> Sending RESET event (error cleared)");
    if machine.send("RESET")? {
        println!("Transitioned to: {}\n", machine.current_state());
    }

    println!("\n=== Testing Safety Guard ===\n");

    // Set door open (safety violation)
    machine.context_mut().set_bool("door_open", true);
    
    println!(">>> Sending START event (door open)");
    if !machine.send("START")? {
        println!("Start blocked by safety guard\n");
    }

    // Close door and retry
    machine.context_mut().set_bool("door_open", false);
    println!(">>> Sending START event (door closed)");
    if machine.send("START")? {
        println!("Transitioned to: {}\n", machine.current_state());
    }

    println!("\n=== Execution State ===\n");
    let exec_state = machine.execution_state();
    println!("Current: {}", exec_state.current_state);
    println!("Previous: {:?}", exec_state.previous_state);
    println!("Available events: {:?}", exec_state.available_events);
    println!("Last transition: {} ms", exec_state.timestamp);

    Ok(())
}
