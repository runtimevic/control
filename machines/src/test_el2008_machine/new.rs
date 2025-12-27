use crate::test_el2008_machine::TestEL2008Machine;
use crate::test_el2008_machine::api::TestEL2008MachineNamespace;
use smol::block_on;
use std::time::Instant;

use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_dublicates, validate_same_machine_identification_unique,
};

use anyhow::Error;
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
            let el2008 = get_ethercat_device::<EL2008>(
                hardware,
                params,
                1,
                [EL2008_IDENTITY_A, EL2008_IDENTITY_B, EL2008_IDENTITY_C].to_vec(),
            )
            .await?
            .0;
            let do1 = DigitalOutput::new(el2008.clone(), EL2008Port::DO1);
            let do2 = DigitalOutput::new(el2008.clone(), EL2008Port::DO2);
            let do3 = DigitalOutput::new(el2008.clone(), EL2008Port::DO3);
            let do4 = DigitalOutput::new(el2008.clone(), EL2008Port::DO4);
            let do5 = DigitalOutput::new(el2008.clone(), EL2008Port::DO5);
            let do6 = DigitalOutput::new(el2008.clone(), EL2008Port::DO6);
            let do7 = DigitalOutput::new(el2008.clone(), EL2008Port::DO7);
            let do8 = DigitalOutput::new(el2008.clone(), EL2008Port::DO8);

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
                douts: [do1, do2, do3, do4, do5, do6, do7, do8],
            };
            my_test.emit_state();
            Ok(my_test)
        })
    }
}