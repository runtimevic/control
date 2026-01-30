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
#[serde(untagged)]
pub enum Mutation {
    JogStart { JogStart: String },
    JogStop,
    Start,
    Stop,
    Reset,
    Reference,
    SetEnabling {
        SetEnabling: EnablingParams,
    },
    DownloadKvFactor {
        DownloadKvFactor: f32,
    },
    DownloadRefVelocity {
        DownloadRefVelocity: f32,
    },
    DownloadTargetPosition {
        DownloadTargetPosition: f32,
    },
    StartMovement {
        StartMovement: MovementParams,
    },
    SetRawOutput {
        SetRawOutput: i16,
    },
}

#[derive(Deserialize, Debug)]
pub struct EnablingParams {
    pub controller: bool,
    pub feedFw: bool,
    pub feedBw: bool,
    #[serde(rename = "override")]
    pub override_value: u16,
}

#[derive(Deserialize, Debug)]
pub struct MovementParams {
    pub mode: String,
    pub targetPosition: Option<f32>,
    pub targetVelocity: Option<f32>,
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
            Mutation::JogStart { JogStart: direction } => {
                tracing::info!("JOG Start: {}", direction);
                // TODO: Implement jog logic based on direction
                Ok(())
            }
            Mutation::JogStop => {
                tracing::info!("JOG Stop");
                // TODO: Stop jog
                Ok(())
            }
            Mutation::Start => {
                tracing::info!("Start");
                let mut servo = self.servo.write().await;
                // CiA402: Enable operation (control word: shutdown -> switch on -> enable operation)
                // Simplified: send enable operation control word (0x000F)
                servo.servo_mut().process_control_word(0x000F)?;
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
            Mutation::SetEnabling { SetEnabling: params } => {
                tracing::info!("Set Enabling: {:?}", params);
                // TODO: Configure enabling logic
                Ok(())
            }
            Mutation::DownloadKvFactor { DownloadKvFactor: value } => {
                tracing::info!("Download KV Factor: {}", value);
                // TODO: Write to servo SDO
                Ok(())
            }
            Mutation::DownloadRefVelocity { DownloadRefVelocity: value } => {
                tracing::info!("Download Ref Velocity: {}", value);
                // TODO: Write to servo SDO
                Ok(())
            }
            Mutation::DownloadTargetPosition { DownloadTargetPosition: value } => {
                tracing::info!("Download Target Position: {}", value);
                let mut servo = self.servo.write().await;
                servo.servo_mut().set_target_position(value as i32)?;
                Ok(())
            }
            Mutation::StartMovement { StartMovement: params } => {
                tracing::info!("Start Movement: {:?}", params);
                // TODO: Implement movement modes
                Ok(())
            }
            Mutation::SetRawOutput { SetRawOutput: value } => {
                tracing::info!("Set Raw Output: {}", value);
                // TODO: Write raw torque/velocity command
                Ok(())
            }
        }
    }
}
