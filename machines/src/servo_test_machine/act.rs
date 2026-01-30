use super::ServoTestMachine;
use crate::{MachineAct, MachineMessage};
use ethercat_hal::devices::servo::ServoDevice;
use smol::block_on;
use std::time::{Duration, Instant};

impl<T: ServoDevice> MachineAct for ServoTestMachine<T> {
    fn act(&mut self, now: Instant) {
        // Handle API messages
        if let Ok(msg) = self.api_receiver.try_recv() {
            self.act_machine_message(msg);
        }

        // Emit state at 10 Hz (100ms) to match frontend expectation
        if now.duration_since(self.last_state_emit) > Duration::from_millis(100) {
            block_on(self.emit_state());
            self.last_state_emit = now;
        }
    }

    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                smol::block_on(self.emit_state());
            }
            MachineMessage::UnsubscribeNamespace => self.namespace.namespace = None,
            MachineMessage::HttpApiJsonRequest(value) => {
                use crate::MachineApi;
                let _res = self.api_mutate(value);
            }
            MachineMessage::ConnectToMachine(_machine_connection) => {
                // Does not connect to any Machine; do nothing
            }
            MachineMessage::DisconnectMachine(_machine_connection) => {
                // Does not connect to any Machine; do nothing
            }
        }
    }
}
