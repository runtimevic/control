//! Hardware-specific actions and guards for state machines
//!
//! This module provides pre-defined actions and guards that can be used
//! in state machines to control hardware devices, check safety conditions, etc.

use control_core::state_machine::{Action, ActionRegistry, Guard, GuardRegistry, MachineContext};
use anyhow::Result;
use std::sync::Arc;

/// Example action: Log a message
pub struct LogHardwareAction {
    message: String,
}

impl LogHardwareAction {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Action for LogHardwareAction {
    fn execute(&self, _context: &mut MachineContext) -> Result<()> {
        tracing::info!("[StateMachine Hardware] {}", self.message);
        Ok(())
    }

    fn name(&self) -> &str {
        "logHardware"
    }
}

/// Action: Set a digital output (hardware)
pub struct SetDigitalOutputAction {
    name: String,
    output_id: u8,
    value: bool,
}

impl SetDigitalOutputAction {
    pub fn new(name: impl Into<String>, output_id: u8, value: bool) -> Self {
        Self {
            name: name.into(),
            output_id,
            value,
        }
    }
}

impl Action for SetDigitalOutputAction {
    fn execute(&self, context: &mut MachineContext) -> Result<()> {
        tracing::info!(
            "[StateMachine Hardware] {} → Output {} = {}",
            self.name,
            self.output_id,
            self.value
        );
        
        // Store in context for tracking
        context.set_bool(&format!("output_{}", self.output_id), self.value);
        
        // TODO: Integrate with actual hardware control when running on Linux
        // Example:
        // let ethercat = app_state.ethercat.lock().await;
        // ethercat.devices[0].set_output(self.output_id, self.value)?;
        
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Action: Ramp motor speed gradually
pub struct RampMotorAction {
    name: String,
    target_speed: f64,
    is_up: bool,
}

impl RampMotorAction {
    pub fn new(name: impl Into<String>, target_speed: f64, is_up: bool) -> Self {
        Self {
            name: name.into(),
            target_speed,
            is_up,
        }
    }
}

impl Action for RampMotorAction {
    fn execute(&self, context: &mut MachineContext) -> Result<()> {
        let direction = if self.is_up { "up" } else { "down" };
        tracing::info!(
            "[StateMachine Hardware] {} → Ramping {} to {} RPM",
            self.name,
            direction,
            self.target_speed
        );
        
        context.set_float("target_speed", self.target_speed);
        context.set_float("motor_speed", if self.is_up { self.target_speed * 0.5 } else { 0.0 });
        
        // TODO: Implement gradual speed ramping
        // Spawn async task to gradually change speed over time
        
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Guard: Check if system is in safe state
pub struct SystemSafeGuard;

impl Guard for SystemSafeGuard {
    fn evaluate(&self, context: &MachineContext) -> Result<bool> {
        // Check multiple safety conditions
        // Inputs are normally closed (NC), so HIGH = safe
        let emergency_stop_ok = context.get_bool("input_0").unwrap_or(false);
        let door_closed = context.get_bool("input_1").unwrap_or(false);
        
        let is_safe = emergency_stop_ok && door_closed;
        
        if is_safe {
            tracing::info!("[StateMachine Hardware] System is SAFE (e_stop={}, door={})", emergency_stop_ok, door_closed);
        } else {
            tracing::warn!("[StateMachine Hardware] System is UNSAFE (e_stop={}, door={})", emergency_stop_ok, door_closed);
        }
        
        Ok(is_safe)
    }

    fn name(&self) -> &str {
        "systemSafe"
    }
}

/// Guard: Check if motor has reached target speed
pub struct MotorSpeedOkGuard;

impl Guard for MotorSpeedOkGuard {
    fn evaluate(&self, context: &MachineContext) -> Result<bool> {
        let current_speed = context.get_float("motor_speed").unwrap_or(0.0);
        let target_speed = context.get_float("target_speed").unwrap_or(100.0);
        
        let speed_ok = current_speed >= (target_speed * 0.9);
        
        if speed_ok {
            tracing::info!(
                "[StateMachine Hardware] Motor speed OK: {} RPM (target: {} RPM)",
                current_speed,
                target_speed
            );
        }
        
        Ok(speed_ok)
    }

    fn name(&self) -> &str {
        "motorSpeedOk"
    }
}

/// Guard: Check if motor has stopped (speed near zero)
pub struct MotorSpeedZeroGuard;

impl Guard for MotorSpeedZeroGuard {
    fn evaluate(&self, context: &MachineContext) -> Result<bool> {
        let current_speed = context.get_float("motor_speed").unwrap_or(0.0);
        let stopped = current_speed < 1.0;
        
        if stopped {
            tracing::info!("[StateMachine Hardware] Motor stopped: {} RPM", current_speed);
        }
        
        Ok(stopped)
    }

    fn name(&self) -> &str {
        "motorSpeedZero"
    }
}

/// Guard: Check if error has been cleared
pub struct ErrorClearedGuard;

impl Guard for ErrorClearedGuard {
    fn evaluate(&self, context: &MachineContext) -> Result<bool> {
        let error_acknowledged = context.get_bool("error_acknowledged").unwrap_or(false);
        
        // Also verify safety conditions are restored
        let emergency_stop_ok = context.get_bool("input_0").unwrap_or(false);
        let door_closed = context.get_bool("input_1").unwrap_or(false);
        
        let can_reset = error_acknowledged && emergency_stop_ok && door_closed;
        
        if !can_reset {
            tracing::warn!(
                "[StateMachine Hardware] Cannot reset: ack={}, e_stop={}, door={}",
                error_acknowledged,
                emergency_stop_ok,
                door_closed
            );
        }
        
        Ok(can_reset)
    }

    fn name(&self) -> &str {
        "errorCleared"
    }
}

/// Example guard: Check if a variable is true
pub struct VariableTrueGuard {
    variable_name: String,
}

impl VariableTrueGuard {
    pub fn new(variable_name: String) -> Self {
        Self { variable_name }
    }
}

impl Guard for VariableTrueGuard {
    fn evaluate(&self, context: &MachineContext) -> Result<bool> {
        Ok(context.get_bool(&self.variable_name).unwrap_or(false))
    }

    fn name(&self) -> &str {
        &self.variable_name
    }
}

/// Register all hardware actions into the registry
pub fn register_actions(registry: &mut ActionRegistry) {
    // Logging
    registry.register(Arc::new(LogHardwareAction::new("Log")));
    
    // Motor control
    registry.register(Arc::new(SetDigitalOutputAction::new("disableMotor", 0, false)));
    registry.register(Arc::new(SetDigitalOutputAction::new("enableMotor", 0, true)));
    registry.register(Arc::new(RampMotorAction::new("rampUp", 1500.0, true)));
    registry.register(Arc::new(RampMotorAction::new("rampDown", 0.0, false)));
    
    // Lights (traffic light style indicator)
    registry.register(Arc::new(SetDigitalOutputAction::new("setRedLight", 1, true)));
    registry.register(Arc::new(SetDigitalOutputAction::new("clearRedLight", 1, false)));
    registry.register(Arc::new(SetDigitalOutputAction::new("setYellowLight", 2, true)));
    registry.register(Arc::new(SetDigitalOutputAction::new("clearYellowLight", 2, false)));
    registry.register(Arc::new(SetDigitalOutputAction::new("setGreenLight", 3, true)));
    registry.register(Arc::new(SetDigitalOutputAction::new("clearGreenLight", 3, false)));
    
    // Alarm
    registry.register(Arc::new(SetDigitalOutputAction::new("triggerAlarm", 4, true)));
    registry.register(Arc::new(SetDigitalOutputAction::new("clearAlarm", 4, false)));
    
    // Generic logging actions for state transitions
    registry.register(Arc::new(LogHardwareAction::new("logStop")));
    registry.register(Arc::new(LogHardwareAction::new("logStartRequest")));
    registry.register(Arc::new(LogHardwareAction::new("logSafetyCheck")));
    registry.register(Arc::new(LogHardwareAction::new("logSafetyPassed")));
    registry.register(Arc::new(LogHardwareAction::new("logSafetyFailed")));
    registry.register(Arc::new(LogHardwareAction::new("logStarting")));
    registry.register(Arc::new(LogHardwareAction::new("logMotorReady")));
    registry.register(Arc::new(LogHardwareAction::new("logRunning")));
    registry.register(Arc::new(LogHardwareAction::new("logStopping")));
    registry.register(Arc::new(LogHardwareAction::new("logError")));
    registry.register(Arc::new(LogHardwareAction::new("logReset")));
    registry.register(Arc::new(LogHardwareAction::new("logDoorOpened")));
    registry.register(Arc::new(LogHardwareAction::new("logStartTimeout")));
    registry.register(Arc::new(LogHardwareAction::new("checkEmergencyStop")));
    registry.register(Arc::new(LogHardwareAction::new("checkDoorClosed")));
    registry.register(Arc::new(LogHardwareAction::new("acknowledgeStop")));
    registry.register(Arc::new(LogHardwareAction::new("confirmStopped")));
    registry.register(Arc::new(LogHardwareAction::new("emergencyShutdown")));
    registry.register(Arc::new(LogHardwareAction::new("clearError")));
    registry.register(Arc::new(LogHardwareAction::new("enableFullPower")));
    registry.register(Arc::new(LogHardwareAction::new("disableFullPower")));
}

/// Register all hardware guards into the registry
pub fn register_guards(registry: &mut GuardRegistry) {
    // Safety guards
    registry.register(Arc::new(SystemSafeGuard));
    registry.register(Arc::new(ErrorClearedGuard));
    
    // Motor state guards
    registry.register(Arc::new(MotorSpeedOkGuard));
    registry.register(Arc::new(MotorSpeedZeroGuard));
}
