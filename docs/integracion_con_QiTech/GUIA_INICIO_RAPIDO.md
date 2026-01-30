# 🚀 Guía de Inicio Rápido - Integración

**Documento:** Primeros pasos para integrar rust-ethercat-devices con control  
**Audiencia:** Desarrolladores que van a implementar la integración  
**Tiempo estimado:** 2-4 horas para configuración inicial

---

## 📋 Prerrequisitos

### Software necesario

```bash
# Rust toolchain (stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update

# Herramientas de desarrollo
cargo install cargo-edit    # Para gestionar dependencias
cargo install cargo-watch   # Para desarrollo iterativo
cargo install cargo-tarpaulin  # Para code coverage (opcional)

# Git
git --version  # Verificar que está instalado
```

### Conocimientos necesarios

- ✅ Rust intermedio (traits, generics, error handling)
- ✅ Conceptos básicos de EtherCAT
- ✅ Familiaridad con CiA402 (deseable)
- ✅ Git y GitHub workflow

---

## 🎯 Sprint 1: Configuración Inicial (Día 1)

### Paso 1: Clonar ambos repositorios

```bash
# Crear directorio de trabajo
mkdir ~/ethercat-integration
cd ~/ethercat-integration

# Clonar rust-ethercat-devices
git clone https://github.com/runtimevic/rust-ethercat-devices.git
cd rust-ethercat-devices
git checkout -b feature/ethercat-hal-integration

# Verificar que compila y tests pasan
cargo build
cargo test
# Debe pasar: 414 tests OK

cd ..

# Clonar control (tu fork)
git clone https://github.com/runtimevic/control.git
cd control
git checkout -b feature/device-simulation-support

# Verificar compilación
cd ethercat-hal
cargo check
```

**Checkpoint 1:** Ambos proyectos clonan y compilan sin errores.

---

### Paso 2: Agregar dependencia en control

**Archivo:** `control/ethercat-hal/Cargo.toml`

```toml
[dependencies]
# ... dependencias existentes ...
ethercrab = "0.6"
bitvec = "1.0"
# ... otras ...

# NUEVO: Agregar rust-ethercat-devices
ethercat-devices = { path = "../../../rust-ethercat-devices" }
# Alternativa si prefieres git dependency:
# ethercat-devices = { git = "https://github.com/runtimevic/rust-ethercat-devices", branch = "feature/ethercat-hal-integration" }
```

**Verificar:**
```bash
cd control/ethercat-hal
cargo check
# Debe compilar sin errores
```

**Checkpoint 2:** `ethercat-devices` se importa correctamente en `ethercat-hal`.

---

### Paso 3: Crear estructura de adaptadores

```bash
cd control/ethercat-hal/src/devices

# Crear directorio de adaptadores
mkdir adapters
touch adapters/mod.rs
touch adapters/servo_adapter.rs
touch adapters/terminal_adapter.rs
```

**Archivo:** `control/ethercat-hal/src/devices/adapters/mod.rs`

```rust
//! Adaptadores para integrar ethercat-devices con ethercat-hal

pub mod servo_adapter;
pub mod terminal_adapter;

pub use servo_adapter::ServoDeviceAdapter;
// Terminal adapter se implementará más tarde
// pub use terminal_adapter::TerminalDeviceAdapter;
```

**Archivo:** `control/ethercat-hal/src/devices/mod.rs`

Agregar al final:
```rust
// ... código existente ...

// Adaptadores para ethercat-devices
pub mod adapters;
pub use adapters::ServoDeviceAdapter;
```

**Verificar:**
```bash
cargo check
# Debe compilar (aunque adapters/ esté vacío por ahora)
```

**Checkpoint 3:** Estructura de directorios creada y módulos exportados.

---

## 🛠️ Sprint 1: Implementación Básica (Días 2-3)

### Paso 4: Implementar adaptador básico

**Archivo:** `control/ethercat-hal/src/devices/adapters/servo_adapter.rs`

