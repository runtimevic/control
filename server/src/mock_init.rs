use crate::add_serial_device;
use crate::app_state::{SharedState, HotThreadMessage};
use crate::socketio::main_namespace::machines_event::MachineObj;
use machines::registry::MACHINE_REGISTRY;
use machines::servo_test_machine::ServoTestMachine;
use machines::test_el2008_machine::TestEL2008Machine;
use machines::machine_identification::{MachineIdentification, MachineIdentificationUnique};
use machines::MachineApi;
use ethercat_hal::devices::lichuan::LichuanSimulator;
use std::sync::Arc;

pub fn init_mock(app_state: Arc<SharedState>) -> Result<(), anyhow::Error> {
    // For mock devices, we need to manually create and add them to the machine manager
    // since they won't be detected by the serial detection loop
    return smol::block_on(async {
        // Create a mock serial device manually

        use machines::{
            SerialDeviceNew, SerialDeviceNewParams,
            serial::devices::{
                extruder_mock::ExtruderMockSerialDevice, mock::MockSerialDevice,
                winder_mock::WinderMockSerialDevice,
            },
        };
        let serial_params = SerialDeviceNewParams {
            path: "/dev/mock-serial".to_string(),
        };

        // Create the mock serial device
        let _ = match MockSerialDevice::new_serial(&serial_params) {
            Ok((device_identification, mock_serial_device)) => {
                // Add the mock device to the machine manager
                {
                    use crate::add_serial_device;
                    use machines::registry::MACHINE_REGISTRY;
                    add_serial_device(
                        app_state.clone(),
                        &device_identification,
                        mock_serial_device,
                        &MACHINE_REGISTRY,
                        app_state.socketio_setup.socket_queue_tx.clone(),
                    )
                    .await;
                }
                Ok::<(), anyhow::Error>(())
            }
            Err(e) => {
                tracing::error!("Failed to create mock serial device: {}", e);
                return Err(e);
            }
        };

        let _ = match ExtruderMockSerialDevice::new_serial(&serial_params) {
            Ok((device_identification, mock_serial_device)) => {
                // Add the mock device to the machine manager
                {
                    use crate::add_serial_device;
                    use machines::registry::MACHINE_REGISTRY;

                    add_serial_device(
                        app_state.clone(),
                        &device_identification,
                        mock_serial_device,
                        &MACHINE_REGISTRY,
                        app_state.socketio_setup.socket_queue_tx.clone(),
                    )
                    .await;
                }

                Ok::<(), anyhow::Error>(())
            }
            Err(e) => {
                tracing::error!("Failed to create extruder mock serial device: {}", e);
                return Err(e);
            }
        };

        match WinderMockSerialDevice::new_serial(&serial_params) {
            Ok((device_identification, mock_serial_device)) => {
                // Add the mock device to the machine manager
                add_serial_device(
                    app_state.clone(),
                    &device_identification,
                    mock_serial_device,
                    &MACHINE_REGISTRY,
                    app_state.clone().socketio_setup.socket_queue_tx.clone(),
                )
                .await;

                Ok::<(), anyhow::Error>(())
            }
            Err(e) => {
                tracing::error!("Failed to create winder mock serial device: {}", e);
                return Err(e);
            }
        }?;

        // Add ServoTestMachine with LichuanSimulator (no EtherCAT needed)
        {
            let machine_identification_unique = MachineIdentificationUnique {
                machine_identification: MachineIdentification {
                    vendor: 1, // VENDOR_QITECH
                    machine: 55, // SERVO_TEST_MACHINE
                },
                serial: 999,
            };

            match ServoTestMachine::<LichuanSimulator>::new_without_hardware(
                machine_identification_unique.clone(),
                Some(app_state.main_channel.clone()),
            ) {
                Ok(machine) => {
                    tracing::info!("Created ServoTestMachine with LichuanSimulator");
                    
                    // Add to machine metadata
                    app_state.add_machines_if_not_exists(vec![MachineObj {
                        machine_identification_unique: machine_identification_unique.clone(),
                        error: None,
                    }]).await;

                    // Add to API machines map
                    app_state
                        .api_machines
                        .lock()
                        .await
                        .insert(machine_identification_unique.clone(), machine.api_get_sender());

                    // Send machine to hot thread
                    let _ = app_state
                        .rt_machine_creation_channel
                        .send(HotThreadMessage::AddMachines(vec![Box::new(machine)]))
                        .await;

                    app_state.clone().send_machines_event().await;
                }
                Err(e) => {
                    tracing::error!("Failed to create ServoTestMachine with LichuanSimulator: {}", e);
                    return Err(e);
                }
            }
        }

        // Add TestEL2008Machine (mock outputs without hardware)
        {
            let machine_identification_unique = MachineIdentificationUnique {
                machine_identification: MachineIdentification {
                    vendor: 1, // VENDOR_QITECH
                    machine: 0x0036, // TEST_EL2008_MACHINE
                },
                serial: 1,
            };

            match TestEL2008Machine::new_without_hardware(
                machine_identification_unique.clone(),
                Some(app_state.main_channel.clone()),
            ) {
                Ok(machine) => {
                    tracing::info!("Created TestEL2008Machine (mock outputs)");
                    
                    // Add to machine metadata
                    app_state.add_machines_if_not_exists(vec![MachineObj {
                        machine_identification_unique: machine_identification_unique.clone(),
                        error: None,
                    }]).await;

                    // Add to API machines map
                    app_state
                        .api_machines
                        .lock()
                        .await
                        .insert(machine_identification_unique.clone(), machine.api_get_sender());

                    // Send machine to hot thread
                    let _ = app_state
                        .rt_machine_creation_channel
                        .send(HotThreadMessage::AddMachines(vec![Box::new(machine)]))
                        .await;

                    app_state.clone().send_machines_event().await;
                }
                Err(e) => {
                    tracing::error!("Failed to create TestEL2008Machine: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(())
    });
}
