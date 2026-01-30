# Integración de rust-ethercat-devices - Ejemplo de Uso

## Estado de la Integración

✅ **Sprint 1 Completado** - Configuración Inicial  
✅ **Sprint 2 Completado** - Adaptadores y Mapeo PDO

La dependencia `rust-ethercat-devices` está totalmente integrada y lista para usar en el bucle de control.

### Dispositivos Soportados

- ✅ **Lichuan LC10E** - Simulador y Hardware
- ✅ **SMC-Mitsubishi MR-J4-TM** - Simulador y Hardware

## Configuración

En [`ethercat-hal/Cargo.toml`](../ethercat-hal/Cargo.toml):

```toml
[dependencies]
# rust-ethercat-devices integration via GitHub
ethercat-devices = { git = "https://github.com/runtimevic/rust-ethercat-devices.git", features = ["simulation"] }
```

## Uso Básico

### Sprint 2: Usar servos como EthercatDevice

```rust
use ethercat_hal::devices::{
    LichuanSimulator,
    SmcMitsubishiSimulator,
    ServoAdapter,
};

// Crear adaptador que implementa EthercatDevice
let lichuan_servo = ServoAdapter::new(LichuanSimulator::new());
let mitsubishi_servo = ServoAdapter::new(SmcMitsubishiSimulator::new());

// Ahora pueden usarse en el bucle de control como cualquier otro EthercatDevice
// implementan input(), output(), input_len(), output_len(), etc.
```

### Importar los tipos

```rust
use ethercat_hal::devices::lichuan::{LichuanSimulator, LichuanHardware, is_lichuan_lc10e};
```

### Crear un simulador

```rust
use ethercat_devices::common::constants::control_word;
use ethercat_devices::common::error::ServoError;

fn example_simulator() -> Result<(), ServoError> {
    // Crear simulador
    let mut servo = LichuanSimulator::new();
    
    // Secuencia de arranque CiA402
    servo.process_control_word(control_word::SHUTDOWN)?;
    servo.process_control_word(control_word::SWITCH_ON)?;
    servo.process_control_word(control_word::ENABLE_OPERATION)?;
    
    // Configurar modo CSP (Cyclic Synchronous Position)
    servo.set_mode_of_operation(8)?;
    
    // Mover a posición objetivo
    servo.set_target_position(10000)?;
    
    // Simular el ciclo
    for _ in 0..1000 {
        servo.update(1)?; // 1ms de ciclo
    }
    
    // Leer estado
    let position = servo.get_position_actual()?;
    let status = servo.get_status_word()?;
    
    println!("Posición actual: {}", position);
    println!("Status word: 0x{:04X}", status);
    
    Ok(())
}
```

### Identificación de dispositivos

```rust
use ethercat_hal::devices::lichuan::is_lichuan_lc10e;

// Verificar si un dispositivo EtherCAT es un Lichuan LC10E
if is_lichuan_lc10e(vendor_id, product_code) {
    // Crear instancia del servo
    let servo = LichuanSimulator::new();
    // ...
}
```

### Usando el Builder Pattern

```rust
let servo = LichuanSimulator::builder()
    .max_velocity(5000)
    .max_acceleration(10000)
    .initial_position(1000)
    .build();
```

## Constantes de Identificación

```rust
use ethercat_hal::devices::lichuan::{
    LICHUAN_LC10E_VENDOR_ID,    // 0x00000766
    LICHUAN_LC10E_PRODUCT_CODE, // 0x00000402
};
```

## Próximos Pasos

### Sprint 3: Integración en Máquinas Específicas

El siguiente sprint debe:
1. Integrar los servos en máquinas concretas del proyecto
2. Configurar presets para cada máquina
3. Tests end-to-end con bucle de control real

## Arquitectura de la Integración

```
rust-ethercat-devices (GitHub)
    ├── LichuanSimulator
    └── SmcMitsubishiSimulator
           ↓
ethercat-hal/devices/
    ├── lichuan.rs (Wrapper)
    └── smc_mitsubishi.rs (Wrapper)
           ↓
ethercat-hal/devices/adapters/
    ├── servo_adapter.rs (ServoAdapter<T>)
    ├── pdo_mapping.rs (BitSlice ↔ Cia402PDO)
    └── mod.rs (ServoDevice trait)
           ↓
    EthercatDevice trait
           ↓
    Control Loop
```

## Archivos Creados/Modificados

**Sprint 1:**
- [`ethercat-hal/Cargo.toml`](../../ethercat-hal/Cargo.toml) - Dependencia de GitHub
- [`ethercat-hal/src/devices/lichuan.rs`](../../ethercat-hal/src/devices/lichuan.rs) - Wrapper Lichuan
- [`ethercat-hal/src/devices/smc_mitsubishi.rs`](../../ethercat-hal/src/devices/smc_mitsubishi.rs) - Wrapper Mitsubishi

**Sprint 2:**
- [`ethercat-hal/src/devices/adapters/mod.rs`](../../ethercat-hal/src/devices/adapters/mod.rs) - Trait ServoDevice y estructuras PDO
- [`ethercat-hal/src/devices/adapters/servo_adapter.rs`](../../ethercat-hal/src/devices/adapters/servo_adapter.rs) - ServoAdapter<T>
- [`ethercat-hal/src/devices/adapters/pdo_mapping.rs`](../../ethercat-hal/src/devices/adapters/pdo_mapping.rs) - Mapeo PDO

## Comandos Útiles

```bash
# Compilar solo ethercat-hal
cargo check -p ethercat_hal

# Compilar todo el workspace
cargo build

# Ver documentación generada
cargo doc --open -p ethercat_hal
```

## Notas

- La integración usa el branch `master` de rust-ethercat-devices
- El feature `simulation` está habilitado por defecto para desarrollo
- Para hardware real, se puede cambiar al feature `hardware`
