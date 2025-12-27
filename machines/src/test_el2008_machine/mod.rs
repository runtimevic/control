use crate::machine_identification::{MachineIdentification, MachineIdentificationUnique};
use crate::test_el2008_machine::api::{StateEvent, TestEL2008MachineEvents};
use crate::{AsyncThreadMessage, Machine, MachineMessage};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::digital_output::DigitalOutput;
use serde::{Deserialize, Serialize};
use smol::channel::{Receiver, Sender};
use std::time::{Duration, Instant};
pub mod act;
pub mod api;
pub mod new;
use crate::test_el2008_machine::api::TestEL2008MachineNamespace;
use crate::{TEST_EL2008_MACHINE, VENDOR_QITECH};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum TestEL2008Mode {
    Manual,
    Home,
    Automatic,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum MachineState {
    Stopped,
    Running,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutomaticPhase {
    TurningOn(usize),  // Index of current output being turned on
    TurningOff(usize), // Index of current output being turned off
    Idle,              // All outputs off, waiting to restart
}

#[derive(Debug)]
pub struct TestEL2008Machine {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: TestEL2008MachineNamespace,
    pub last_state_emit: Instant,
    pub led_on: [bool; 8],
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub douts: [DigitalOutput; 8],
    
    // Mode and state
    pub mode: TestEL2008Mode,
    pub machine_state: MachineState,
    
    // Automatic mode fields
    pub automatic_phase: AutomaticPhase,
    pub automatic_delay_ms: u64,
    pub last_automatic_step: Instant,
}

impl Machine for TestEL2008Machine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}
impl TestEL2008Machine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: TEST_EL2008_MACHINE,
    };
}

impl TestEL2008Machine {
    pub fn emit_state(&mut self) {
        let event = StateEvent {
            led_on: self.led_on,
            mode: self.mode.clone(),
            machine_state: self.machine_state.clone(),
            automatic_delay_ms: self.automatic_delay_ms,
        }
        .build();

        self.namespace.emit(TestEL2008MachineEvents::State(event));
    }

    /// Set the state of a specific LED (only in manual mode and running)
    pub fn set_led(&mut self, index: usize, on: bool) {
        if self.mode == TestEL2008Mode::Manual 
            && self.machine_state == MachineState::Running 
            && index < self.led_on.len() 
        {
            self.led_on[index] = on;
            self.emit_state();
        }
    }

    /// Set all LEDs at once (only in manual mode and running)
    pub fn set_all_leds(&mut self, on: bool) {
        if self.mode == TestEL2008Mode::Manual && self.machine_state == MachineState::Running {
            self.led_on = [on; 8];
            self.emit_state();
        }
    }

    /// Start the machine based on current mode
    pub fn start(&mut self) {
        match self.mode {
            TestEL2008Mode::Manual => {
                self.machine_state = MachineState::Running;
            }
            TestEL2008Mode::Home => {
                self.machine_state = MachineState::Running;
                self.go_home();
            }
            TestEL2008Mode::Automatic => {
                self.machine_state = MachineState::Running;
                self.automatic_phase = AutomaticPhase::TurningOn(0);
                self.last_automatic_step = Instant::now();
            }
        }
        self.emit_state();
    }

    /// Stop the machine
    pub fn stop(&mut self) {
        self.machine_state = MachineState::Stopped;
        if self.mode == TestEL2008Mode::Automatic {
            self.automatic_phase = AutomaticPhase::Idle;
        }
        self.emit_state();
    }

    /// Reset the machine
    pub fn reset(&mut self) {
        self.machine_state = MachineState::Stopped;
        self.led_on = [false; 8];
        self.automatic_phase = AutomaticPhase::Idle;
        self.apply_outputs();
        self.emit_state();
    }

    /// Go to home position
    fn go_home(&mut self) {
        self.led_on = [false; 8];
        self.apply_outputs();
    }

    /// Change mode
    pub fn set_mode(&mut self, mode: TestEL2008Mode) {
        if self.machine_state == MachineState::Stopped {
            self.mode = mode;
            self.automatic_phase = AutomaticPhase::Idle;
            self.emit_state();
        }
    }

    /// Set automatic delay in milliseconds
    pub fn set_automatic_delay(&mut self, delay_ms: u64) {
        self.automatic_delay_ms = delay_ms;
        self.emit_state();
    }

    /// Apply current LED state to physical outputs
    pub fn apply_outputs(&mut self) {
        for (led, &on) in self.douts.iter().zip(self.led_on.iter()) {
            led.set(on);
        }
    }

    /// Execute automatic mode logic
    pub fn execute_automatic(&mut self, now: Instant) {
        if self.mode != TestEL2008Mode::Automatic || self.machine_state != MachineState::Running {
            return;
        }

        let elapsed = now.duration_since(self.last_automatic_step);
        let delay = Duration::from_millis(self.automatic_delay_ms);

        if elapsed < delay {
            return;
        }

        self.last_automatic_step = now;

        match self.automatic_phase {
            AutomaticPhase::TurningOn(index) => {
                if index < 8 {
                    self.led_on[index] = true;
                    if index == 7 {
                        // Reached the last output, start turning off
                        self.automatic_phase = AutomaticPhase::TurningOff(7);
                    } else {
                        // Move to next output
                        self.automatic_phase = AutomaticPhase::TurningOn(index + 1);
                    }
                }
            }
            AutomaticPhase::TurningOff(index) => {
                self.led_on[index] = false;
                if index == 0 {
                    // All outputs are off, go to idle
                    self.automatic_phase = AutomaticPhase::Idle;
                } else {
                    // Move to previous output
                    self.automatic_phase = AutomaticPhase::TurningOff(index - 1);
                }
            }
            AutomaticPhase::Idle => {
                // All outputs off, restart the cycle
                self.automatic_phase = AutomaticPhase::TurningOn(0);
            }
        }

        self.apply_outputs();
    }
}