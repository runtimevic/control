use super::ServoTestMachine;
use crate::{MachineApi, MachineMessage};
use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use ethercat_hal::devices::adapters::ServoDevice;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Drive state event matching frontend DriveStateEventData schema
#[derive(Serialize, Debug, Clone)]
pub struct DriveStateEvent {
    pub position: f32,
    pub setpoint_position: f32,
    pub velocity: f32,
    pub error_code: u16,
    pub lag_error_current: f32,
    pub lag_error_min: f32,
    pub lag_error_max: f32,
    pub actual_velocity: f32,
    pub setpoint_velocity: f32,
    #[serde(rename = "override")]
    pub override_value: u16, // 0-10000 (100.00%)
    pub output_percent: f32,
    pub controller_output_percent: f32,
    // Status flags (logical)
    pub ready: bool,
    pub calibrated: bool,
    pub has_job: bool,
    pub not_moving: bool,
    pub moving_forward: bool,
    pub moving_backward: bool,
    // Status flags (physical)
    pub coupled_mode: bool,
    pub in_target_pos: bool,
    pub in_pos_range: bool,
    // Enabling
    pub controller_enabled: bool,
    pub feed_fw_enabled: bool,
    pub feed_bw_enabled: bool,
}

impl DriveStateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("DriveStateEvent", self.clone())
    }
}

/// Command acknowledgment event
#[derive(Serialize, Debug, Clone)]
pub struct CommandAckEvent {
    pub command: String,
    pub success: bool,
    pub error_message: Option<String>,
}

impl CommandAckEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("CommandAckEvent", self.clone())
    }
}

/// Movement complete event
#[derive(Serialize, Debug, Clone)]
pub struct MovementCompleteEvent {
    pub final_position: f32,
    pub time_ms: u64,
}

impl MovementCompleteEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("MovementCompleteEvent", self.clone())
    }
}

pub enum ServoTestMachineEvents {
    DriveState(Event<DriveStateEvent>),
    CommandAck(Event<CommandAckEvent>),
    MovementComplete(Event<MovementCompleteEvent>),
}

