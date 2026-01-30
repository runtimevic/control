//! Adaptador que convierte servos de rust-ethercat-devices en EthercatDevice
//!
//! Este módulo proporciona `ServoAdapter<T>` que wrappea cualquier implementación
//! de `ServoDevice` y la expone como un `EthercatDevice` compatible con el framework.

use super::{ServoDevice, Cia402RxPdo, Cia402TxPdo, pdo_mapping};
use crate::devices::{EthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed, NewEthercatDevice, Module};
use anyhow::Result;
use bitvec::{order::Lsb0, slice::BitSlice};
use std::any::Any;
use std::fmt::Debug;

/// Adaptador genérico que convierte un ServoDevice en EthercatDevice
///
/// # Genéricos
///
/// * `T` - Tipo que implementa `ServoDevice` (e.g., LichuanSimulator, SmcMitsubishiSimulator)
///
/// # Ejemplo
///
/// ```no_run
/// use ethercat_hal::devices::adapters::ServoAdapter;
/// use ethercat_hal::devices::LichuanSimulator;
///
/// let adapter = ServoAdapter::new(LichuanSimulator::new());
/// // adapter ahora puede usarse como EthercatDevice
/// ```
pub struct ServoAdapter<T: ServoDevice> {
    /// El servo subyacente (simulador o hardware)
    servo: T,
    
    /// Último RxPDO recibido
    rx_pdo: Cia402RxPdo,
    
    /// Último TxPDO a enviar
    tx_pdo: Cia402TxPdo,
    
    /// Información del módulo EtherCAT
    module: Option<Module>,
    
    /// Marca si el dispositivo está siendo usado
    used: bool,
}

impl<T: ServoDevice> ServoAdapter<T> {
    /// Crea un nuevo adaptador wrapeando el servo dado
    pub fn new(servo: T) -> Self {
        Self {
            servo,
            rx_pdo: Cia402RxPdo::default(),
            tx_pdo: Cia402TxPdo::default(),
            module: None,
            used: false,
        }
    }
    
    /// Obtiene una referencia al servo subyacente
    pub fn servo(&self) -> &T {
        &self.servo
    }
    
    /// Obtiene una referencia mutable al servo subyacente
    pub fn servo_mut(&mut self) -> &mut T {
        &mut self.servo
    }
    
    /// Establece la velocidad de perfil (0x6081) para movimientos CSP
    pub fn set_profile_velocity(&mut self, velocity: u32) -> Result<()> {
        self.servo.set_profile_velocity(velocity)
    }
    
    /// Obtiene la velocidad de perfil configurada (0x6081)
    pub fn get_profile_velocity(&self) -> Result<u32> {
        self.servo.get_profile_velocity()
    }
    
    /// Procesa el RxPDO aplicando los comandos al servo
    fn process_rx_pdo(&mut self) -> Result<()> {
        self.servo.process_control_word(self.rx_pdo.control_word)?;
        self.servo.set_target_position(self.rx_pdo.target_position)?;
        self.servo.set_target_velocity(self.rx_pdo.target_velocity)?;
        self.servo.set_target_torque(self.rx_pdo.target_torque)?;
        self.servo.set_mode_of_operation(self.rx_pdo.mode_of_operation)?;
        Ok(())
    }
    
    /// Lee el estado del servo y lo actualiza en el TxPDO
    fn update_tx_pdo(&mut self) -> Result<()> {
        self.tx_pdo.status_word = self.servo.get_status_word()?;
        self.tx_pdo.position_actual = self.servo.get_position_actual()?;
        self.tx_pdo.velocity_actual = self.servo.get_velocity_actual()?;
        self.tx_pdo.torque_actual = self.servo.get_torque_actual()?;
        self.tx_pdo.mode_of_operation_display = self.servo.get_mode_of_operation_display()?;
        self.tx_pdo.error_code = self.servo.get_error_code()?;
        Ok(())
    }
}

impl<T: ServoDevice> Debug for ServoAdapter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServoAdapter")
            .field("servo_device", &self.servo.device_name())
            .field("rx_pdo", &self.rx_pdo)
            .field("tx_pdo", &self.tx_pdo)
            .field("used", &self.used)
            .finish()
    }
}

impl<T: ServoDevice + Default> NewEthercatDevice for ServoAdapter<T> {
    fn new() -> Self {
        Self::new(T::default())
    }
}

