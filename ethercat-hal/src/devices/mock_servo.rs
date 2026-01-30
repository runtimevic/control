//! Simulador simple de servo para testing sin hardware
//!
//! Este simulador proporciona una implementación básica de ServoDevice
//! que simula movimiento real para pruebas.

use crate::devices::adapters::ServoDevice;
use anyhow::Result;

/// Simulador simple de servo CiA402
#[derive(Debug, Clone)]
pub struct MockServo {
    // Estado CiA402
    status_word: u16,
    control_word: u16,
    
    // Posición y velocidad
    position: i32,
    velocity: i32,
    torque: i16,
    
    // Consignas
    target_position: i32,
    target_velocity: i32,
    target_torque: i16,
    
    // Modo de operación
    mode: i8,  // 1=PP, 3=PV, 8=CSP, 9=CSV
}

impl MockServo {
    pub fn new() -> Self {
        Self {
            status_word: 0x0650, // Ready to switch on
            control_word: 0,
            position: 0,
            velocity: 0,
            torque: 0,
            target_position: 0,
            target_velocity: 0,
            target_torque: 0,
            mode: 0,
        }
    }
}

impl Default for MockServo {
    fn default() -> Self {
        Self::new()
    }
}

impl ServoDevice for MockServo {
    fn process_control_word(&mut self, control_word: u16) -> Result<()> {
        self.control_word = control_word;
        
        // Implementación simplificada de máquina de estados CiA402
        match control_word & 0x00FF {
            0x06 if control_word & 0x80 != 0 => self.status_word = 0x0650, // Quick stop
            0x06 => self.status_word = 0x0631, // Shutdown -> Ready to switch on
            0x07 => self.status_word = 0x0633, // Switch on -> Switched on
            0x0F => self.status_word = 0x0637, // Enable operation -> Operation enabled
            _ => {}
        }
        
        Ok(())
    }
    
    fn set_target_position(&mut self, position: i32) -> Result<()> {
        self.target_position = position;
        Ok(())
    }
    
    fn set_target_velocity(&mut self, velocity: i32) -> Result<()> {
        self.target_velocity = velocity;
        Ok(())
    }
    
    fn set_target_torque(&mut self, torque: i16) -> Result<()> {
        self.target_torque = torque;
        Ok(())
    }
    
    fn set_mode_of_operation(&mut self, mode: i8) -> Result<()> {
        self.mode = mode;
        Ok(())
    }
    
    fn get_status_word(&self) -> Result<u16> {
        Ok(self.status_word)
    }
    
    fn get_position_actual(&self) -> Result<i32> {
        Ok(self.position)
    }
    
    fn get_velocity_actual(&self) -> Result<i32> {
        Ok(self.velocity)
    }
    
    fn get_torque_actual(&self) -> Result<i16> {
        Ok(self.torque)
    }
    
    fn get_mode_of_operation_display(&self) -> Result<i8> {
        Ok(self.mode)
    }
    
    fn get_error_code(&self) -> Result<u16> {
        Ok(0)
    }
    
    fn set_profile_velocity(&mut self, _velocity: u32) -> Result<()> {
        // MockServo no implementa profile velocity
        Ok(())
    }
    
    fn get_profile_velocity(&self) -> Result<u32> {
        Ok(3000)
    }
    
    fn update(&mut self, delta_ms: u64) -> Result<()> {
        // Solo actualizar si está habilitado (operation enabled)
        if self.status_word & 0x0004 == 0 {
            return Ok(());
        }
        
        match self.mode {
            3 => {
                // Profile Velocity Mode: actualizar posición según velocidad objetivo
                self.velocity = self.target_velocity;
                // Calcular incremento de posición (velocity es en increments/second)
                let delta_seconds = delta_ms as f64 / 1000.0;
                let position_increment = (self.velocity as f64 * delta_seconds) as i32;
                self.position = self.position.wrapping_add(position_increment);
            }
            1 => {
                // Profile Position Mode: mover hacia posición objetivo
                let error = self.target_position - self.position;
                if error != 0 {
                    let max_velocity = 10000; // Velocidad máxima
                    let velocity = error.signum() * max_velocity.min(error.abs());
                    self.velocity = velocity;
                    let delta_seconds = delta_ms as f64 / 1000.0;
                    let position_increment = (velocity as f64 * delta_seconds) as i32;
                    self.position = self.position.wrapping_add(position_increment);
                    
                    // Si estamos muy cerca, ajustar directamente
                    if error.abs() < max_velocity / 10 {
                        self.position = self.target_position;
                        self.velocity = 0;
                    }
                } else {
                    self.velocity = 0;
                }
            }
            _ => {
                // Otros modos no implementados: dejar velocidad en 0
                self.velocity = 0;
            }
        }
        
        Ok(())
    }
    
    fn device_name(&self) -> &str {
        "Mock Servo (Simulator)"
    }
}
