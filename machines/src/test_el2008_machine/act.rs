use super::TestEL2008Machine;
use crate::{MachineAct, MachineMessage};
use std::time::{Duration, Instant};

impl MachineAct for TestEL2008Machine {
    fn act(&mut self, now: Instant) {
        if let Ok(msg) = self.api_receiver.try_recv() {
            self.act_machine_message(msg);
        }

        // Execute automatic mode logic
        self.execute_automatic(now);

        // Emit state at 30 Hz
        if now.duration_since(self.last_state_emit) > Duration::from_secs_f64(1.0 / 30.0) {
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
            MachineMessage::UnsubscribeNamespace => self.namespace.namespace = None,
            MachineMessage::HttpApiJsonRequest(value) => {
                tracing::info!("[test_el2008_machine] Received HttpApiJsonRequest: {:?}", value);
                use crate::MachineApi;
                let res = self.api_mutate(value.clone());
                match res {
                    Ok(_) => tracing::info!("[test_el2008_machine] Successfully executed mutation: {:?}", value),
                    Err(e) => tracing::error!("[test_el2008_machine] Failed to execute mutation: {:?}, error: {:?}", value, e),
                }
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