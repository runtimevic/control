# 🔗 Plan de Integración: rust-ethercat-devices ↔ control

**Proyecto Origen:** rust-ethercat-devices (Biblioteca de dispositivos EtherCAT)  
**Proyecto Destino:** control (Framework de control industrial QiTech)  
**Fecha:** Enero 30, 2026  
**Estado:** Plan de integración completo

---

## 📋 Resumen Ejecutivo

Este documento describe el plan completo para integrar la biblioteca `rust-ethercat-devices` en el framework `control` de QiTech, permitiendo que los dispositivos EtherCAT (servos, terminales E/S) funcionen tanto con hardware real como en modo simulación.

**Objetivo:** Extender `control` con soporte para múltiples vendors de dispositivos EtherCAT con capacidad de simulación cuando no hay hardware físico disponible.

**Beneficios:**
- ✅ Desarrollo sin hardware (simuladores)
- ✅ Testing automatizado sin dependencias físicas
- ✅ Soporte multi-vendor (Lichuan, Mitsubishi, Beckhoff)
- ✅ Implementación CiA402 completa
- ✅ Reutilización de código entre proyectos

---

## 🏗️ Arquitectura Actual

### Proyecto `control` (QiTech)

```
control/
├── ethercat-hal/              # Hardware Abstraction Layer
│   ├── src/devices/
│   │   ├── mod.rs            # Trait EthercatDevice
│   │   ├── beckhoff/         # Dispositivos Beckhoff (EL2004, EL3021, etc.)
│   │   └── wago/             # Dispositivos WAGO
│   └── src/pdo.rs            # PDO management
├── ethercat-hal-derive/       # Macros para derivar EthercatDevice
├── server/                    # Backend del control loop
│   └── src/ethercat/
│       └── init.rs           # Inicialización de EtherCAT
└── control-core/             # Lógica de control genérica
```

**Características:**
- Usa `ethercrab` para comunicación EtherCAT
- Trait `EthercatDevice` como abstracción principal
- PDO (Process Data Objects) para E/S cíclica
- CoE (CAN over EtherCAT) para configuración
- Control loop en tiempo real con `smol`
- Frontend Electron + React
- Backend Rust con Axum + SocketIO

### Proyecto `rust-ethercat-devices`

```
rust-ethercat-devices/
├── src/
│   ├── lib.rs                # API pública
│   ├── traits/
│   │   └── servo.rs          # Trait Servo (CiA402)
│   ├── common/
│   │   ├── simulation/
│   │   │   └── cia402.rs     # Simulador CiA402
│   │   ├── hardware/
│   │   │   ├── detection.rs  # Detección de hardware
│   │   │   ├── mock.rs       # Mock hardware para tests
│   │   │   └── stub.rs       # Stub para hardware no conectado
│   │   ├── pdo_config.rs     # Configuración de PDOs
│   │   ├── esi_parser.rs     # Parser de archivos ESI
│   │   └── error.rs          # Tipos de error
│   ├── lichuan/              # Lichuan LC10E
│   │   └── lc10e/
│   │       ├── simulator.rs  # Simulador
│   │       └── hardware.rs   # Hardware real
│   ├── smc_mitsubishi/       # SMC-Mitsubishi MR-J4-TM
│   │   └── mr_j4_tm/
│   │       ├── simulator.rs  # Simulador
│   │       └── hardware.rs   # Hardware real
│   └── beckhoff/             # Beckhoff EL1xxx/EL2xxx
│       └── terminals/
│           ├── simulator.rs  # Simulador
│           └── hardware.rs   # Hardware real
└── esi/                      # Archivos ESI de dispositivos
```

**Características:**
- Soporte para simulación y hardware real
- Implementación completa de CiA402
- Parser de archivos ESI
- Trait `Servo` para servomotores
- Sistema de feature flags por vendor
- 414 tests (76.20% cobertura)
- Mock/Stub para testing

---

## 🎯 Estrategia de Integración

### Opción Recomendada: Biblioteca Cargo

**Enfoque:** `rust-ethercat-devices` como dependencia cargo en `control`

**Ventajas:**
- ✅ Separación clara de responsabilidades
- ✅ Reutilización de código
- ✅ Versionado independiente
- ✅ Testing independiente
- ✅ Publicación en crates.io posible

