use crate::app_state::{EthercatSetup, HotThreadMessage};
use crate::performance_metrics::EthercatPerformanceMetrics;
use bitvec::prelude::*;
use machines::Machine;
use machines::machine_identification::write_machine_device_identification;
use smol::channel::Receiver;
use spin_sleep::SpinSleeper;
use std::time::Duration;
use std::time::Instant;

use crate::metrics::jitter::record_machines_loop_jitter;
pub struct RtLoopInputs<'a> {
    pub machines: &'a mut Vec<Box<dyn Machine>>,
    pub ethercat_setup: Option<Box<EthercatSetup>>,
    pub ethercat_perf_metrics: Option<&'a mut EthercatPerformanceMetrics>,
    pub sleeper: SpinSleeper,
    pub cycle_target: Duration,
    // Auto-recover watchdog
    pub consecutive_txrx_failures: u32,
    pub last_recover_attempt: Option<Instant>,
    pub recover_cooldown: Duration,
}

// 300 us loop cycle target
// SharedState is mostly read from and rarely locked, but does not contain any machine,ethercat devices etc
pub fn start_loop_thread(
    rt_receiver: Receiver<HotThreadMessage>,
    cycle_target: Duration,
) -> Result<std::thread::JoinHandle<()>, std::io::Error> {
    // Start control loop
    let res = std::thread::Builder::new()
        .name("loop".to_owned())
        .spawn(move || {
            let rt_receiver = rt_receiver.to_owned();
            let sleeper =
                SpinSleeper::new(3_333_333)
                    .with_spin_strategy(spin_sleep::SpinStrategy::YieldThread);
            let mut ethercat_perf = EthercatPerformanceMetrics::new();
            let mut machines: Vec<Box<dyn Machine>> = vec![];
            let mut last_iter_start: Option<Instant> = None;
            let mut rt_loop_inputs = RtLoopInputs {
                machines: &mut machines,
                ethercat_setup: None,
                sleeper,
                cycle_target,
                ethercat_perf_metrics: Some(&mut ethercat_perf),
                consecutive_txrx_failures: 0,
                last_recover_attempt: None,
                recover_cooldown: Duration::from_secs(3),
            };

            loop {
                let msg = match rt_receiver.try_recv() {
                    Ok(msg) => msg,
                    Err(_) => HotThreadMessage::NoMsg,
                };

                match msg {
                    HotThreadMessage::NoMsg => {}
                    HotThreadMessage::AddEtherCatSetup(ethercat_setup) => {
                        println!("EthercatSetup: {:?}", ethercat_setup.devices);
                        rt_loop_inputs.ethercat_setup = Some(Box::new(ethercat_setup));
                    }
                    HotThreadMessage::WriteMachineDeviceInfo(info_request) => {
                        if let Some(ethercat_setup) = &rt_loop_inputs.ethercat_setup {
                            if let Ok(subdevice) = ethercat_setup.group.subdevice(
                                &ethercat_setup.maindevice,
                                info_request
                                    .hardware_identification_ethercat
                                    .subdevice_index,
                            ) {
                                let _res = smol::block_on(write_machine_device_identification(
                                    &subdevice,
                                    &ethercat_setup.maindevice,
                                    &info_request.device_machine_identification,
                                ));
                                if let Err(e) = _res {
                                    tracing::error!("Failed to write machine device identification to EEPROM: {}", e);
                                } else {
                                    tracing::info!("Successfully wrote machine device identification to EEPROM");
                                }
                            }

                            // Recreate machines after assignment is handled by REST flow.
                            // No action in the RT loop.
                        }
                    }
                    HotThreadMessage::DeleteMachine(unique_id) => {
                        rt_loop_inputs
                            .machines
                            .retain(|m| m.get_machine_identification_unique() != unique_id);
                    }
                    HotThreadMessage::AddMachines(machine_vec) => {
                        tracing::info!("received machines{:?}", machine_vec);
                        for new_machine in machine_vec {
                            let id = new_machine.get_machine_identification_unique();
                            if !rt_loop_inputs
                                .machines
                                .iter()
                                .any(|m| m.get_machine_identification_unique() == id)
                            {
                                rt_loop_inputs.machines.push(new_machine);
                            }
                        }
                    }
                }
                let iter_start = Instant::now();
                if let Some(prev) = last_iter_start {
                    if let Some(period) = iter_start.checked_duration_since(prev) {
                        let jitter_ns = period.as_nanos() as i128
                            - rt_loop_inputs.cycle_target.as_nanos() as i128;
                        record_machines_loop_jitter(jitter_ns);
                    }
                }
                last_iter_start = Some(iter_start);

                if let Err(e) = loop_once(&mut rt_loop_inputs) {
                    tracing::error!(
                        "Loop failed\n {:?} \n Last Loop Took: {:?}",
                        e,
                        rt_loop_inputs
                            .ethercat_perf_metrics
                            .unwrap()
                            .last_loop_start
                            .unwrap()
                            .elapsed()
                    );
                    break;
                }
            }

            // Exit the entire program if the Loop fails
            // gets restarted by systemd if running on NixOS, or different distro wtih the same sysd service
            std::process::exit(1);
        });
    return res;
}

