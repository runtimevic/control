//! Wrapper para servos Lichuan LC10E
//!
//! Re-exporta los tipos de ethercat-devices adaptados al framework de control.
//! 
//! Los servos Lichuan se pueden usar en dos modos:
//! - Simulación: Para desarrollo sin hardware
//! - Hardware: Para control del dispositivo físico

use crate::devices::adapters::ServoDevice;
use anyhow::Result;

// Re-exportar tipos principales de ethercat-devices con renombre interno
pub use ethercat_devices::lichuan::{
    LichuanSimulator as LichuanSimulatorInner,
    LichuanHardware as LichuanHardwareInner,
    Cia402State,
};

/// Wrapper para LichuanSimulator que implementa ServoDevice y Default
pub struct LichuanSimulator(LichuanSimulatorInner);

impl std::fmt::Debug for LichuanSimulator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LichuanSimulator").finish()
    }
}

impl LichuanSimulator {
    pub fn new() -> Self {
        Self(LichuanSimulatorInner::new())
    }
    
    pub fn inner(&self) -> &LichuanSimulatorInner {
        &self.0
    }
    
    pub fn inner_mut(&mut self) -> &mut LichuanSimulatorInner {
        &mut self.0
    }
}

/// Wrapper para LichuanHardware que implementa ServoDevice y Default
pub struct LichuanHardware(LichuanHardwareInner);

impl std::fmt::Debug for LichuanHardware {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LichuanHardware").finish()
    }
}

impl LichuanHardware {
    pub fn new() -> Self {
        Self(LichuanHardwareInner::new())
    }
    
    pub fn inner(&self) -> &LichuanHardwareInner {
        &self.0
    }
    
    pub fn inner_mut(&mut self) -> &mut LichuanHardwareInner {
        &mut self.0
    }
}

/// Constantes de identificación del dispositivo Lichuan LC10E
pub const LICHUAN_LC10E_VENDOR_ID: u32 = 0x00000766;
pub const LICHUAN_LC10E_PRODUCT_CODE: u32 = 0x00000402;

/// Identity tuple para Lichuan LC10E (vendor_id, product_code, revision)
pub const LICHUAN_LC10E_IDENTITY: crate::devices::SubDeviceIdentityTuple = (
    LICHUAN_LC10E_VENDOR_ID,
    LICHUAN_LC10E_PRODUCT_CODE,
    0x00000000, // revision number
);

/// Verifica si un dispositivo es un Lichuan LC10E basándose en Vendor ID y Product Code
///
/// # Argumentos
/// * `vendor_id` - ID del fabricante del dispositivo
/// * `product_code` - Código de producto del dispositivo
///
/// # Retorna
/// `true` si el dispositivo coincide con un Lichuan LC10E
pub fn is_lichuan_lc10e(vendor_id: u32, product_code: u32) -> bool {
    vendor_id == LICHUAN_LC10E_VENDOR_ID && product_code == LICHUAN_LC10E_PRODUCT_CODE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_identification() {
        assert!(is_lichuan_lc10e(LICHUAN_LC10E_VENDOR_ID, LICHUAN_LC10E_PRODUCT_CODE));
        assert!(!is_lichuan_lc10e(0x0000, 0x0000));
    }

    #[test]
    fn test_simulator_creation() {
        let _sim = LichuanSimulator::new();
        // El simulador debe crearse sin errores
    }
}

// Implementación de ServoDevice para LichuanSimulator
impl ServoDevice for LichuanSimulator {
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
        self.0.get_mode_display().map_err(|e| anyhow::anyhow!("{}", e))
    }
    
    fn get_error_code(&self) -> Result<u16> {
        Ok(0) // Lichuan no tiene método get_error_code directo
    }
    
    fn set_profile_velocity(&mut self, velocity: u32) -> Result<()> {
        self.0.set_profile_velocity(velocity).map_err(|e| anyhow::anyhow!("{}", e))
    }
    
    fn get_profile_velocity(&self) -> Result<u32> {
        self.0.get_profile_velocity().map_err(|e| anyhow::anyhow!("{}", e))
    }
    
    fn update(&mut self, delta_ms: u64) -> Result<()> {
        self.0.update(delta_ms).map_err(|e| anyhow::anyhow!("{}", e))
    }
    
    fn device_name(&self) -> &str {
        "Lichuan LC10E (Simulator)"
    }
}

impl Default for LichuanSimulator {
    fn default() -> Self {
        Self::new()
    }
}

// Implementación de ServoDevice para LichuanHardware
impl ServoDevice for LichuanHardware {
    fn process_control_word(&mut self, _control_word: u16) -> Result<()> {
        // Hardware requiere comunicación EtherCAT real
        // Por ahora, stub
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
    
    fn set_profile_velocity(&mut self, _velocity: u32) -> Result<()> {
        // Hardware: se configurar\u00eda v\u00eda CoE SDO 0x6081
        Ok(())
    }
    
    fn get_profile_velocity(&self) -> Result<u32> {
        // Hardware: se leer\u00eda v\u00eda CoE SDO 0x6081
        Ok(3000) // Valor por defecto
    }
    
    fn update(&mut self, _delta_ms: u64) -> Result<()> {
        // Hardware no necesita update simulado
        Ok(())
    }
    
    fn device_name(&self) -> &str {
        "Lichuan LC10E (Hardware)"
    }
}

impl Default for LichuanHardware {
    fn default() -> Self {
        Self::new()
    }
}