**Arquitectura integrada:**
```
control/ethercat-hal/
├── Cargo.toml               # + dependency: ethercat-devices
└── src/devices/
    ├── mod.rs              # Trait EthercatDevice (existente)
    ├── adapters/           # NUEVO: Adaptadores
    │   ├── mod.rs
    │   ├── servo_adapter.rs      # Servo → EthercatDevice
    │   └── terminal_adapter.rs   # Terminal I/O → EthercatDevice
    ├── lichuan.rs          # NUEVO: Wrapper para Lichuan
    ├── mitsubishi.rs       # NUEVO: Wrapper para Mitsubishi
    └── beckhoff/
        └── el2xxx.rs       # Integración con simulación
```

---

## 📝 Modificaciones Necesarias

### En `rust-ethercat-devices` (Origen)

#### 1. **Preparar para uso como biblioteca**

**Estado actual:** ✅ Ya está preparado
- Metadata completa en `Cargo.toml`
- Documentación exhaustiva
- Feature flags por vendor
- API pública bien definida

**Cambios necesarios:** Ninguno crítico, opcional:

```toml
# Cargo.toml
[package]
publish = true  # Cambiar de false a true si se quiere publicar

# Feature para integración con ethercrab
[features]
default = ["simulation"]
ethercrab-integration = ["ethercrab", "tokio"]
```

#### 2. **Crear adaptador para trait `EthercatDevice`**

**Archivo nuevo:** `src/adapters/ethercat_hal.rs`

```rust
//! Adaptador para integración con ethercat-hal de QiTech Control

#[cfg(feature = "ethercrab-integration")]
use ethercrab::SubDevice;
use crate::traits::servo::Servo;
use crate::common::error::ServoError;

/// Adaptador que convierte cualquier Servo en un tipo compatible
/// con el trait EthercatDevice de ethercat-hal
pub struct ServoAdapter<T: Servo> {
    servo: T,
    pub rxpdo: ServoPDOOutput,  // Output del master → servo
    pub txpdo: ServoPDOInput,   // Input del servo → master
}

impl<T: Servo> ServoAdapter<T> {
    pub fn new(servo: T) -> Self {
        Self {
            servo,
            rxpdo: ServoPDOOutput::default(),
            txpdo: ServoPDOInput::default(),
        }
    }
    
    /// Obtener referencia al servo interno
    pub fn servo(&self) -> &T {
        &self.servo
    }
    
    /// Obtener referencia mutable al servo interno
    pub fn servo_mut(&mut self) -> &mut T {
        &mut self.servo
    }
}

// PDOs para CiA402 (ejemplo para CSP - Cyclic Synchronous Position)
#[derive(Default, Clone, Copy)]
pub struct ServoPDOOutput {
    pub control_word: u16,
    pub target_position: i32,
    pub target_velocity: i32,
    pub target_torque: i16,
}

#[derive(Default, Clone, Copy)]
pub struct ServoPDOInput {
    pub status_word: u16,
    pub position_actual: i32,
    pub velocity_actual: i32,
    pub torque_actual: i16,
}
```

#### 3. **Documentar casos de uso de integración**

**Archivo nuevo:** `docs/INTEGRACION_ETHERCAT_HAL.md`

```markdown
# Integración con ethercat-hal

Este documento explica cómo integrar rust-ethercat-devices
con el framework control de QiTech.

## Ejemplo básico

\`\`\`rust
use ethercat_devices::{LichuanSimulator, adapters::ServoAdapter};
use ethercat_hal::devices::EthercatDevice;

// Crear simulador
let servo = LichuanSimulator::new();

// Envolver en adaptador
let adapter = ServoAdapter::new(servo);

// Usar como EthercatDevice
// adapter.input(&input_bits)?;
// adapter.output(&mut output_bits)?;
\`\`\`
```

---

### En `control` (Destino)

#### 1. **Agregar dependencia en `ethercat-hal/Cargo.toml`**

```toml
[dependencies]
# ... dependencias existentes ...
ethercrab = "0.6"
# ... otras ...

# NUEVO: Biblioteca de dispositivos EtherCAT
ethercat-devices = { version = "0.1.0", path = "../../../rust-ethercat-devices" }
# O si está publicado:
# ethercat-devices = { version = "0.1.0", features = ["ethercrab-integration"] }
```

#### 2. **Crear módulo de adaptadores**

**Archivo nuevo:** `ethercat-hal/src/devices/adapters/mod.rs`

```rust
//! Adaptadores para integrar ethercat-devices con ethercat-hal

pub mod servo_adapter;
pub mod terminal_adapter;

pub use servo_adapter::ServoDeviceAdapter;
pub use terminal_adapter::TerminalDeviceAdapter;
```

**Archivo nuevo:** `ethercat-hal/src/devices/adapters/servo_adapter.rs`