```rust
//! Adaptador de Servo (ethercat-devices) → EthercatDevice (ethercat-hal)

use crate::devices::{
    EthercatDevice, NewEthercatDevice, 
    EthercatDeviceProcessing, EthercatDeviceUsed
};
use ethercat_devices::traits::servo::Servo;
use bitvec::prelude::*;
use std::any::Any;
use std::marker::PhantomData;

/// PDO de salida (Master → Servo)
#[derive(Default, Debug, Clone, Copy)]
pub struct ServoPDOOutput {
    pub control_word: u16,
    pub target_position: i32,
    pub target_velocity: i32,
    pub target_torque: i16,
}

/// PDO de entrada (Servo → Master)
#[derive(Default, Debug, Clone, Copy)]
pub struct ServoPDOInput {
    pub status_word: u16,
    pub position_actual: i32,
    pub velocity_actual: i32,
    pub torque_actual: i16,
}

/// Adaptador que permite usar cualquier Servo como EthercatDevice
pub struct ServoDeviceAdapter<T: Servo + Send + Sync + 'static> {
    servo: T,
    rxpdo: ServoPDOOutput,  // Output del master
    txpdo: ServoPDOInput,   // Input del servo
    used: bool,
}

impl<T: Servo + Send + Sync + 'static> ServoDeviceAdapter<T> {
    /// Crear nuevo adaptador
    pub fn new(servo: T) -> Self {
        Self {
            servo,
            rxpdo: ServoPDOOutput::default(),
            txpdo: ServoPDOInput::default(),
            used: false,
        }
    }
    
    /// Acceso al servo interno
    pub fn servo(&self) -> &T {
        &self.servo
    }
    
    /// Acceso mutable al servo interno
    pub fn servo_mut(&mut self) -> &mut T {
        &mut self.servo
    }
    
    /// Acceso a PDO de salida (para control de alto nivel)
    pub fn rxpdo(&self) -> &ServoPDOOutput {
        &self.rxpdo
    }
    
    /// Acceso mutable a PDO de salida
    pub fn rxpdo_mut(&mut self) -> &mut ServoPDOOutput {
        &mut self.rxpdo
    }
    
    /// Acceso a PDO de entrada (para lectura)
    pub fn txpdo(&self) -> &ServoPDOInput {
        &self.txpdo
    }
}

impl<T: Servo + Send + Sync + 'static> EthercatDevice for ServoDeviceAdapter<T> {
    fn input(&mut self, input: &BitSlice<u8, Lsb0>) -> Result<(), anyhow::Error> {
        // Leer TxPDO (servo → master)
        let mut offset = 0;
        
        // Status word (16 bits)
        self.txpdo.status_word = input[offset..offset+16].load_le::<u16>();
        offset += 16;
        
        // Position actual (32 bits)
        self.txpdo.position_actual = input[offset..offset+32].load_le::<i32>();
        offset += 32;
        
        // Velocity actual (32 bits)
        self.txpdo.velocity_actual = input[offset..offset+32].load_le::<i32>();
        offset += 32;
        
        // Torque actual (16 bits)
        self.txpdo.torque_actual = input[offset..offset+16].load_le::<i16>();
        
        Ok(())
    }
    
    fn input_len(&self) -> usize {
        // Status (16) + Position (32) + Velocity (32) + Torque (16) = 96 bits = 12 bytes
        12
    }
    
    fn output(&self, output: &mut BitSlice<u8, Lsb0>) -> Result<(), anyhow::Error> {
        // Escribir RxPDO (master → servo)
        let mut offset = 0;
        
        // Control word (16 bits)
        output[offset..offset+16].store_le(self.rxpdo.control_word);
        offset += 16;
        
        // Target position (32 bits)
        output[offset..offset+32].store_le(self.rxpdo.target_position);
        offset += 32;
        
        // Target velocity (32 bits)
        output[offset..offset+32].store_le(self.rxpdo.target_velocity);
        offset += 32;
        
        // Target torque (16 bits)
        output[offset..offset+16].store_le(self.rxpdo.target_torque);
        
        Ok(())
    }
    
    fn output_len(&self) -> usize {
        // Control (16) + Position (32) + Velocity (32) + Torque (16) = 96 bits = 12 bytes
        12
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    
    fn is_module(&self) -> bool {
        false
    }
    
    fn get_module(&self) -> Option<crate::devices::Module> {
        None
    }
    
    fn set_module(&mut self, _module: crate::devices::Module) {
        // Servos no son módulos
    }
}

impl<T: Servo + Send + Sync + 'static> NewEthercatDevice for ServoDeviceAdapter<T> {
    fn new() -> Self {
        // Este método se usa para dispositivos genéricos.
        // Para servos específicos, usar constructores con parámetros.
        unimplemented!("Use constructores específicos para cada vendor")
    }
}

impl<T: Servo + Send + Sync + 'static> EthercatDeviceProcessing for ServoDeviceAdapter<T> {
    fn input_post_process(&mut self) -> Result<(), anyhow::Error> {
        // Aquí se podría actualizar el estado interno del servo
        // basado en los datos leídos
        Ok(())
    }
    
    fn output_pre_process(&mut self) -> Result<(), anyhow::Error> {
        // Aquí se podrían preparar los datos antes de enviarlos
        Ok(())
    }
}

impl<T: Servo + Send + Sync + 'static> EthercatDeviceUsed for ServoDeviceAdapter<T> {
    fn set_used(&mut self) {
        self.used = true;
    }
    
    fn is_used(&self) -> bool {
        self.used
    }
}

impl<T: Servo + Send + Sync + 'static> std::fmt::Debug for ServoDeviceAdapter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServoDeviceAdapter")
            .field("used", &self.used)
            .field("control_word", &self.rxpdo.control_word)
            .field("target_position", &self.rxpdo.target_position)
            .field("status_word", &self.txpdo.status_word)
            .field("position_actual", &self.txpdo.position_actual)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethercat_devices::LichuanSimulator;
    
    #[test]
    fn test_adapter_creation() {
        let servo = LichuanSimulator::new();
        let adapter = ServoDeviceAdapter::new(servo);
        
        assert_eq!(adapter.input_len(), 12);
        assert_eq!(adapter.output_len(), 12);
        assert!(!adapter.is_used());
    }
    
    #[test]
    fn test_pdo_serialization() {
        let servo = LichuanSimulator::new();
        let mut adapter = ServoDeviceAdapter::new(servo);
        
        // Preparar datos de salida
        adapter.rxpdo_mut().control_word = 0x0F;
        adapter.rxpdo_mut().target_position = 1000;
        
        // Serializar
        let mut output_bits = bitvec![u8, Lsb0; 0; 96];
        adapter.output(&mut output_bits).unwrap();
        
        // Verificar
        let control_word = output_bits[0..16].load_le::<u16>();
        assert_eq!(control_word, 0x0F);
        
        let position = output_bits[16..48].load_le::<i32>();
        assert_eq!(position, 1000);
    }
}
```