/// Mutation commands from frontend
#[derive(Deserialize, Debug)]
pub enum Mutation {
    #[serde(rename = "JogStart")]
    JogStart { 
        jog_start: String 
    },
    JogStop,
    Start,
    Stop,
    Reset,
    Reference,
    #[serde(rename = "SetEnabling")]
    SetEnabling {
        set_enabling: EnablingParams,
    },
    #[serde(rename = "DownloadKvFactor")]
    DownloadKvFactor(f32),
    #[serde(rename = "DownloadRefVelocity")]
    DownloadRefVelocity(f32),
    #[serde(rename = "DownloadTargetPosition")]
    DownloadTargetPosition(f32),
    #[serde(rename = "SetProfileVelocity")]
    SetProfileVelocity(u32),
    #[serde(rename = "StartMovement")]
    StartMovement {
        start_movement: MovementParams,
    },
    #[serde(rename = "SetRawOutput")]
    SetRawOutput {
        set_raw_output: i16,
    },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EnablingParams {
    pub controller: bool,
    pub feed_fw: bool,
    pub feed_bw: bool,
    #[serde(rename = "override")]
    pub override_value: u16,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MovementParams {
    pub mode: String,
    pub target_position: Option<f32>,
    pub target_velocity: Option<f32>,
    pub acceleration: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct ServoTestMachineNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<ServoTestMachineEvents> for ServoTestMachineNamespace {
    fn emit(&mut self, events: ServoTestMachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<ServoTestMachineEvents> for ServoTestMachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            ServoTestMachineEvents::DriveState(event) => event.clone().into(),
            ServoTestMachineEvents::CommandAck(event) => event.clone().into(),
            ServoTestMachineEvents::MovementComplete(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl<T: ServoDevice> MachineApi for ServoTestMachine<T> {
    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        
        let command_name = format!("{:?}", mutation);
        let result = smol::block_on(self.handle_mutation(mutation));
        
        // Send acknowledgment
        let ack = CommandAckEvent {
            command: command_name,
            success: result.is_ok(),
            error_message: result.as_ref().err().map(|e| e.to_string()),
        }
        .build();
        
        self.namespace.emit(ServoTestMachineEvents::CommandAck(ack));
        
        // Emit updated state immediately
        smol::block_on(self.emit_state());
        
        result
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}

impl<T: ServoDevice> ServoTestMachine<T> {
    async fn handle_mutation(&mut self, mutation: Mutation) -> Result<(), anyhow::Error> {
        match mutation {
            Mutation::JogStart { jog_start: direction } => {
                tracing::info!("JOG Start: {}", direction);
                let mut servo = self.servo.write().await;
                
                // Set mode to Cyclic Synchronous Velocity (CSV, mode 9)
                // ProfileVelocity (mode 3) is not implemented in LichuanSimulator
                // CSV mode provides direct velocity control, perfect for jogging
                servo.servo_mut().set_mode_of_operation(9)?;
                
                // Set target velocity (positive for forward, negative for backward)
                // Using 10000 increments/s as jog speed (típico para contadores de encoder)
                let target_velocity = if direction == "forward" { 10000 } else { -10000 };
                servo.servo_mut().set_target_velocity(target_velocity)?;
                
                // Enable operation and start movement
                // Control word sequence for CIA402:
                // 0x0006 = shutdown
                // 0x0007 = switch on
                // 0x000F = enable operation
                servo.servo_mut().process_control_word(0x0006)?; // Shutdown
                servo.servo_mut().process_control_word(0x0007)?; // Switch On  
                servo.servo_mut().process_control_word(0x000F)?; // Enable Operation
                
                tracing::debug!("Jog started: mode=9 (CSV), velocity={}", target_velocity);
                
                Ok(())
            }
            Mutation::JogStop => {
                tracing::info!("JOG Stop");
                let mut servo = self.servo.write().await;
                
                // Stop movement by setting velocity to 0
                servo.servo_mut().set_target_velocity(0)?;
                
                tracing::debug!("Jog stopped: velocity=0");
                
                Ok(())
            }
            Mutation::Start => {
                tracing::info!("Start");
                let mut servo = self.servo.write().await;
                // CiA402: Enable operation (control word: shutdown -> switch on -> enable operation)
                servo.servo_mut().process_control_word(0x0006)?; // Shutdown
                servo.servo_mut().process_control_word(0x0007)?; // Switch On
                servo.servo_mut().process_control_word(0x000F)?; // Enable Operation
                Ok(())
            }
            Mutation::Stop => {
                tracing::info!("Stop");
                let mut servo = self.servo.write().await;
                // CiA402: Disable operation (control word: 0x0006 for quick stop or 0x0007 for disable voltage)
                servo.servo_mut().process_control_word(0x0006)?;
                Ok(())
            }
            Mutation::Reset => {
                tracing::info!("Reset");
                let mut servo = self.servo.write().await;
                // CiA402: Fault reset (control word: 0x0080)
                servo.servo_mut().process_control_word(0x0080)?;
                Ok(())
            }
            Mutation::Reference => {
                tracing::info!("Reference");
                // TODO: Implement homing
                Ok(())
            }
            Mutation::SetEnabling { set_enabling: params } => {
                tracing::info!("Set Enabling: {:?}", params);
                // TODO: Configure enabling logic
                Ok(())
            }
            Mutation::DownloadKvFactor(value) => {
                tracing::info!("Download KV Factor: {}", value);
                // TODO: Write to servo SDO
                Ok(())
            }
            Mutation::DownloadRefVelocity(value) => {
                tracing::info!("Download Ref Velocity: {}", value);
                // TODO: Write to servo SDO
                Ok(())
            }
            Mutation::DownloadTargetPosition(value) => {
                tracing::info!("Download Target Position: {} - Starting movement", value);
                let mut servo = self.servo.write().await;
                
                // Set mode to Cyclic Synchronous Position (CSP, mode 8)
                servo.servo_mut().set_mode_of_operation(8)?;
                tracing::debug!("Mode set to 8 (CSP)");
                
                // Set target position
                let target_pos = value as i32;
                tracing::info!("Setting target_position to i32: {}", target_pos);
                servo.servo_mut().set_target_position(target_pos)?;
                tracing::debug!("Target position set to {}", target_pos);
                
                // Enable operation and start movement
                // CIA402 state machine: Shutdown -> SwitchOn -> EnableOperation
                servo.servo_mut().process_control_word(0x0006)?; // Shutdown
                tracing::debug!("Control word: Shutdown (0x0006)");
                
                servo.servo_mut().process_control_word(0x0007)?; // Switch On  
                tracing::debug!("Control word: Switch On (0x0007)");
                
                servo.servo_mut().process_control_word(0x000F)?; // Enable Operation
                tracing::debug!("Control word: Enable Operation (0x000F)");
                
                // Don't call update() here - the act() loop will handle simulation updates
                // Check status
                let status = servo.servo().get_status_word()?;
                let mode_display = servo.servo().get_mode_of_operation_display()?;
                let pos_actual = servo.servo().get_position_actual()?;
                tracing::info!(
                    "Position command sent: target={}, current_pos={}, mode=8 (CSP), status=0x{:04X}, mode_display={}",
                    target_pos, pos_actual, status, mode_display
                );
                
                Ok(())
            }
            Mutation::SetProfileVelocity(velocity) => {
                tracing::info!("Set Profile Velocity: {} units/s", velocity);
                let mut servo = self.servo.write().await;
                
                // Set profile velocity for CSP movements (objeto CoE 0x6081)
                servo.set_profile_velocity(velocity)?;
                
                // Read back to confirm
                let actual_velocity = servo.get_profile_velocity()?;
                tracing::info!("Profile velocity set to: {} units/s", actual_velocity);
                
                Ok(())
            }
            Mutation::StartMovement { start_movement: params } => {
                tracing::info!("Start Movement: {:?}", params);
                let mut servo = self.servo.write().await;
                
                match params.mode.as_str() {
                    "position" => {
                        if let Some(target_pos) = params.target_position {
                            // Set mode to Cyclic Synchronous Position (CSP, mode 8)
                            // This mode is implemented in the simulator
                            servo.servo_mut().set_mode_of_operation(8)?;
                            
                            // Set target position
                            servo.servo_mut().set_target_position(target_pos as i32)?;
                            
                            // Enable operation if not already enabled
                            servo.servo_mut().process_control_word(0x0006)?; // Shutdown
                            servo.servo_mut().process_control_word(0x0007)?; // Switch On  
                            servo.servo_mut().process_control_word(0x000F)?; // Enable Operation
                            
                            tracing::info!("Position movement started: target={}, mode=8 (CSP)", target_pos);
                        } else {
                            tracing::warn!("Position mode requires target_position");
                        }
                    }
                    "velocity" => {
                        if let Some(target_vel) = params.target_velocity {
                            // Set mode to Cyclic Synchronous Velocity (CSV, mode 9)
                            servo.servo_mut().set_mode_of_operation(9)?;
                            servo.servo_mut().set_target_velocity(target_vel as i32)?;
                            
                            // Enable operation
                            servo.servo_mut().process_control_word(0x0006)?;
                            servo.servo_mut().process_control_word(0x0007)?;
                            servo.servo_mut().process_control_word(0x000F)?;
                            
                            tracing::info!("Velocity movement started: target={}, mode=9 (CSV)", target_vel);
                        } else {
                            tracing::warn!("Velocity mode requires target_velocity");
                        }
                    }
                    _ => {
                        tracing::warn!("Unknown movement mode: {}", params.mode);
                    }
                }
                
                Ok(())
            }
            Mutation::SetRawOutput { set_raw_output: value } => {
                tracing::info!("Set Raw Output: {}", value);
                // TODO: Write raw torque/velocity command
                Ok(())
            }
        }
    }
}