```rust
//! Adaptador de Servo (ethercat-devices) → EthercatDevice (ethercat-hal)

use crate::devices::{EthercatDevice, NewEthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed};
use ethercat_devices::traits::servo::Servo;
use ethercat_devices::adapters::ServoAdapter;
use bitvec::prelude::*;
use std::any::Any;

/// Adaptador que permite usar cualquier Servo de ethercat-devices
/// como un EthercatDevice de ethercat-hal
pub struct ServoDeviceAdapter<T: Servo + Send + Sync + 'static> {
    adapter: ServoAdapter<T>,
    used: bool,
}

impl<T: Servo + Send + Sync + 'static> ServoDeviceAdapter<T> {
    pub fn new(servo: T) -> Self {
        Self {
            adapter: ServoAdapter::new(servo),
            used: false,
        }
    }
    
    /// Acceso directo al servo para control de alto nivel
    pub fn servo(&self) -> &T {
        self.adapter.servo()
    }
    
    pub fn servo_mut(&mut self) -> &mut T {
        self.adapter.servo_mut()
    }
}

// Implementar trait EthercatDevice
impl<T: Servo + Send + Sync + 'static> EthercatDevice for ServoDeviceAdapter<T> {
    fn input(&mut self, input: &BitSlice<u8, Lsb0>) -> Result<(), anyhow::Error> {
        // Leer TxPDO (servo → master)
        let mut offset = 0;
        
        // Status word (16 bits)
        let status_word = input[offset..offset+16].load_le::<u16>();
        self.adapter.txpdo.status_word = status_word;
        offset += 16;
        
        // Position actual (32 bits)
        let position = input[offset..offset+32].load_le::<i32>();
        self.adapter.txpdo.position_actual = position;
        offset += 32;
        
        // Velocity actual (32 bits)
        let velocity = input[offset..offset+32].load_le::<i32>();
        self.adapter.txpdo.velocity_actual = velocity;
        offset += 32;
        
        // Torque actual (16 bits)
        let torque = input[offset..offset+16].load_le::<i16>();
        self.adapter.txpdo.torque_actual = torque;
        
        Ok(())
    }
    
    fn input_len(&self) -> usize {
        // Status word (16) + position (32) + velocity (32) + torque (16) = 96 bits
        96 / 8 // 12 bytes
    }
    
    fn output(&self, output: &mut BitSlice<u8, Lsb0>) -> Result<(), anyhow::Error> {
        // Escribir RxPDO (master → servo)
        let mut offset = 0;
        
        // Control word (16 bits)
        output[offset..offset+16].store_le(self.adapter.rxpdo.control_word);
        offset += 16;
        
        // Target position (32 bits)
        output[offset..offset+32].store_le(self.adapter.rxpdo.target_position);
        offset += 32;
        
        // Target velocity (32 bits)
        output[offset..offset+32].store_le(self.adapter.rxpdo.target_velocity);
        offset += 32;
        
        // Target torque (16 bits)
        output[offset..offset+16].store_le(self.adapter.rxpdo.target_torque);
        
        Ok(())
    }
    
    fn output_len(&self) -> usize {
        // Control word (16) + target position (32) + velocity (32) + torque (16) = 96 bits
        96 / 8 // 12 bytes
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
    
    fn set_module(&mut self, _module: crate::devices::Module) {}
}

impl<T: Servo + Send + Sync + 'static> NewEthercatDevice for ServoDeviceAdapter<T> {
    fn new() -> Self {
        // Nota: Este método se usa para crear dispositivos genéricos
        // Para servos específicos, usar constructores específicos
        unimplemented!("Use constructores específicos para cada vendor")
    }
}

impl<T: Servo + Send + Sync + 'static> EthercatDeviceProcessing for ServoDeviceAdapter<T> {
    fn input_post_process(&mut self) -> Result<(), anyhow::Error> {
        // Actualizar el simulador con los datos leídos
        // (Si es un simulador)
        Ok(())
    }
    
    fn output_pre_process(&mut self) -> Result<(), anyhow::Error> {
        // Preparar datos antes de enviar
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
            .finish()
    }
}
```

#### 3. **Crear dispositivos específicos por vendor**

**Archivo nuevo:** `ethercat-hal/src/devices/lichuan/lc10e.rs`