**Verificar:**
```bash
cargo test adapters::servo_adapter
# Deben pasar los 2 tests básicos
```

**Checkpoint 4:** Adaptador básico implementado y testeado.

---

### Paso 5: Crear wrapper para Lichuan

**Archivo:** `control/ethercat-hal/src/devices/lichuan.rs` (NUEVO)

```rust
//! Dispositivos Lichuan LC10E

use crate::devices::adapters::ServoDeviceAdapter;
use ethercat_devices::{LichuanSimulator, LichuanHardware};
use control_core::machines::identification::DeviceIdentification;

/// Tipo alias para LC10E simulado
pub type LichuanLC10ESimulator = ServoDeviceAdapter<LichuanSimulator>;

/// Tipo alias para LC10E hardware
pub type LichuanLC10EHardware = ServoDeviceAdapter<LichuanHardware>;

/// Verificar si un dispositivo es un Lichuan LC10E
pub fn is_lichuan_lc10e(device_id: &DeviceIdentification) -> bool {
    device_id.vendor_id == 0x0766 && device_id.product_code == 0x0402
}

/// Factory function para crear LC10E (simulado o hardware)
pub fn create_lichuan_lc10e(
    _device_id: &DeviceIdentification,
    use_simulation: bool,
) -> Box<dyn crate::devices::EthercatDevice> {
    if use_simulation {
        log::info!("Creando Lichuan LC10E en modo SIMULACIÓN");
        Box::new(LichuanLC10ESimulator::new(LichuanSimulator::new()))
    } else {
        log::info!("Creando Lichuan LC10E con HARDWARE REAL");
        Box::new(LichuanLC10EHardware::new(LichuanHardware::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_lichuan() {
        let device_id = DeviceIdentification {
            vendor_id: 0x0766,
            product_code: 0x0402,
            revision: 0x0204,
            device_name: "LC10E".to_string(),
        };
        
        assert!(is_lichuan_lc10e(&device_id));
    }
    
    #[test]
    fn test_create_simulator() {
        let device_id = DeviceIdentification {
            vendor_id: 0x0766,
            product_code: 0x0402,
            revision: 0x0204,
            device_name: "LC10E".to_string(),
        };
        
        let device = create_lichuan_lc10e(&device_id, true);
        assert_eq!(device.input_len(), 12);
        assert_eq!(device.output_len(), 12);
    }
}
```