impl<T: ServoDevice> EthercatDeviceUsed for ServoAdapter<T> {
    fn is_used(&self) -> bool {
        self.used
    }

    fn set_used(&mut self, used: bool) {
        self.used = used;
    }
}

impl<T: ServoDevice> EthercatDeviceProcessing for ServoAdapter<T> {
    fn input_post_process(&mut self) -> Result<()> {
        // Después de recibir input, procesar los comandos y actualizar simulación
        self.process_rx_pdo()?;
        
        // Actualizar simulación (1ms de ciclo típico)
        self.servo.update(1)?;
        
        // Leer nuevo estado del servo
        self.update_tx_pdo()?;
        
        Ok(())
    }

    fn output_pre_process(&mut self) -> Result<()> {
        // Antes de enviar output, asegurar que TxPDO está actualizado
        self.update_tx_pdo()
    }
}

impl<T: ServoDevice + Default + 'static> EthercatDevice for ServoAdapter<T> {
    fn input(&mut self, input: &BitSlice<u8, Lsb0>) -> Result<()> {
        // Parsear RxPDO desde BitSlice
        self.rx_pdo = pdo_mapping::read_rx_pdo(input)?;
        Ok(())
    }

    fn input_len(&self) -> usize {
        Cia402RxPdo::SIZE_BITS
    }

    fn output(&self, output: &mut BitSlice<u8, Lsb0>) -> Result<()> {
        // Escribir TxPDO a BitSlice
        pdo_mapping::write_tx_pdo(output, &self.tx_pdo)?;
        Ok(())
    }

    fn output_len(&self) -> usize {
        Cia402TxPdo::SIZE_BITS
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn is_module(&self) -> bool {
        self.module.is_some()
    }

    fn get_module(&self) -> Option<Module> {
        self.module
    }

    fn set_module(&mut self, module: Module) {
        self.module = Some(module);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock servo para tests
    #[derive(Debug, Default)]
    struct MockServo {
        control_word: u16,
        target_position: i32,
        status_word: u16,
        position_actual: i32,
    }

    impl ServoDevice for MockServo {
        fn process_control_word(&mut self, control_word: u16) -> Result<()> {
            self.control_word = control_word;
            Ok(())
        }

        fn set_target_position(&mut self, position: i32) -> Result<()> {
            self.target_position = position;
            Ok(())
        }

        fn set_target_velocity(&mut self, _velocity: i32) -> Result<()> { Ok(()) }
        fn set_target_torque(&mut self, _torque: i16) -> Result<()> { Ok(()) }
        fn set_mode_of_operation(&mut self, _mode: i8) -> Result<()> { Ok(()) }

        fn get_status_word(&self) -> Result<u16> { Ok(self.status_word) }
        fn get_position_actual(&self) -> Result<i32> { Ok(self.position_actual) }
        fn get_velocity_actual(&self) -> Result<i32> { Ok(0) }
        fn get_torque_actual(&self) -> Result<i16> { Ok(0) }
        fn get_mode_of_operation_display(&self) -> Result<i8> { Ok(0) }
        fn get_error_code(&self) -> Result<u16> { Ok(0) }
        
        fn set_profile_velocity(&mut self, _velocity: u32) -> Result<()> { Ok(()) }
        fn get_profile_velocity(&self) -> Result<u32> { Ok(3000) }

        fn update(&mut self, _delta_ms: u64) -> Result<()> {
            // Simular movimiento hacia target
            if self.position_actual < self.target_position {
                self.position_actual += 100;
            }
            Ok(())
        }

        fn device_name(&self) -> &str { "MockServo" }
    }

    #[test]
    fn test_servo_adapter_creation() {
        let servo = MockServo::default();
        let adapter = ServoAdapter::new(servo);
        
        assert!(!adapter.is_used());
        assert_eq!(adapter.input_len(), Cia402RxPdo::SIZE_BITS);
        assert_eq!(adapter.output_len(), Cia402TxPdo::SIZE_BITS);
    }

    #[test]
    fn test_servo_adapter_update() {
        let mut servo = MockServo::default();
        servo.status_word = 0x1234;
        servo.position_actual = 5000;
        
        let mut adapter = ServoAdapter::new(servo);
        adapter.update_tx_pdo().unwrap();
        
        assert_eq!(adapter.tx_pdo.status_word, 0x1234);
        assert_eq!(adapter.tx_pdo.position_actual, 5000);
    }
}