```rust
//! Lichuan LC10E Servo Drive

use crate::devices::adapters::ServoDeviceAdapter;
use ethercat_devices::{LichuanSimulator, LichuanHardware};
use control_core::machines::identification::DeviceIdentification;

pub type LichuanLC10ESimulator = ServoDeviceAdapter<LichuanSimulator>;
pub type LichuanLC10EHardware = ServoDeviceAdapter<LichuanHardware>;

/// Crear dispositivo Lichuan LC10E en modo apropiado (simulado o hardware)
pub fn create_lichuan_lc10e(
    device_id: &DeviceIdentification,
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

/// Verificar si un dispositivo es un Lichuan LC10E
pub fn is_lichuan_lc10e(device_id: &DeviceIdentification) -> bool {
    device_id.vendor_id == 0x0766 && device_id.product_code == 0x0402
}

#[cfg(test)]
mod tests {
    use super::*;
    
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

#### 4. **Modificar inicialización de EtherCAT en `server`**

**Archivo:** `server/src/ethercat/init.rs`

```rust
use ethercat_hal::devices::{
    EthercatDevice,
    lichuan::lc10e::{create_lichuan_lc10e, is_lichuan_lc10e},
    // ... otros imports
};

pub async fn initialize_devices(
    subdevices: &[SubDevice],
    use_simulation: bool, // NUEVO parámetro
) -> Result<Vec<Box<dyn EthercatDevice>>, anyhow::Error> {
    let mut devices = Vec::new();
    
    for subdevice in subdevices {
        let device_id = read_device_identification(subdevice).await?;
        
        // Detección automática del tipo de dispositivo
        let device: Box<dyn EthercatDevice> = if is_lichuan_lc10e(&device_id) {
            create_lichuan_lc10e(&device_id, use_simulation)
        } else if is_mitsubishi_mrj4tm(&device_id) {
            create_mitsubishi_mrj4tm(&device_id, use_simulation)
        } else if is_beckhoff_el2004(&device_id) {
            create_beckhoff_el2004(&device_id, use_simulation)
        } else {
            // Dispositivo desconocido
            log::warn!("Dispositivo desconocido: {:?}", device_id);
            continue;
        };
        
        devices.push(device);
    }
    
    Ok(devices)
}
```

#### 5. **Agregar configuración de simulación**

**Archivo:** `server/src/config.rs` (nuevo o existente)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Usar simulación cuando no hay hardware disponible
    pub use_simulation: bool,
    
    /// Lista de dispositivos que deben estar en simulación
    /// aunque haya hardware disponible (para testing)
    pub force_simulation: Vec<String>,
    
    /// Interfaz de red EtherCAT
    pub ethercat_interface: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            use_simulation: std::env::var("SIMULATION_MODE")
                .unwrap_or_default() == "true",
            force_simulation: Vec::new(),
            ethercat_interface: "eth0".to_string(),
        }
    }
}
```

#### 6. **Crear máquina de ejemplo/test con servo simulado**

**Archivo nuevo:** `machines/src/test_servo.rs`

```rust
//! Máquina de prueba con servo Lichuan simulado

use control_core::machines::{Machine, MachineState};
use ethercat_hal::devices::lichuan::lc10e::LichuanLC10ESimulator;
use ethercat_devices::traits::servo::Servo;

pub struct TestServoMachine {
    servo: LichuanLC10ESimulator,
    state: MachineState,
}

impl TestServoMachine {
    pub fn new() -> Self {
        Self {
            servo: LichuanLC10ESimulator::new(
                ethercat_devices::LichuanSimulator::new()
            ),
            state: MachineState::Idle,
        }
    }
    
    /// Mover a posición objetivo
    pub fn move_to_position(&mut self, target: i32) -> Result<(), anyhow::Error> {
        let servo = self.servo.servo_mut();
        servo.set_target_position(target)?;
        Ok(())
    }
    
    /// Leer posición actual
    pub fn get_position(&self) -> Result<i32, anyhow::Error> {
        self.servo.servo().get_position_actual()
    }
}

impl Machine for TestServoMachine {
    fn update(&mut self, _dt: f64) -> Result<(), anyhow::Error> {
        // Actualizar simulador si es necesario
        Ok(())
    }
    
    fn get_state(&self) -> MachineState {
        self.state.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_servo_machine() {
        let mut machine = TestServoMachine::new();
        machine.move_to_position(1000).unwrap();
        // En simulación, la posición se alcanza instantáneamente o progresivamente
        // dependiendo de la implementación del simulador
    }
}
```

---

## 📅 Plan de Sprints

### 🎯 Sprint 1: Preparación y Fundamentos (1 semana)

**Objetivo:** Establecer la base de integración sin romper código existente