pub async fn copy_ethercat_inputs(
    ethercat_setup: Option<&EthercatSetup>,
) -> Result<bool, anyhow::Error> {
    // only if we have an ethercat setup
    // - tx/rx cycle
    // - copy inputs to devices
    if let Some(ethercat_setup) = ethercat_setup {
        match ethercat_setup
            .group
            .tx_rx(&ethercat_setup.maindevice)
            .await
        {
            Ok(_) => {
                // copy inputs to devices
                for (i, subdevice) in ethercat_setup
                    .group
                    .iter(&ethercat_setup.maindevice)
                    .enumerate()
                {
                    // retrieve inputs
                    let input = subdevice.inputs_raw();
                    let input_bits = input.view_bits::<Lsb0>();

                    // get device
                    let mut device = ethercat_setup.devices[i].1.as_ref().write().await;

                    // check if the device is used
                    if !device.is_used() {
                        // if the device is not used, we skip it
                        continue;
                    }

                    // put inputs into device
                    device.input_checked(input_bits).map_err(|e| {
                        anyhow::anyhow!(
                            "[{}::loop_once] SubDevice with index {} failed to copy inputs\n{:?}",
                            module_path!(),
                            i,
                            e
                        )
                    })?;

                    // post process inputs
                    device.input_post_process().map_err(|e| {
                        anyhow::anyhow!(
                            "[{}::loop_once] SubDevice with index {} failed to copy post_process\n{:?}",
                            module_path!(),
                            i,
                            e
                        )
                    })?;
                }
                return Ok(true);
            }
            Err(e) => {
                // If tx_rx fails (e.g., timeout due to network disconnection), log and skip this cycle
                tracing::warn!("EtherCAT tx_rx failed: {}. Skipping input copy.", e);
                return Ok(false);
            }
        }
    }
    Ok(true)
}

pub async fn copy_ethercat_outputs(
    ethercat_setup: Option<&EthercatSetup>,
) -> Result<(), anyhow::Error> {
    if let Some(ethercat_setup) = ethercat_setup {
        // copy outputs from devices
        for (i, subdevice) in ethercat_setup
            .group
            .iter(&ethercat_setup.maindevice)
            .enumerate()
        {
            // get output buffer for device
            let mut output = subdevice.outputs_raw_mut();
            let output_bits = output.view_bits_mut::<Lsb0>();

            // get device
            let mut device = ethercat_setup.devices[i].1.as_ref().write().await;

            // check if the device is used
            if !device.is_used() {
                // if the device is not used, we skip it
                continue;
            }

            // pre process outputs
            device.output_pre_process().map_err(|e| {
                anyhow::anyhow!(
                    "[{}::loop_once] SubDevice with index {} failed to pre process outputs \n{:?}",
                    module_path!(),
                    i,
                    e
                )
            })?;

            // put outputs into device
            device.output_checked(output_bits).map_err(|e| {
                anyhow::anyhow!(
                    "[{}::loop_once] SubDevice with index {} failed to copy outputs\n{:?}",
                    module_path!(),
                    i,
                    e
                )
            })?;
        }
    }
    Ok(())
}

pub fn execute_machines(machines: &mut Vec<Box<dyn Machine>>) {
    let now = Instant::now();
    for machine in machines.iter_mut() {
        machine.act(now);
    }
}
// No more logging in loop_once
pub fn loop_once<'maindevice>(inputs: &mut RtLoopInputs<'_>) -> Result<(), anyhow::Error> {
    let loop_once_start = std::time::Instant::now();
    if inputs.ethercat_setup.is_some() && inputs.ethercat_perf_metrics.is_some() {
        inputs
            .ethercat_perf_metrics
            .as_deref_mut()
            .unwrap()
            .cycle_start();

        let res = smol::block_on(copy_ethercat_inputs(inputs.ethercat_setup.as_deref()));
        match res {
            Ok(success) => {
                if success {
                    inputs.consecutive_txrx_failures = 0;
                } else {
                    inputs.consecutive_txrx_failures = inputs.consecutive_txrx_failures.saturating_add(1);
                }
            }
            Err(e) => {
                return Err(anyhow::anyhow!("copy_ethercat_inputs failed: {:?}", e));
            }
        };

        // Auto-restart on persistent EtherCAT failures
        if inputs.consecutive_txrx_failures >= 20 {
            tracing::error!(
                "EtherCAT connection lost after {} consecutive failures. Restarting server...",
                inputs.consecutive_txrx_failures
            );
            // Exit with code 2 to signal EtherCAT connection loss
            // systemd or the startup script will automatically restart the server
            std::process::exit(2);
        }
    }

    execute_machines(&mut inputs.machines);


    if inputs.ethercat_setup.is_some() && inputs.ethercat_perf_metrics.is_some() {
        let res = smol::block_on(copy_ethercat_outputs(inputs.ethercat_setup.as_deref()));
        match res {
            Ok(_) => (),
            Err(e) => {
                return Err(anyhow::anyhow!("copy_ethercat_outputs failed: {:?}", e));
            }
        };
    }

    if inputs.ethercat_setup.is_some() {
        // spin_sleep so we have a cycle time of ~300us
        // This does push usage to 100% if completely busy, but provides much better accuracy then thread sleep or async sleep
        inputs
            .sleeper
            .sleep_until(loop_once_start + inputs.cycle_target);
    } else {
        // if we dont have an ethercat setup or other rt relevant stuff do the "worse" async sleep or later if we get rid of async thread::sleep or yielding
        // We do this, so that when no rt relevant code runs the cpu doesnt spin at 100% for no reason
        let loop_duration = loop_once_start.elapsed();
        if inputs.cycle_target > loop_once_start.elapsed() {
            smol::block_on(smol::Timer::after(inputs.cycle_target - loop_duration));
        }
    }

    Ok(())
}
