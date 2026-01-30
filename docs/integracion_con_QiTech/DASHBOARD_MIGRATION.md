# Drive Control Component Migration

## Overview

Se migró exitosamente el dashboard de control de servos/drives de TwinSharp (.NET WinForms) a React con Shadcn UI para el proyecto control.

## Ubicación

`electron/src/components/drive/`

## Componentes Creados

### 1. DriveControl.tsx (Principal)
- Componente principal que replica `NcAxis` del .NET
- **Estructura**: TabControl con tabs "Online" y "Functions" como en el original
- Incluye DriveAxisHeader para mostrar posición y estado
- Integra DriveStatusPanel y JogControls en el tab "Online"
- Integra DriveFunctionsPanel en el tab "Functions"

### 2. DriveAxisHeader.tsx
- Replica `NcAxisHeader` del .NET (464x48px)
- **BarberPole** animado indicando movimiento del eje (36x36px)
- Display de posición actual (grande, font 16pt equivalente)
- Display de setpoint position (pequeño, 150px)

### 3. BarberPole.tsx
- Componente animado con rayas diagonales
- Indica visualmente cuando el eje está en movimiento
- Animación con SVG pattern rotado 45°
- Color azul cuando activo, gris cuando inactivo

### 4. DriveStatusPanel.tsx
- Panel izquierdo del tab "Online" - replica `NcOnline` (498x427px)
- **GroupBox "Status (logical)"**: 6 checkboxes (Ready, Calibrated, HasJob, NotMoving, MovingFw, MovingBw)
- **GroupBox "Status (physical)"**: 3 checkboxes (CoupledMode, InTargetPos, InPosRange)
- **GroupBox "Enabling"**: 3 checkboxes (Controller, FeedFw, FeedBw) + botón "Set..."
- **Sección Configuration**: KvFactor, RefVelocity, TargetPosition con botones de descarga "↓"
- Displays de estado: LagError, ActualVelocity, SetpointVelocity, Error, Output, Override