**Tareas en `rust-ethercat-devices`:**
1. ✅ Verificar que Cargo.toml tiene metadata completa (YA HECHO)
2. ✅ Verificar que tests pasan (414 tests pasando)
3. ⏳ Crear módulo `adapters/ethercat_hal.rs` con:
   - Struct `ServoAdapter<T: Servo>`
   - PDOs básicos (ServoPDOOutput, ServoPDOInput)
   - Documentación de uso
4. ⏳ Agregar feature flag `ethercrab-integration`
5. ⏳ Escribir tests para el adaptador (mock)
6. ⏳ Documentar en `docs/INTEGRACION_ETHERCAT_HAL.md`

**Tareas en `control`:**
1. ⏳ Clonar o hacer fork de `control` si aún no está
2. ⏳ Estudiar arquitectura de `ethercat-hal`:
   - Trait `EthercatDevice`
   - Sistema de PDOs existente
   - Macro `#[derive(EthercatDevice)]`
3. ⏳ Agregar `ethercat-devices` como dependencia path en `ethercat-hal/Cargo.toml`
4. ⏳ Crear directorio `ethercat-hal/src/devices/adapters/`
5. ⏳ Compilar para verificar que no hay conflictos de dependencias

**Entregables:**
- [ ] `rust-ethercat-devices` con módulo `adapters`
- [ ] `control/ethercat-hal` con dependencia agregada
- [ ] Documento de arquitectura actualizado
- [ ] Tests básicos del adaptador pasando

**Criterios de aceptación:**
- Compilación exitosa de ambos proyectos
- No hay warnings de Rust
- Tests existentes siguen pasando
- Documentación básica lista

---

### 🎯 Sprint 2: Adaptador ServoDevice (2 semanas)

**Objetivo:** Implementar adaptador completo que permita usar Servo como EthercatDevice

**Tareas en `rust-ethercat-devices`:**
1. ⏳ Implementar conversión PDO completa:
   - Status word → Control word mapping
   - Position/Velocity/Torque actual/target
2. ⏳ Agregar métodos helper para configuración CiA402:
   - Setup modes (CSP, CSV, CST)
   - Emergency stop
   - Homing procedures
3. ⏳ Tests de integración con mock PDO data
4. ⏳ Benchmarks de performance del adaptador

**Tareas en `control`:**
1. ⏳ Implementar `ServoDeviceAdapter` en `ethercat-hal/src/devices/adapters/servo_adapter.rs`:
   - Implementar trait `EthercatDevice`
   - Implementar trait `NewEthercatDevice`
   - Implementar trait `EthercatDeviceProcessing`
   - Implementar trait `EthercatDeviceUsed`
2. ⏳ Crear wrappers específicos:
   - `devices/lichuan/lc10e.rs`
   - `devices/mitsubishi/mr_j4_tm.rs`
3. ⏳ Tests unitarios del adaptador
4. ⏳ Ejemplo standalone (sin control loop completo)

**Entregables:**
- [ ] Adaptador funcional para Servo → EthercatDevice
- [ ] Tests de adaptador (>80% cobertura)
- [ ] Ejemplo funcionando con simulador
- [ ] Documentación técnica del adaptador

**Criterios de aceptación:**
- Servo simulado funciona como EthercatDevice
- PDO input/output correctos
- Tests de integración pasando
- Sin memory leaks (valgrind/miri)

---

### 🎯 Sprint 3: Integración con Control Loop (2 semanas)

**Objetivo:** Integrar dispositivos simulados en el control loop real de `control`

**Tareas en `control`:**
1. ⏳ Modificar `server/src/ethercat/init.rs`:
   - Agregar parámetro `use_simulation`
   - Implementar detección automática de dispositivos
   - Factory function para crear dispositivos
2. ⏳ Crear `ServerConfig` con opciones de simulación
3. ⏳ Modificar control loop para soportar simuladores:
   - Actualizar simuladores en cada ciclo
   - Logging de estado de simulación
4. ⏳ Crear máquina de prueba `TestServoMachine`
5. ⏳ Integrar con frontend:
   - Indicador visual de modo simulación
   - API para cambiar entre real/simulado

**Tareas en ambos proyectos:**
1. ⏳ Tests de integración E2E:
   - Control loop con servo simulado
   - Comandos de posición/velocidad
   - Lectura de estado
2. ⏳ Performance testing:
   - Latencia del control loop
   - CPU usage
   - Memory footprint

**Entregables:**
- [ ] Control loop funcionando con simulador
- [ ] Máquina de prueba operacional
- [ ] Tests E2E pasando
- [ ] Frontend mostrando modo simulación

