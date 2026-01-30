use crate::servo_test_machine::ServoTestMachine;
use crate::servo_test_machine::api::ServoTestMachineNamespace;
use smol::block_on;
use std::time::Instant;

use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_dublicates, validate_same_machine_identification_unique,
};

use anyhow::Error;
use ethercat_hal::devices::adapters::{ServoAdapter, ServoDevice};
use ethercat_hal::devices::{LICHUAN_LC10E_IDENTITY, SMC_MITSUBISHI_IDENTITY};

impl<T: ServoDevice + Default + 'static> MachineNewTrait for ServoTestMachine<T> {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
        // Validate general stuff
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
                    "[{}::MachineNewTrait/ServoTestMachine::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        block_on(async {
            // Try to find a servo drive - supports both Lichuan and Mitsubishi
            // The actual type T will be determined by the caller
            let servo_adapter = get_ethercat_device::<ServoAdapter<T>>(
                hardware,
                params,
                1,
                vec![
                    LICHUAN_LC10E_IDENTITY,
                    SMC_MITSUBISHI_IDENTITY,
                ],
            )
            .await?
            .0;

            {
                let servo = servo_adapter.read().await;
                tracing::info!("Servo drive initialized: {:?}", servo);
            }

            let (sender, receiver) = smol::channel::unbounded();
            let mut machine = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: ServoTestMachineNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                main_sender: params.main_thread_channel.clone(),
                servo: servo_adapter,
            };
            
            machine.emit_state().await;
            Ok(machine)
        })
    }
}
