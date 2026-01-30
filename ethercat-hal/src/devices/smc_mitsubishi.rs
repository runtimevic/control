//! Wrapper para servos SMC-Mitsubishi MR-J4-TM
//!
//! Re-exporta los tipos de ethercat-devices adaptados al framework de control.
//! 
//! Los servos Mitsubishi se pueden usar en dos modos:
//! - Simulación: Para desarrollo sin hardware
//! - Hardware: Para control del dispositivo físico

use crate::devices::adapters::ServoDevice;
use anyhow::Result;

// Re-exportar tipos principales de ethercat-devices con renombre interno
pub use ethercat_devices::smc_mitsubishi::{
    SmcMitsubishiSimulator as SmcMitsubishiSimulatorInner,
    SmcMitsubishiHardware as SmcMitsubishiHardwareInner,
};

pub use ethercat_devices::common::Cia402State;

/// Wrapper para SmcMitsubishiSimulator que implementa ServoDevice y Default
pub struct SmcMitsubishiSimulator(SmcMitsubishiSimulatorInner);

impl std::fmt::Debug for SmcMitsubishiSimulator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SmcMitsubishiSimulator").finish()
    }
}

impl SmcMitsubishiSimulator {
    pub fn new() -> Self {
        Self(SmcMitsubishiSimulatorInner::new())
    }
    
    pub fn inner(&self) -> &SmcMitsubishiSimulatorInner {
        &self.0
    }
    
    pub fn inner_mut(&mut self) -> &mut SmcMitsubishiSimulatorInner {
        &mut self.0
    }
}

/// Wrapper para SmcMitsubishiHardware que implementa ServoDevice y Default
pub struct SmcMitsubishiHardware(SmcMitsubishiHardwareInner);

impl std::fmt::Debug for SmcMitsubishiHardware {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SmcMitsubishiHardware").finish()
    }
}

impl SmcMitsubishiHardware {
    pub fn new() -> Self {
        Self(SmcMitsubishiHardwareInner::new())
    }
    
    pub fn inner(&self) -> &SmcMitsubishiHardwareInner {
        &self.0
    }
    
    pub fn inner_mut(&mut self) -> &mut SmcMitsubishiHardwareInner {
        &mut self.0
    }
}

/// Constantes de identificación del dispositivo Mitsubishi MR-J4-TM
/// Vendor ID: Mitsubishi Electric Corporation Nagoya Works
pub const SMC_MITSUBISHI_VENDOR_ID: u32 = 0x00000A1E;
/// Product Code: MR-J4-TM
pub const SMC_MITSUBISHI_PRODUCT_CODE: u32 = 0x00000201;

/// Verifica si un dispositivo EtherCAT es un Mitsubishi MR-J4-TM
///
/// # Argumentos
///
/// * `vendor_id` - ID del fabricante del dispositivo EtherCAT
/// * `product_code` - Código de producto del dispositivo EtherCAT
///
/// # Retorna
///
/// `true` si el dispositivo coincide con un Mitsubishi MR-J4-TM
///
/// # Ejemplo
///
/// ```
/// use ethercat_hal::devices::smc_mitsubishi::is_smc_mitsubishi;
///
/// let vendor_id = 0x00000A1E;
/// let product_code = 0x00000201;
///
/// if is_smc_mitsubishi(vendor_id, product_code) {
///     println!("Dispositivo Mitsubishi MR-J4-TM detectado");
/// }
/// ```
pub fn is_smc_mitsubishi(vendor_id: u32, product_code: u32) -> bool {
    vendor_id == SMC_MITSUBISHI_VENDOR_ID && product_code == SMC_MITSUBISHI_PRODUCT_CODE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_identification() {
        assert!(is_smc_mitsubishi(SMC_MITSUBISHI_VENDOR_ID, SMC_MITSUBISHI_PRODUCT_CODE));
        assert!(!is_smc_mitsubishi(0x12345678, 0x87654321));
    }
}