**Criterios de aceptación:**
- Control loop mantiene timing con simulador
- Cambio entre real/simulado sin reinicio (nice to have)
- Tests E2E estables (>95% success rate)
- Documentación de uso completa

---

### 🎯 Sprint 4: Soporte Multi-Vendor (1 semana)

**Objetivo:** Agregar soporte para Mitsubishi MR-J4-TM y Beckhoff

**Tareas en `control`:**
1. ⏳ Implementar wrapper para Mitsubishi:
   - `devices/mitsubishi/mr_j4_tm.rs`
   - Tests específicos
2. ⏳ Implementar wrapper para terminales Beckhoff:
   - `devices/beckhoff/el1008.rs` (DI)
   - `devices/beckhoff/el2004.rs` (DO)
   - `devices/beckhoff/el2008.rs` (DO)
3. ⏳ Actualizar detección automática
4. ⏳ Crear adaptador para terminales E/S (no servos):
   - `TerminalDeviceAdapter`
   - Digital Input/Output específico

**Tareas en `rust-ethercat-devices`:**
1. ⏳ Verificar implementaciones de Mitsubishi y Beckhoff
2. ⏳ Agregar ejemplos específicos de cada vendor
3. ⏳ Tests de compatibilidad

**Entregables:**
- [ ] Soporte para 3 vendors (Lichuan, Mitsubishi, Beckhoff)
- [ ] Factory pattern para crear dispositivos
- [ ] Tests para cada vendor
- [ ] Ejemplo con máquina multi-vendor

**Criterios de aceptación:**
- Al menos 2 vendors funcionando simultáneamente
- Detección automática correcta (>99%)
- Tests de integración pasando para todos los vendors

---

### 🎯 Sprint 5: Hardware Real (2 semanas)

**Objetivo:** Integrar con hardware físico y validar en entorno real

**Tareas (requiere hardware):**
1. ⏳ Configurar entorno con servo físico:
   - Lichuan LC10E o Mitsubishi MR-J4-TM
   - EtherCAT master (Raspberry Pi / PC Industrial)
2. ⏳ Implementar `LichuanHardware` usando `ethercrab`:
   - SDO configuration
   - PDO mapping real
   - Error handling para fallos de comunicación
3. ⏳ Tests con hardware:
   - Startup sequence
   - Position control
   - Velocity control
   - Emergency stop
4. ⏳ Comparar simulador vs hardware:
   - Timing differences
   - Accuracy
   - Performance

**Tareas de validación:**
1. ⏳ Safety testing:
   - Emergency stop response time
   - Error recovery
   - Fault handling
2. ⏳ Performance testing:
   - Control loop jitter
   - PDO cycle time
   - CPU usage en tiempo real

**Entregables:**
- [ ] Hardware real funcionando con `control`
- [ ] Comparativa simulador vs hardware
- [ ] Safety procedures documentadas
- [ ] Performance report

**Criterios de aceptación:**
- Control loop estable con hardware (jitter <100µs)
- Emergency stop <50ms
- Error handling robusto
- Documentación de seguridad completa

---

### 🎯 Sprint 6: Optimización y Productización (1 semana)

**Objetivo:** Optimizar performance y preparar para producción

**Tareas:**
1. ⏳ Profiling y optimización:
   - Eliminar allocations en hot path
   - Optimizar PDO serialization
   - Reducir latencia del adaptador
2. ⏳ Configuración de producción:
   - Feature flags para release
   - Logging optimizado
   - Error reporting
3. ⏳ Documentación completa:
   - Manual de usuario
   - Guía de troubleshooting
   - API reference
4. ⏳ CI/CD:
   - Tests automáticos
   - Benchmarks en CI
   - Release automation

**Entregables:**
- [ ] Performance optimizado (latencia <10µs)
- [ ] Documentación completa
- [ ] CI/CD configurado
- [ ] Release 1.0.0

**Criterios de aceptación:**
- Benchmarks dentro de targets
- Documentación completa y revisada
- CI verde (100% tests passing)
- Ready para producción

---

## 🔧 Aspectos Técnicos Detallados

### Trait Compatibility

**Problema:** `control` usa `EthercatDevice` mientras `rust-ethercat-devices` usa `Servo`

**Solución:** Patrón Adapter

