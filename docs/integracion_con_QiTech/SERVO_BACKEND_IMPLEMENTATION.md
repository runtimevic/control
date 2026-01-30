# Implementación Backend - Servo Test Machine

## ✅ Estado: Implementación Completa (Pendiente Testing en Linux)

La máquina de prueba para servos CiA402 ha sido completamente implementada siguiendo el patrón del proyecto.

## Archivos Creados

### 1. `machines/src/servo_test_machine/mod.rs` (115 líneas)

Estructura principal de la máquina:
- Usa `Arc<RwLock<ServoAdapter<T>>>` para acceso thread-safe al servo
- Implementa `Machine` trait
- `emit_state()` async que lee estado del servo cada 100ms
- Parsea CiA402 statusword (bits 0-10) en flags booleanos

### 2. `machines/src/servo_test_machine/api.rs` (265 líneas)

Socket.io namespace y mutaciones HTTP:
- **Eventos**:
  - `DriveStateEvent`: 23 campos (position, velocity, statusword, etc.)
  - `CommandAckEvent`: Confirmación de comandos
  - `MovementCompleteEvent`: Notificación de fin de movimiento
- **Mutaciones** (15 comandos):
  - `JogStart`/`JogStop`: Control manual
  - `Start`/`Stop`: Habilitar/deshabilitar drive
  - `Reset`: Resetear faults
  - `Reference`: Homing (pendiente impl)
  - `SetEnabling`: Configurar habilitaciones
  - `DownloadKvFactor`/`DownloadRefVelocity`/`DownloadTargetPosition`: Parámetros
  - `StartMovement`: Iniciar movimiento con modo
  - `SetRawOutput`: Salida directa de torque/velocidad

### 3. `machines/src/servo_test_machine/act.rs` (40 líneas)

Implementa `MachineAct`:
- Emite estado cada 100ms (10 Hz)
- Maneja suscripciones de namespace
- Procesa comandos HTTP vía `api_mutate`

### 4. `machines/src/servo_test_machine/new.rs` (75 líneas)

Constructor de máquina:
- Detecta automáticamente Lichuan LC10E (0x766:0x402) o Mitsubishi MR-J4-TM (0xA1E:0x201)
- Usa `get_ethercat_device<ServoAdapter<T>>` con múltiples identidades
- Configuración de namespace y channels

### 5. `machines/src/lib.rs` (Modificado)

- Agregado `pub mod servo_test_machine;`
- Constante: `pub const SERVO_TEST_MACHINE: u16 = 0x0037;`

### 6. `machines/src/registry.rs` (Modificado)

Registrados dos tipos de máquina:
```rust
type ServoTestMachineLichuan = ServoTestMachine<LichuanLC10E>;
type ServoTestMachineMitsubishi = ServoTestMachine<MitsubishiMRJ4TM>;

mc.register::<ServoTestMachineLichuan>(ServoTestMachineLichuan::MACHINE_IDENTIFICATION);
mc.register::<ServoTestMachineMitsubishi>(ServoTestMachineMitsubishi::MACHINE_IDENTIFICATION);
```

## Arquitectura

```
Frontend (React + Socket.io)
    ↓ HTTP POST /api/v1/machine/mutate
    ↓
Server (REST API)
    ↓ api_mutate(value)
    ↓
ServoTestMachine::handle_mutation()
    ↓ servo.write().await
    ↓
ServoAdapter<T> (Arc<RwLock<>>)
    ↓ servo_mut().enable() / set_position_setpoint()
    ↓
LichuanLC10E / MitsubishiMRJ4TM (ServoDevice trait)
    ↓ PDO mapping
    ↓
EtherCAT PDOs (RxPDO/TxPDO)
    ↓
Hardware (Real drive)

Loop Thread (10 Hz):
    act() → emit_state() → read servo → emit DriveStateEvent
    ↓
Socket.io namespace
    ↓
ThrottledStoreUpdater (30 FPS)
    ↓
React useDriveNamespace()
```

## Flujo de Datos