**Archivo:** `control/ethercat-hal/src/devices/mod.rs`

Agregar al final:
```rust
// ... código existente ...

// Wrappers por vendor
pub mod lichuan;
pub use lichuan::{LichuanLC10ESimulator, LichuanLC10EHardware, create_lichuan_lc10e, is_lichuan_lc10e};
```

**Verificar:**
```bash
cargo test lichuan
# Deben pasar los 2 tests
```

**Checkpoint 5:** Wrapper de Lichuan funcionando.

---

## ✅ Verificación Final Sprint 1

### Checklist

- [ ] Ambos repos clonados y compilando
- [ ] Dependencia `ethercat-devices` agregada en `control`
- [ ] Estructura de `adapters/` creada
- [ ] `ServoDeviceAdapter` implementado
- [ ] Tests básicos pasando (4 tests mínimo)
- [ ] Wrapper de Lichuan creado
- [ ] Sin warnings de compilación
- [ ] Documentación básica en comentarios

### Comandos de verificación final

```bash
cd control/ethercat-hal

# Compilación sin errores
cargo build

# Tests pasando
cargo test

# Sin warnings
cargo clippy

# Verificar que todo está en git
git status
git add .
git commit -m "feat: Add basic ServoDeviceAdapter for ethercat-devices integration"
```

---

## 📚 Siguientes Pasos

Una vez completado Sprint 1, continuar con:

### Sprint 2: Implementación Completa del Adaptador
- Agregar más métodos helper al adaptador
- Implementar sincronización de estado CiA402
- Agregar tests exhaustivos (cobertura >80%)
- Benchmarks de performance

### Sprint 3: Integración con Control Loop
- Modificar `server/src/ethercat/init.rs`
- Agregar configuración de simulación
- Crear máquina de ejemplo
- Integrar con frontend

**Ver:** [INTEGRACION_CON_CONTROL.md](INTEGRACION_CON_CONTROL.md) para el plan completo.

---

## 🆘 Troubleshooting

### Error: Cannot find `ethercat-devices`

```bash
# Verificar que la ruta en Cargo.toml es correcta
cd control/ethercat-hal
cat Cargo.toml | grep ethercat-devices

# Debe apuntar a la ruta correcta relativa
```

### Error: Trait bounds not satisfied

```bash
# Asegurarse de que todos los traits están importados
use crate::devices::{EthercatDevice, NewEthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed};
```

### Tests fallan con "unimplemented"

Esto es normal para `NewEthercatDevice::new()`. Usar los constructores específicos:
```rust
let adapter = ServoDeviceAdapter::new(LichuanSimulator::new());
```

---

## 💬 Contacto y Soporte

- **Issues:** GitHub Issues en ambos repos
- **Documentación:** [docs/](../docs/)
- **Ejemplos:** [examples/](../examples/)

---

**Documento creado:** Enero 30, 2026  
**Próxima actualización:** Después de completar Sprint 1  
**Tiempo estimado Sprint 1:** 2-3 días