```rust
// ethercat-devices tiene:
pub trait Servo {
    fn process_control_word(&mut self, word: u16) -> Result<(), Box<dyn Error>>;
    fn get_status_word(&self) -> Result<u16, Box<dyn Error>>;
    // ... otros métodos CiA402
}

// control tiene:
pub trait EthercatDevice {
    fn input(&mut self, input: &BitSlice<u8, Lsb0>) -> Result<(), anyhow::Error>;
    fn output(&self, output: &mut BitSlice<u8, Lsb0>) -> Result<(), anyhow::Error>;
    fn input_len(&self) -> usize;
    fn output_len(&self) -> usize;
}

// Adaptador conecta ambos:
pub struct ServoDeviceAdapter<T: Servo> {
    servo: T,
    rxpdo: ServoPDOOutput,  // Control word, targets
    txpdo: ServoPDOInput,   // Status word, actuals
}
```

### PDO Mapping

**CiA402 PDO estándar para CSP (Cyclic Synchronous Position):**

```
RxPDO (Master → Servo):
- 0x6040: Control Word (16 bits)
- 0x607A: Target Position (32 bits)
- 0x60FF: Target Velocity (32 bits)
- 0x6071: Target Torque (16 bits)
Total: 96 bits (12 bytes)

TxPDO (Servo → Master):
- 0x6041: Status Word (16 bits)
- 0x6064: Position Actual (32 bits)
- 0x606C: Velocity Actual (32 bits)
- 0x6077: Torque Actual (16 bits)
Total: 96 bits (12 bytes)
```

### Simulación vs Hardware

**Modos de operación:**

| Modo | Descripción | Cuando usar |
|------|-------------|-------------|
| **Simulación pura** | Sin ethercrab, todo simulado | Desarrollo sin hardware, CI/CD |
| **Hardware real** | Ethercrab + dispositivos físicos | Producción, validación final |
| **Híbrido** | Algunos simulados, otros reales | Testing progresivo, desarrollo |

**Implementación:**

```rust
// Configuración en server/config.toml
[ethercat]
use_simulation = true  # Global flag

[[devices]]
name = "servo1"
type = "lichuan_lc10e"
force_simulation = false  # Override per-device

[[devices]]
name = "servo2"
type = "mitsubishi_mrj4tm"
force_simulation = true  # Siempre simulado
```

### Error Handling

**Estrategia de errores:**

```rust
// Errores de ethercat-devices
pub enum ServoError {
    InvalidState { current: u8, expected: u8 },
    CommunicationError(String),
    NotImplemented,
    // ... otros
}

// Conversión a errores de control
impl From<ServoError> for anyhow::Error {
    fn from(err: ServoError) -> Self {
        anyhow::anyhow!("Servo error: {:?}", err)
    }
}
```

### Logging y Debugging

**Sistema de logging unificado:**

```rust
// Usar log crate en ambos proyectos
use log::{info, debug, warn, error};

// En control loop
debug!("Servo position: {} (target: {})", actual, target);
warn!("Servo not reaching target in time");
error!("Emergency stop triggered!");

// Configuración en runtime
RUST_LOG=debug cargo run              # Todo en debug
RUST_LOG=control=info,ethercat=debug  # Granular
```

---

## 📊 Métricas de Éxito

### Performance Targets

| Métrica | Simulación | Hardware Real | Comentario |
|---------|-----------|---------------|------------|
| Control loop cycle time | 1ms | 1ms | Tiempo de ciclo EtherCAT estándar |
| Adaptador overhead | <10µs | <10µs | Latencia del adapter |
| Memory footprint | <1MB por device | <1MB por device | Memoria por dispositivo |
| CPU usage @ 1kHz | <5% | <10% | Un core, Raspberry Pi 4 |
| Jitter | N/A | <100µs | Desviación del ciclo |

### Quality Targets

| Aspecto | Target | Medición |
|---------|--------|----------|
| Code coverage | >80% | `cargo tarpaulin` |
| Test success rate | >99% | CI pipeline |
| Compilation warnings | 0 | `cargo clippy` |
| Documentation | 100% public API | `cargo doc` |
| Security audit | 0 critical issues | `cargo audit` |

### Integration Targets

| Feature | Sprint | Status |
|---------|--------|--------|
| Servo simulado en control loop | Sprint 3 | ⏳ |
| Multi-vendor support | Sprint 4 | ⏳ |
| Hardware real funcionando | Sprint 5 | ⏳ |
| Modo híbrido (mix real/sim) | Sprint 5 | ⏳ |
| CI/CD completo | Sprint 6 | ⏳ |

---

## 🚧 Riesgos y Mitigaciones

### Riesgo 1: Incompatibilidad de PDO layouts

**Descripción:** Los PDO layouts entre `ethercat-devices` y `control` pueden no coincidir

