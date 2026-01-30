use super::ServoTestMachineMock;
use crate::{MachineAct, MachineMessage};
use std::time::{Duration, Instant};

impl MachineAct for ServoTestMachineMock {
    fn act(&mut self, now: Instant) {
        // Handle API messages
        if let Ok(msg) = self.api_receiver.try_recv() {
            self.act_machine_message(msg);
        }
        
        // Simulate motion physics
        let dt = now.duration_since(self.last_state_emit).as_secs_f32();
        if dt > 0.0 {
            self.simulate_motion(dt);
        }
        
        // Emit state at 10 Hz (100ms)
        if now.duration_since(self.last_state_emit) > Duration::from_millis(100) {
            self.emit_state();
            self.last_state_emit = now;
        }
    }
    
    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                self.emit_state();
            }
            MachineMessage::UnsubscribeNamespace => {
                self.namespace.namespace = None;
            }
            MachineMessage::HttpApiJsonRequest(value) => {
                use crate::MachineApi;
                let _res = self.api_mutate(value);
            }
            MachineMessage::ConnectToMachine(_) => {
                // Does not connect to other machines
            }
            MachineMessage::DisconnectMachine(_) => {
                // Does not connect to other machines
            }
        }
    }
}
