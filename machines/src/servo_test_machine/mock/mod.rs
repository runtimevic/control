use crate::AsyncThreadMessage;
use crate::{
    Machine, MachineMessage, SERVO_TEST_MACHINE, VENDOR_QITECH,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};
use crate::servo_test_machine::api::{DriveStateEvent, ServoTestMachineEvents, ServoTestMachineNamespace};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use smol::channel::{Receiver, Sender};
use std::time::Instant;

pub mod act;
pub mod api;
pub mod new;

/// Mock version of ServoTestMachine that doesn't require physical hardware
#[derive(Debug)]
pub struct ServoTestMachineMock {
    machine_identification_unique: MachineIdentificationUnique,
    main_sender: Option<Sender<AsyncThreadMessage>>,
    
    // API communication
    api_sender: Sender<MachineMessage>,
    api_receiver: Receiver<MachineMessage>,
    
    // SocketIO
    namespace: ServoTestMachineNamespace,
    last_state_emit: Instant,
    
    // Simulated servo state
    position: f32,
    target_position: f32,
    velocity: f32,
    target_velocity: f32,
    error_code: u16,
    
    // CiA402 state flags
    ready: bool,
    switched_on: bool,
    operation_enabled: bool,
    fault: bool,
    target_reached: bool,
    
    // Control parameters
    kv_factor: f32,
    ref_velocity: f32,
    override_value: u16,
    
    // Simulation timing
    t_0: Instant,
}

impl Machine for ServoTestMachineMock {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }
    
    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl ServoTestMachineMock {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: SERVO_TEST_MACHINE,
    };
    
    pub fn emit_state(&mut self) {
        let event = DriveStateEvent {
            position: self.position,
            setpoint_position: self.target_position,
            velocity: self.velocity,
            error_code: self.error_code,
            lag_error_current: self.target_position - self.position,
            lag_error_min: -10.0,
            lag_error_max: 10.0,
            actual_velocity: self.velocity,
            setpoint_velocity: self.target_velocity,
            override_value: self.override_value,
            output_percent: if self.operation_enabled { 50.0 } else { 0.0 },
            controller_output_percent: if self.operation_enabled { 50.0 } else { 0.0 },
            ready: self.ready,
            calibrated: !self.fault,
            has_job: self.operation_enabled,
            not_moving: self.velocity.abs() < 0.1,
            moving_forward: self.velocity > 0.1,
            moving_backward: self.velocity < -0.1,
            coupled_mode: self.operation_enabled,
            in_target_pos: self.target_reached,
            in_pos_range: self.target_reached,
            controller_enabled: self.operation_enabled,
            feed_fw_enabled: true,
            feed_bw_enabled: true,
        }
        .build();
        
        self.namespace.emit(ServoTestMachineEvents::DriveState(event));
    }
    
    /// Simulate servo movement towards target position
    pub fn simulate_motion(&mut self, dt: f32) {
        if !self.operation_enabled {
            self.velocity = 0.0;
            return;
        }
        
        // Simple position control simulation
        let position_error = self.target_position - self.position;
        let max_velocity = self.ref_velocity * (self.override_value as f32 / 10000.0);
        
        // Proportional velocity control
        self.target_velocity = (position_error * self.kv_factor).clamp(-max_velocity, max_velocity);
        
        // Simple velocity ramp (acceleration limit)
        let velocity_error = self.target_velocity - self.velocity;
        let max_accel = 1000.0; // units/s²
        let velocity_change = velocity_error.clamp(-max_accel * dt, max_accel * dt);
        self.velocity += velocity_change;
        
        // Update position
        self.position += self.velocity * dt;
        
        // Check if target reached
        self.target_reached = position_error.abs() < 0.5 && self.velocity.abs() < 0.1;
    }
}