**Probabilidad:** Media  
**Impacto:** Alto

**Mitigación:**
- Crear abstracción de PDO configurable
- Tests exhaustivos de serialización/deserialización
- Validación contra archivos ESI

### Riesgo 2: Performance del adaptador

**Descripción:** El adaptador puede agregar latencia inaceptable

**Probabilidad:** Baja  
**Impacto:** Alto

**Mitigación:**
- Zero-copy donde sea posible
- Benchmarks tempranos (Sprint 2)
- Profile en hardware real

### Riesgo 3: Falta de hardware para testing

**Descripción:** No tener acceso a servos físicos para Sprint 5

**Probabilidad:** Media  
**Impacto:** Medio

**Mitigación:**
- Simuladores de alta fidelidad
- Testear con terminales E/S más baratos primero
- Partnership con vendedores para préstamos

### Riesgo 4: Breaking changes en ethercrab

**Descripción:** Actualizaciones de `ethercrab` pueden romper compatibilidad

**Probabilidad:** Baja  
**Impacto:** Medio

**Mitigación:**
- Pin version de ethercrab
- Monitorear releases
- Tests de compatibilidad en CI

---

## 📚 Referencias y Recursos

### Documentación Técnica

- **CiA402 Specification:** [can-cia.org](https://www.can-cia.org/can-knowledge/canopen/cia402/)
- **EtherCAT Technology Group:** [ethercat.org](https://www.ethercat.org)
- **Ethercrab Documentation:** [docs.rs/ethercrab](https://docs.rs/ethercrab)

### Proyectos Relacionados

- **QiTech Control:** https://github.com/qitechgmbh/control
- **rust-ethercat-devices:** https://github.com/runtimevic/rust-ethercat-devices
- **Ethercrab:** https://github.com/ethercrab-rs/ethercrab

### Archivos ESI

Los archivos ESI (EtherCAT Slave Information) de cada dispositivo están en:
- `rust-ethercat-devices/esi/`
- Disponibles en sitios web de fabricantes

### Herramientas

- **TwinCAT:** Para análisis de EtherCAT (Windows)
- **IGH EtherCAT Master:** Alternativa open-source (Linux)
- **Wireshark:** Para debug de protocolos EtherCAT

---

## 🎓 Guía de Inicio Rápido

### Para Desarrolladores de `rust-ethercat-devices`

```bash
# 1. Clonar ambos repos
git clone https://github.com/runtimevic/rust-ethercat-devices.git
git clone https://github.com/runtimevic/control.git

# 2. Trabajar en adaptador
cd rust-ethercat-devices
git checkout -b feature/ethercat-hal-adapter

# 3. Implementar adaptador (Sprint 1-2)
# ... código ...

# 4. Tests
cargo test --all-features

# 5. Push y PR
git push origin feature/ethercat-hal-adapter
```

### Para Desarrolladores de `control`

```bash
# 1. Actualizar control
cd control/ethercat-hal

# 2. Agregar dependencia
# Editar Cargo.toml

# 3. Implementar wrappers (Sprint 2-3)
# ... código ...

# 4. Tests
cargo test

# 5. Test integración
cd ../server
cargo run  # Con servo simulado
```

### Configuración de Entorno

```bash
# Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Herramientas útiles
cargo install cargo-tarpaulin  # Coverage
cargo install cargo-audit      # Security
cargo install cargo-flamegraph # Profiling

# Variables de entorno
export RUST_LOG=debug
export SIMULATION_MODE=true
```

---

## 💡 Conclusión

Este plan de integración permite:

✅ **Reutilizar código:** `rust-ethercat-devices` se integra sin duplicación  
✅ **Desarrollo flexible:** Trabajar con/sin hardware  
✅ **Testing robusto:** Simuladores para CI/CD  
✅ **Multi-vendor:** Soporte para múltiples fabricantes  
✅ **Escalabilidad:** Fácil agregar nuevos dispositivos  
✅ **Producción ready:** Performance y seguridad validados  

**Timeline total:** ~8 semanas (2 meses)  
**Esfuerzo estimado:** 1-2 desarrolladores full-time  
**Riesgo general:** Bajo-Medio (con mitigaciones)

**Próximos pasos inmediatos:**
1. Revisar y aprobar este documento
2. Iniciar Sprint 1 (Preparación)
3. Setup de entorno de desarrollo
4. Primera implementación del adaptador

---

**Documento creado:** Enero 30, 2026  
**Autor:** GitHub Copilot  
**Versión:** 1.0  
**Estado:** ✅ Listo para revisión
