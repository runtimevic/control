use super::ServoTestMachineMock;
use crate::servo_test_machine::api::{CommandAckEvent, Mutation, ServoTestMachineEvents};
use crate::{MachineApi, MachineMessage};
use control_core::socketio::namespace::{Namespace, NamespaceCacheingLogic};
use serde_json::Value;

impl MachineApi for ServoTestMachineMock {
    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> {
        self.api_sender.clone()
    }
    
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        
        let command_name = format!("{:?}", mutation);
        let result = self.handle_mutation(mutation);
        
        // Send acknowledgment
        let ack = CommandAckEvent {
            command: command_name,
            success: result.is_ok(),
            error_message: result.as_ref().err().map(|e| e.to_string()),
        }
        .build();
        
        self.namespace.emit(ServoTestMachineEvents::CommandAck(ack));
        
        // Emit updated state immediately
        self.emit_state();
        
        result
    }
    
    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}

impl ServoTestMachineMock {
    fn handle_mutation(&mut self, mutation: Mutation) -> Result<(), anyhow::Error> {
        match mutation {
            Mutation::JogStart { JogStart: direction } => {
                tracing::info!("Mock JOG Start: {}", direction);
                if self.operation_enabled {
                    self.target_velocity = match direction.as_str() {
                        "forward" => self.ref_velocity,
                        "backward" => -self.ref_velocity,
                        _ => 0.0,
                    };
                }
                Ok(())
            }
            Mutation::JogStop => {
                tracing::info!("Mock JOG Stop");
                self.target_velocity = 0.0;
                Ok(())
            }
            Mutation::Start => {
                tracing::info!("Mock Start");
                self.ready = true;
                self.switched_on = true;
                self.operation_enabled = true;
                self.fault = false;
                Ok(())
            }
            Mutation::Stop => {
                tracing::info!("Mock Stop");
                self.operation_enabled = false;
                self.switched_on = false;
                self.velocity = 0.0;
                Ok(())
            }
            Mutation::Reset => {
                tracing::info!("Mock Reset");
                self.fault = false;
                self.error_code = 0;
                self.ready = true;
                Ok(())
            }
            Mutation::Reference => {
                tracing::info!("Mock Reference");
                // Reset position to 0 (simulated homing)
                self.position = 0.0;
                self.target_position = 0.0;
                Ok(())
            }
            Mutation::SetEnabling { SetEnabling: params } => {
                tracing::info!("Mock Set Enabling: {:?}", params);
                self.operation_enabled = params.controller;
                self.override_value = params.override_value;
                Ok(())
            }
            Mutation::DownloadKvFactor { DownloadKvFactor: value } => {
                tracing::info!("Mock Download KV Factor: {}", value);
                self.kv_factor = value;
                Ok(())
            }
            Mutation::DownloadRefVelocity { DownloadRefVelocity: value } => {
                tracing::info!("Mock Download Ref Velocity: {}", value);
                self.ref_velocity = value;
                Ok(())
            }
            Mutation::DownloadTargetPosition { DownloadTargetPosition: value } => {
                tracing::info!("Mock Download Target Position: {}", value);
                self.target_position = value;
                Ok(())
            }
            Mutation::StartMovement { StartMovement: params } => {
                tracing::info!("Mock Start Movement: {:?}", params);
                if let Some(pos) = params.targetPosition {
                    self.target_position = pos;
                }
                if let Some(vel) = params.targetVelocity {
                    self.ref_velocity = vel;
                }
                Ok(())
            }
            Mutation::SetRawOutput { SetRawOutput: value } => {
                tracing::info!("Mock Set Raw Output: {}", value);
                // Simulate direct velocity command
                self.target_velocity = value as f32;
                Ok(())
            }
        }
    }
}