// Implementación de ServoDevice para SmcMitsubishiSimulator
impl ServoDevice for SmcMitsubishiSimulator {
    fn process_control_word(&mut self, control_word: u16) -> Result<()> {
        self.0.process_control_word(control_word).map_err(|e| anyhow::anyhow!("{}", e))
    }
    
    fn set_target_position(&mut self, position: i32) -> Result<()> {
        self.0.set_target_position(position).map_err(|e| anyhow::anyhow!("{}", e))
    }
    
    fn set_target_velocity(&mut self, velocity: i32) -> Result<()> {
        self.0.set_target_velocity(velocity).map_err(|e| anyhow::anyhow!("{}", e))
    }
    
    fn set_target_torque(&mut self, torque: i16) -> Result<()> {
        self.0.set_target_torque(torque).map_err(|e| anyhow::anyhow!("{}", e))
    }
    
    fn set_mode_of_operation(&mut self, mode: i8) -> Result<()> {
        self.0.set_mode_of_operation(mode).map_err(|e| anyhow::anyhow!("{}", e))
    }
    
    fn get_status_word(&self) -> Result<u16> {
        self.0.get_status_word().map_err(|e| anyhow::anyhow!("{}", e))
    }
    
    fn get_position_actual(&self) -> Result<i32> {
        self.0.get_position_actual().map_err(|e| anyhow::anyhow!("{}", e))
    }
    
    fn get_velocity_actual(&self) -> Result<i32> {
        self.0.get_velocity_actual().map_err(|e| anyhow::anyhow!("{}", e))
    }
    
    fn get_torque_actual(&self) -> Result<i16> {
        self.0.get_torque_actual().map_err(|e| anyhow::anyhow!("{}", e))
    }
    
    fn get_mode_of_operation_display(&self) -> Result<i8> {
        self.0.get_mode_of_operation().map_err(|e| anyhow::anyhow!("{}", e))
    }
    
    fn get_error_code(&self) -> Result<u16> {
        Ok(0) // SMC Mitsubishi no tiene método get_error_code directo
    }
    
    fn update(&mut self, delta_ms: u64) -> Result<()> {
        self.0.update(delta_ms).map_err(|e| anyhow::anyhow!("{}", e))
    }
    
    fn device_name(&self) -> &str {
        "SMC-Mitsubishi MR-J4-TM (Simulator)"
    }
}

impl Default for SmcMitsubishiSimulator {
    fn default() -> Self {
        Self::new()
    }
}

// Implementación de ServoDevice para SmcMitsubishiHardware
impl ServoDevice for SmcMitsubishiHardware {
    fn process_control_word(&mut self, _control_word: u16) -> Result<()> {
        Ok(())
    }
    
    fn set_target_position(&mut self, _position: i32) -> Result<()> {
        Ok(())
    }
    
    fn set_target_velocity(&mut self, _velocity: i32) -> Result<()> {
        Ok(())
    }
    
    fn set_target_torque(&mut self, _torque: i16) -> Result<()> {
        Ok(())
    }
    
    fn set_mode_of_operation(&mut self, _mode: i8) -> Result<()> {
        Ok(())
    }
    
    fn get_status_word(&self) -> Result<u16> {
        Ok(0)
    }
    
    fn get_position_actual(&self) -> Result<i32> {
        Ok(0)
    }
    
    fn get_velocity_actual(&self) -> Result<i32> {
        Ok(0)
    }
    
    fn get_torque_actual(&self) -> Result<i16> {
        Ok(0)
    }
    
    fn get_mode_of_operation_display(&self) -> Result<i8> {
        Ok(0)
    }
    
    fn get_error_code(&self) -> Result<u16> {
        Ok(0)
    }
    
    fn update(&mut self, _delta_ms: u64) -> Result<()> {
        Ok(())
    }
    
    fn device_name(&self) -> &str {
        "SMC-Mitsubishi MR-J4-TM (Hardware)"
    }
}

impl Default for SmcMitsubishiHardware {
    fn default() -> Self {
        Self::new()
    }
}

