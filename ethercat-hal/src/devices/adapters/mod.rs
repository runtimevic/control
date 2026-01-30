//! Adaptadores para integrar rust-ethercat-devices en el framework de control
//!
//! Este módulo proporciona:
//! - Trait `ServoDevice` que abstrae servos CiA402
//! - Wrapper `ServoAdapter` que implementa `EthercatDevice`
//! - Conversiones entre BitSlice (PDO) y métodos del servo

use anyhow::Result;
use std::fmt::Debug;

pub mod pdo_mapping;
pub mod servo_adapter;

pub use servo_adapter::ServoAdapter;

/// Trait que abstrae las operaciones comunes de un servo CiA402
///
/// Este trait permite que tanto los simuladores como el hardware real
/// se integren de forma uniforme en el framework de control.
pub trait ServoDevice: Send + Sync {
    /// Procesar el control word del RxPDO
    fn process_control_word(&mut self, control_word: u16) -> Result<()>;
    
    /// Establecer la posición objetivo (para modo CSP)
    fn set_target_position(&mut self, position: i32) -> Result<()>;
    
    /// Establecer la velocidad objetivo (para modo CSV)
    fn set_target_velocity(&mut self, velocity: i32) -> Result<()>;
    
    /// Establecer el torque objetivo (para modo CST)
    fn set_target_torque(&mut self, torque: i16) -> Result<()>;
    
    /// Establecer el modo de operación
    fn set_mode_of_operation(&mut self, mode: i8) -> Result<()>;
    
    /// Obtener el status word para el TxPDO
    fn get_status_word(&self) -> Result<u16>;
    
    /// Obtener la posición actual
    fn get_position_actual(&self) -> Result<i32>;
    
    /// Obtener la velocidad actual
    fn get_velocity_actual(&self) -> Result<i32>;
    
    /// Obtener el torque actual
    fn get_torque_actual(&self) -> Result<i16>;
    
    /// Obtener el modo de operación actual
    fn get_mode_of_operation_display(&self) -> Result<i8>;
    
    /// Obtener el código de error
    fn get_error_code(&self) -> Result<u16>;
    
    /// Establecer la velocidad de perfil (0x6081) para movimientos CSP
    fn set_profile_velocity(&mut self, velocity: u32) -> Result<()>;
    
    /// Obtener la velocidad de perfil configurada (0x6081)
    fn get_profile_velocity(&self) -> Result<u32>;
    
    /// Actualizar la simulación (solo para simuladores, no-op para hardware)
    fn update(&mut self, delta_ms: u64) -> Result<()>;
    
    /// Nombre del dispositivo para logging
    fn device_name(&self) -> &str;
}

/// Estructura de RxPDO estándar CiA402 (Master -> Servo)
///
/// Esta estructura mapea los datos que se envían al servo en cada ciclo.
/// El tamaño y campos exactos pueden variar según el modo de operación,
/// pero esta es la configuración más común para CSP (Cyclic Synchronous Position).
#[derive(Debug, Clone, Copy, Default)]
pub struct Cia402RxPdo {
    /// Control word (0x6040)
    pub control_word: u16,
    
    /// Target position (0x607A) - usado en modo CSP
    pub target_position: i32,
    
    /// Target velocity (0x60FF) - usado en modo CSV
    pub target_velocity: i32,
    
    /// Target torque (0x6071) - usado en modo CST
    pub target_torque: i16,
    
    /// Mode of operation (0x6060)
    pub mode_of_operation: i8,
}

/// Estructura de TxPDO estándar CiA402 (Servo -> Master)
///
/// Esta estructura mapea los datos que el servo envía al master en cada ciclo.
#[derive(Debug, Clone, Copy, Default)]
pub struct Cia402TxPdo {
    /// Status word (0x6041)
    pub status_word: u16,
    
    /// Position actual value (0x6064)
    pub position_actual: i32,
    
    /// Velocity actual value (0x606C)
    pub velocity_actual: i32,
    
    /// Torque actual value (0x6077)
    pub torque_actual: i16,
    
    /// Mode of operation display (0x6061)
    pub mode_of_operation_display: i8,
    
    /// Error code (0x603F)
    pub error_code: u16,
}

impl Cia402RxPdo {
    /// Tamaño en bytes del RxPDO estándar
    pub const SIZE_BYTES: usize = 13; // 2 + 4 + 4 + 2 + 1
    
    /// Tamaño en bits del RxPDO estándar
    pub const SIZE_BITS: usize = Self::SIZE_BYTES * 8;
}

impl Cia402TxPdo {
    /// Tamaño en bytes del TxPDO estándar
    pub const SIZE_BYTES: usize = 17; // 2 + 4 + 4 + 2 + 1 + 2 + 2 (con padding)
    
    /// Tamaño en bits del TxPDO estándar
    pub const SIZE_BITS: usize = Self::SIZE_BYTES * 8;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdo_sizes() {
        assert_eq!(Cia402RxPdo::SIZE_BYTES, 13);
        assert_eq!(Cia402TxPdo::SIZE_BYTES, 17);
    }
}