### 5. JogControls.tsx
- Botones de movimiento JOG y control
- **4 botones JOG** (52x52px): `--`, `-`, `+`, `++` con gradiente naranja (#FFA500→#FF8C00)
  - F1-F4 para keyboard shortcuts
- **4 botones de control** (52x52px):
  - **Start** (F5): Verde (#32CD32→#228B22)
  - **Stop** (F6): Rojo (#FF6347→#FF0000)
  - **Reset** (F8): Azul (#1E90FF→#4682B4)
  - **Reference** (F9): Azul (#1E90FF→#4682B4)
- Espaciado entre Stop y Reset como en el original

### 6. DriveFunctionsPanel.tsx
- Panel del tab "Functions" - replica `NcFunctions` (498x513px)
- **GroupBox "Extended Start"**: 
  - Selector de StartMode (23 modos: Absolute, Relative, Jog, etc.)
  - Parámetros dinámicos: TargetPosition, TargetVelocity, Acceleration, Deceleration, Jerk
  - Checkboxes para habilitar parámetros opcionales
  - Botones Start/Stop
  - Display "Last Time"
- **GroupBox "Raw Drive Output"**:
  - Mode dropdown (Torque, Velocity, Position)
  - Value input
  - Start/Stop buttons
- **GroupBox "Set Actual Position"**:
  - Type dropdown (Absolute, Relative)
  - Value input
  - Set button
- **GroupBox "Set Target Position"**:
  - Type dropdown (Absolute, Relative, Home)
  - Value input
  - Set button

### 7. types.ts
- Definiciones TypeScript para todo el sistema
- **DriveState**: Estado completo del eje (posición, velocidad, flags, errores)
- **DriveControlParams**: Parámetros de movimiento
- **MovementMode**: 23 modos de movimiento CiA402
- **RawDriveOutputMode**, **SetPositionMode**, **SetTargetMode**

### 8. SetEnablingDialog.tsx ✅ NUEVO
- Modal para configurar enabling del controlador
- 3 checkboxes: Controller, Feed Forward, Feed Backward
- Input para Override (0-100%)
- Botón "All" para habilitar todo al 100%
- Validación de rango 0.00-100.00

### 9. useDriveConnection.ts ~~DEPRECADO~~
- ~~Custom hook para manejar conexión con backend~~
- ~~Polling de estado cada 100ms (configurable)~~
- Reemplazado por Socket.io (ver abajo)

### 10. driveNamespace.ts ✅ NUEVO (Socket.io)
- Namespace Socket.io para comunicación en tiempo real
- Recibe eventos `DriveStateEvent` cada 100ms desde backend
- Eventos `CommandAckEvent` para confirmación de comandos
- Eventos `MovementCompleteEvent` cuando termina un movimiento
- ThrottledStoreUpdater limita re-renders a ~30 FPS
- Sigue el mismo patrón que winder2Namespace, extruder2Namespace, etc.

### 11. useDrive.ts ✅ NUEVO
- Custom hook que combina namespace + emit
- Métodos tipados para todos los comandos (jogStart, start, stop, etc.)
- useCallback para optimización de rendimiento
- Sigue el patrón de useWinder2, useExtruder2, etc.

## Funcionalidades Implementadas

### ✅ Keyboard Shortcuts (F1-F9)
- **F1**: JOG -- (Fast backward)
- **F2**: JOG - (Slow backward)
- **F3**: JOG + (Slow forward)
- **F4**: JOG ++ (Fast forward)
- **F5**: Start
- **F6**: Stop
- **F8**: Reset
- **F9**: Reference/Home
- **Auto-stop JOG**: Al soltar teclas F1-F4 se detiene el JOG automáticamente
- **Prevención de default**: Las teclas F no ejecutan acciones del navegador

### ✅ Set Enabling Dialog
- Modal con 3 checkboxes + override input
- Botón "All" para habilitar todo
- Actualiza estado local del componente
- Llama callback `onCommand('set_enabling', {...})`

### ✅ Timer de Actualización (Socket.io Push)
- **Socket.io con eventos en tiempo real** (no polling)
- Backend envía `DriveStateEvent` cada 100ms
- ThrottledStoreUpdater limita re-renders a ~30 FPS (~33ms)
- Menor latencia y mejor performance que polling HTTP
- Reconexion automática en caso de pérdida de conexión
- Sigue el patrón establecido en el proyecto (winder, extruder, buffer, laser, mock)

## Ruta de Prueba

Se agregó una ruta de test en `/machines/drivetest`:

**Archivo**: `electron/src/components/drive/DriveTestPage.tsx`

**Acceso**: Navegar a la aplicación Electron → Machines → Drive Test

La página muestra el componente DriveControl con datos mock y console.log de los comandos.

## Colores y Tamaños

### Botones JOG (Orange)
```css
from-orange-500 to-orange-600
hover:from-orange-600 hover:to-orange-700
52px × 52px
```

### Botón START (Green)
```css
from-green-500 to-green-700
hover:from-green-600 hover:to-green-800
52px × 52px
```

### Botón STOP (Red)
```css
from-red-500 to-red-700
hover:from-red-600 hover:to-red-800
52px × 52px
```

### Botones RESET/REFERENCE (Blue)
```css
from-blue-500 to-blue-700
hover:from-blue-600 hover:to-blue-800
52px × 52px
```

## Fuentes

- **Bahnschrift 12pt** del .NET → Se usa el sistema de fonts de Tailwind CSS
- Displays de posición: `font-mono font-semibold`
- Labels: `text-xs` o `text-sm` según contexto

## Tamaños del .NET Original

| Componente | Tamaño .NET |
|-----------|------------|
| NcAxis (completo) | 516×539px |
| NcOnline | 498×427px |
| NcFunctions | 498×513px |
| NcAxisHeader | 464×48px |
| BarberPole | 36×36px |
| Botones JOG/Control | 52×52px |
| Actual Position TextBox | 264×36px (Font 16pt) |
| Setpoint Position TextBox | 150×23px |

## Keyboard Shortcuts (Pendiente implementar)

- F1: JOG --
- F2: JOG -
- F3: JOG +
- F4: JOG ++
- F5: Start
- F6: Stop
- F7: (no usado en .NET)
- F8: Reset
- F9: Reference

## Pendiente

1. ~~**Set Enabling Dialog**~~: ✅ **COMPLETADO**
2. ~~**Keyboard shortcuts**~~: ✅ **COMPLETADO** 
3. ~~**Timer Update**~~: ✅ **COMPLETADO** (Socket.io push)
4. **Integración con backend Rust**: Implementar namespace Socket.io en `server/`
5. **Estados dinámicos**: Backend envía DriveStateEvent cada 100ms
6. **Download buttons**: Enviar eventos de configuración
7. **Movement start con parámetros**: Evento `start_movement` con modo + parámetros
8. **Raw Output control**: Eventos `start_raw_output` / `stop_raw_output`
9. **Set Position logic**: Eventos `set_actual_position` / `set_target_position`

## Ventajas de Socket.io vs Polling HTTP

✅ **Menor latencia**: Server push en lugar de client pull
✅ **Mejor performance**: Sin overhead de HTTP request/response
✅ **Menor carga en server**: Una conexión persistente vs múltiples requests
✅ **Reconexion automática**: Socket.io maneja reconexiones
✅ **Throttling inteligente**: ThrottledStoreUpdater limita re-renders a 30 FPS
✅ **Consistencia**: Todo el proyecto usa este patrón (winder, extruder, buffer, laser, mock)
✅ **Bi-direccional**: Server puede notificar eventos (errores, completions) sin polling

## Integración con Rust Backend

El backend en `server/` ya tiene:
- `ServoAdapter<T>` genérico para CiA402
- Lichuan LC10E (0x00000766:0x00000402)
- SMC-Mitsubishi MR-J4-TM (0x00000A1E:0x00000201)

**Arquitectura de comunicación**: Socket.io (tiempo real)

### Socket.io Events (Server → Client)

#### DriveStateEvent (100ms)
Enviado cada 100ms con el estado completo del servo:
```rust
// Rust backend
struct DriveStateEvent {
    position: f64,
    setpoint_position: f64,
    velocity: f64,
    error_code: u16,
    // ... resto de campos
}

// Enviar cada 100ms
namespace.emit("DriveStateEvent", state).await;
```

#### CommandAckEvent
Confirmación de comandos recibidos:
```rust
struct CommandAckEvent {
    command: String,
    success: bool,
    error_message: Option<String>,
}
```

#### MovementCompleteEvent
Notificación cuando termina un movimiento:
```rust
struct MovementCompleteEvent {
    final_position: f64,
    time_ms: u64,
}
```

### Socket.io Events (Client → Server)

Todos los comandos se envían como eventos Socket.io:

```typescript
// Frontend
const { emit } = useSocketEmit(machine_id);

// JOG
emit('jog_start', { direction: 'fast+' });
emit('jog_stop', {});

// Movement
emit('start_movement', { 
  mode: 'Absolute', 
  targetPosition: 100.0,
  targetVelocity: 50.0,
  // ...
});

// Configuration
emit('set_enabling', { 
  controller: true, 
  feedFw: true, 
  feedBw: true, 
  override: 10000 
});
```

### Implementación en Rust

```rust
// server/src/machines/servo/mod.rs

use socketio::Namespace;
use tokio::time::{interval, Duration};

pub async fn setup_servo_namespace(
    namespace: &Namespace,
    servo: Arc<RwLock<ServoAdapter<LichuanServo>>>,
) {
    // Update timer (100ms)
    let servo_clone = servo.clone();
    let namespace_clone = namespace.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_millis(100));
        loop {
            interval.tick().await;
            let state = servo_clone.read().await.get_state();
            namespace_clone.emit("DriveStateEvent", state).await;
        }
    });
    
    // Command handlers
    namespace.on("jog_start", |socket, data| {
        // Handle JOG start
    });
    
    namespace.on("start_movement", |socket, data| {
        // Handle movement start
    });
    
    // ... resto de handlers
}
```

## Referencias

- TwinSharp .NET source: `docs/TwinSharp/*.cs`
- rust-ethercat-devices: GitHub integration completada en Sprint 1 & 2
- CiA402 protocol: Standard para control de servos
