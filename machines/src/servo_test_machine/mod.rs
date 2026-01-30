use crate::machine_identification::{MachineIdentification, MachineIdentificationUnique};
use crate::servo_test_machine::api::{DriveStateEvent, ServoTestMachineEvents};
use crate::{AsyncThreadMessage, Machine, MachineMessage};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::devices::adapters::ServoAdapter;
use ethercat_hal::devices::servo::ServoDevice;
use smol::channel::{Receiver, Sender};
use smol::lock::RwLock;
use std::sync::Arc;
use std::time::Instant;

pub mod act;
pub mod api;
pub mod new;

use crate::servo_test_machine::api::ServoTestMachineNamespace;
use crate::{VENDOR_QITECH, SERVO_TEST_MACHINE};

#[derive(Debug)]
pub struct ServoTestMachine<T: ServoDevice> {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: ServoTestMachineNamespace,
    pub last_state_emit: Instant,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub servo: Arc<RwLock<ServoAdapter<T>>>,
}

impl<T: ServoDevice> Machine for ServoTestMachine<T> {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl<T: ServoDevice> ServoTestMachine<T> {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: SERVO_TEST_MACHINE,
    };
}

impl<T: ServoDevice> ServoTestMachine<T> {
    pub async fn emit_state(&mut self) {
        // Get current state from servo (async lock)
        let servo = self.servo.read().await;
        
        let position = servo.servo().get_position_actual().unwrap_or(0.0);
        let setpoint_position = servo.servo().get_position_setpoint().unwrap_or(0.0);
        let velocity = servo.servo().get_velocity().unwrap_or(0.0);
        let error_code = servo.servo().get_error_code().unwrap_or(0);
        let lag_error = servo.servo().get_lag_error().unwrap_or(0.0);
        
        let statusword = servo.servo().get_statusword().unwrap_or(0);
        
        drop(servo); // Release lock
        
        // Parse CiA402 statusword flags
        let ready = statusword & 0x0001 != 0;
        let switched_on = statusword & 0x0002 != 0;
        let operation_enabled = statusword & 0x0004 != 0;
        let fault = statusword & 0x0008 != 0;
        let target_reached = statusword & 0x0400 != 0;
        
        let event = DriveStateEvent {
            position,
            setpoint_position,
            velocity,
            error_code,
            lag_error_current: lag_error,
            lag_error_min: lag_error,
            lag_error_max: lag_error,
            actual_velocity: velocity,
            setpoint_velocity: 0.0,
            override_value: 10000,
            output_percent: 0.0,
            controller_output_percent: 0.0,
            ready,
            calibrated: !fault,
            has_job: operation_enabled,
            not_moving: velocity.abs() < 0.1,
            moving_forward: velocity > 0.1,
            moving_backward: velocity < -0.1,
            coupled_mode: operation_enabled,
            in_target_pos: target_reached,
            in_pos_range: target_reached,
            controller_enabled: operation_enabled,
            feed_fw_enabled: true,
            feed_bw_enabled: true,
        }
        .build();

        self.namespace.emit(ServoTestMachineEvents::DriveState(event));
    }
}