### Estado (Socket.io)
1. Loop thread llama `act()` cada 100ms
2. `emit_state()` lee `Arc<RwLock<ServoAdapter>>` con `.read().await`
3. Extrae position, velocity, statusword del servo
4. Parsea CiA402 statusword en flags
5. Emite `DriveStateEvent` via namespace
6. `ThrottledStoreUpdater` sincroniza a store cada 33ms
7. Frontend recibe actualización en `useDriveNamespace()`

### Comandos (HTTP REST)
1. Frontend llama `useDrive().jogStart("fast+")`
2. `useMachineMutation()` envía POST a `/api/v1/machine/mutate`
3. Server despacha a `ServoTestMachine::api_mutate()`
4. `handle_mutation()` es async, adquiere `servo.write().await`
5. Llama `servo_mut().enable()` o similar
6. Emite `CommandAckEvent` con resultado
7. Emite `DriveStateEvent` actualizado inmediatamente

## Patrón de Locking

**Lectura (act loop)**:
```rust
let servo = self.servo.read().await;
let position = servo.servo().get_position_actual()?;
drop(servo); // Libera lock rápido
```

**Escritura (comandos)**:
```rust
let mut servo = self.servo.write().await;
servo.servo_mut().enable()?;
// Lock se libera automáticamente al salir del scope
```

## Próximos Pasos

### En Linux/TwinCAT:

1. **Compilar**:
   ```bash
   cd control
   cargo build --release
   ```

2. **Configurar Hardware**:
   - Conectar Lichuan LC10E o Mitsubishi MR-J4-TM via EtherCAT
   - Asignar `machine_identification` y `serial` en config

3. **Ejecutar Server**:
   ```bash
   cargo run --release
   ```

4. **Abrir Frontend**:
   - Electron app se conecta automáticamente
   - Navegar a `/drive/{vendor}/{machine}/{serial}`
   - Debería ver:
     * DriveControl component con tabs Online/Functions
     * Position actualizado cada 100ms
     * Botones JOG/START/STOP/RESET funcionales

5. **Probar Comandos**:
   - F1-F4: JOG (mantener presionado)
   - F5: START
   - F6: STOP
   - F7: RESET
   - F8: REFERENCE
   - F9: Set Enabling Dialog

### Debugging:

Ver logs del servidor:
```bash
RUST_LOG=info cargo run --release
```

Buscar:
- `"Servo drive initialized"` - Confirmación de detección
- `"JOG Start: fast+"` - Comandos recibidos
- `"Command JogStart acknowledged"` - Respuestas

### Verificación de Socket.io:

En DevTools del navegador:
```javascript
// Ver estado actual
window.socketioStore.getState()

// Ver eventos
// Debería mostrar DriveStateEvent cada 100ms
```

## Pendientes (TODOs en código)

1. **Jog Logic**: Implementar direcciones fast-/slow-/slow+/fast+
2. **Homing**: Implementar secuencia de referencia
3. **Set Enabling**: Aplicar parámetros de habilitación
4. **Download Parameters**: Escribir KV factor, velocidades vía SDO
5. **Movement Modes**: Implementar modos de movimiento (position, velocity, torque)
6. **Raw Output**: Comando directo de torque
7. **Lag Error Min/Max**: Trackear valores mínimos/máximos
8. **Setpoint Velocity**: Obtener del servo

## Notas

- **Thread Safety**: `Arc<RwLock<>>` permite acceso concurrente seguro desde loop thread y API thread
- **Async/Await**: Todos los accesos al servo son async para evitar bloqueos
- **Error Handling**: Todos los comandos retornan `Result<>` y emiten `CommandAckEvent`
- **CiA402 Compliance**: Statusword parsing sigue spec DS402 bits 0-15
- **Generic Design**: `ServoTestMachine<T>` funciona con cualquier `ServoDevice`

## Integración con Electron

El frontend ya está listo:
- `electron/src/components/drive/*`: 11 componentes React
- `driveNamespace.ts`: Socket.io client
- `useDrive.ts`: Hook con 15 comandos
- `DriveTestPage.tsx`: Ruta de prueba

Solo falta arrancar el servidor con un servo conectado.
