use crate::test_el2008_machine::TestEL2008Machine;
use crate::test_el2008_machine::api::TestEL2008MachineNamespace;
use crate::test_el2008_machine::{AutomaticPhase, MachineState, TestEL2008Mode};
use smol::block_on;
use std::time::Instant;

use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_dublicates, validate_same_machine_identification_unique,
};

use anyhow::Error;
use ethercat_hal::devices::el2004::{EL2004, EL2004_IDENTITY_A, EL2004Port};
use ethercat_hal::devices::el2008::{EL2008, EL2008_IDENTITY_A, EL2008_IDENTITY_B, EL2008_IDENTITY_C, EL2008Port};
use ethercat_hal::io::digital_output::DigitalOutput;

impl MachineNewTrait for TestEL2008Machine {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
        // validate general stuff
        let device_identification = params
            .device_group
            .iter()
            .map(|device_identification| device_identification.clone())
            .collect::<Vec<_>>();
        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_dublicates(&device_identification)?;

        let hardware = match &params.hardware {
            MachineNewHardware::Ethercat(x) => x,
            _ => {
                return Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/TestEL2008Machine::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };
        block_on(async {
            // Get EL2004 (role 1) if present
            let el2004_result = get_ethercat_device::<EL2004>(
                hardware,
                params,
                1,
                vec![EL2004_IDENTITY_A],
            )
            .await;
            
            // Get EL2008 (role 2) if present
            let el2008_result = get_ethercat_device::<EL2008>(
                hardware,
                params,
                2,
                [EL2008_IDENTITY_A, EL2008_IDENTITY_B, EL2008_IDENTITY_C].to_vec(),
            )
            .await;

            // Determine which devices we have and build array accordingly
            let douts: [DigitalOutput; 8] = match (el2004_result, el2008_result) {
                // Both devices present
                (Ok((el2004_device, _)), Ok((el2008_device, _))) => {
                    [
                        DigitalOutput::new(el2004_device.clone(), EL2004Port::DO1),
                        DigitalOutput::new(el2004_device.clone(), EL2004Port::DO2),
                        DigitalOutput::new(el2004_device.clone(), EL2004Port::DO3),
                        DigitalOutput::new(el2004_device, EL2004Port::DO4),
                        DigitalOutput::new(el2008_device.clone(), EL2008Port::DO1),
                        DigitalOutput::new(el2008_device.clone(), EL2008Port::DO2),
                        DigitalOutput::new(el2008_device.clone(), EL2008Port::DO3),
                        DigitalOutput::new(el2008_device, EL2008Port::DO4),
                    ]
                },
                // Only EL2004
                (Ok((el2004_device, _)), Err(_)) => {
                    [
                        DigitalOutput::new(el2004_device.clone(), EL2004Port::DO1),
                        DigitalOutput::new(el2004_device.clone(), EL2004Port::DO2),
                        DigitalOutput::new(el2004_device.clone(), EL2004Port::DO3),
                        DigitalOutput::new(el2004_device.clone(), EL2004Port::DO4),
                        DigitalOutput::new(el2004_device.clone(), EL2004Port::DO1),
                        DigitalOutput::new(el2004_device.clone(), EL2004Port::DO2),
                        DigitalOutput::new(el2004_device.clone(), EL2004Port::DO3),
                        DigitalOutput::new(el2004_device, EL2004Port::DO4),
                    ]
                },
                // Only EL2008
                (Err(_), Ok((el2008_device, _))) => {
                    [
                        DigitalOutput::new(el2008_device.clone(), EL2008Port::DO1),
                        DigitalOutput::new(el2008_device.clone(), EL2008Port::DO2),
                        DigitalOutput::new(el2008_device.clone(), EL2008Port::DO3),
                        DigitalOutput::new(el2008_device.clone(), EL2008Port::DO4),
                        DigitalOutput::new(el2008_device.clone(), EL2008Port::DO5),
                        DigitalOutput::new(el2008_device.clone(), EL2008Port::DO6),
                        DigitalOutput::new(el2008_device.clone(), EL2008Port::DO7),
                        DigitalOutput::new(el2008_device, EL2008Port::DO8),
                    ]
                },
                // Neither device found
                (Err(_), Err(_)) => {
                    return Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/TestEL2008Machine::new] No EL2004 (role 1) or EL2008 (role 2) devices found",
                        module_path!()
                    ));
                }
            };

            let (sender, receiver) = smol::channel::unbounded();
            let mut my_test = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: TestEL2008MachineNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                led_on: [false; 8],
                main_sender: params.main_thread_channel.clone(),
                douts,
                mode: TestEL2008Mode::Manual,
                machine_state: MachineState::Stopped,
                automatic_phase: AutomaticPhase::Idle,
                automatic_delay_ms: 500, // Default 500ms delay
                last_automatic_step: Instant::now(),
            };
            my_test.emit_state();
            Ok(my_test)
        })
    }
}