use std::time::Instant;

use super::ServoTestMachineMock;
use crate::servo_test_machine::api::ServoTestMachineNamespace;
use crate::{MachineNewParams, MachineNewTrait};
use anyhow::Error;

impl MachineNewTrait for ServoTestMachineMock {
    fn new<'maindevice, 'subdevices>(
        params: &MachineNewParams<'maindevice, 'subdevices, '_, '_, '_, '_, '_>,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let now = Instant::now();
        let (sender, receiver) = smol::channel::unbounded();
        
        let mut machine = Self {
            machine_identification_unique: params.get_machine_identification_unique(),
            main_sender: params.main_thread_channel.clone(),
            api_sender: sender,
            api_receiver: receiver,
            namespace: ServoTestMachineNamespace {
                namespace: params.namespace.clone(),
            },
            last_state_emit: now,
            
            // Initial simulated state
            position: 0.0,
            target_position: 0.0,
            velocity: 0.0,
            target_velocity: 0.0,
            error_code: 0,
            
            // Start in ready but not enabled state
            ready: true,
            switched_on: false,
            operation_enabled: false,
            fault: false,
            target_reached: true,
            
            // Default control parameters
            kv_factor: 1.0,
            ref_velocity: 100.0, // units/s
            override_value: 10000, // 100%
            
            t_0: now,
        };
        
        machine.emit_state();
        
        Ok(machine)
    }
}
