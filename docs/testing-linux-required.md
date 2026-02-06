# Testing Guide - Linux Required

## ⚠️ Important Notice

The UML StateChart feature **requires Linux** for testing debido a las dependencias de EtherCAT:

- `ethercrab` - EtherCAT master library (Linux-only)
- `interfaces` - Network interface management (requires `sys/ioctl.h`)
- `ethercat_hal` - Hardware abstraction layer (Linux-only)

## Windows Development

Durante desarrollo en Windows, puedes trabajar en:
- ✅ Frontend (Electron + React) - funciona completamente
- ✅ Diseño de state machines - exportar/importar JSON
- ✅ Documentación y tests unitarios

**NO puedes**:
- ❌ Compilar el servidor (requiere dependencias Linux)
- ❌ Probar el WebSocket real
- ❌ Ejecutar state machines en el backend

## Requisitos para Testing Completo

### Sistema Linux (Ubuntu/Debian recomendado)

```bash
# Instalar dependencias
sudo apt-get update
sudo apt-get install build-essential pkg-config libclang-dev

# Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clonar el repo
git clone <repo-url>
cd control
```

### Compilar y Ejecutar

```bash
# Terminal 1: Backend
cd server
cargo run --release

# Debería ver:
# 🚀 Server listening on http://0.0.0.0:3000
# 📡 WebSocket available at ws://localhost:3000/socket.io/
# ✅ StateMachine namespace registered at /statechart
```

```bash
# Terminal 2: Frontend (en el mismo Linux o via red desde Windows)
cd electron
npm install
npm start
```

### Probar el Sistema

1. En Electron, navega a `/statechart`
2. Importa `motor-control-hardware.json` desde examples/
3. Click "Run in Backend"
4. Observa los logs en la terminal del servidor:
   ```
   [StateMachine Hardware] disableMotor → Output 0 = false
   [StateMachine Hardware] setRedLight → Output 1 = true
   ```
5. Haz click en los botones de eventos: [START], [SAFETY_OK], etc.
6. Observa las transiciones en tiempo real

## Alternativas para Windows

### Opción 1: WSL2 (Windows Subsystem for Linux)

```bash
# Instalar WSL2 con Ubuntu
wsl --install -d Ubuntu

# Dentro de WSL
wsl
cd /mnt/e/_Rust-Proyectos/control
cargo run
```

**Ventajas**:
- ✅ Ejecutas Linux dentro de Windows
- ✅ Acceso a archivos de Windows
- ✅ Puedes probar todo

**Desventajas**:
- ⚠️ Performance ligeramente inferior
- ⚠️ No recomendado para hardware real EtherCAT

### Opción 2: Máquina Virtual

Usa VirtualBox o VMware con Ubuntu:

```bash
# En la VM Ubuntu
git clone <repo>
cd control
# ... seguir pasos normales de Linux
```

### Opción 3: Servidor Linux Remoto

Si tienes acceso a un servidor Linux:

```bash
# En el servidor Linux
cd control/server
cargo run --release -- --host 0.0.0.0

# En Windows, en electron/.env
VITE_API_URL=http://<server-ip>:3000
```

Luego ejecuta el frontend desde Windows conectándose al backend remoto.

## Flujo de Trabajo Recomendado

### Durante Desarrollo (Windows)

1. Diseña state machines en el editor visual
2. Exporta a JSON
3. Revisa documentación
4. Escribe tests unitarios del frontend

### Para Testing (Linux - una vez por semana)

1. Haz commit de tus cambios
2. Conéctate al sistema Linux (WSL2, VM, servidor)
3. `git pull origin UML-StateChart`
4. Compila y ejecuta:
   ```bash
   cargo run
   ```
5. Prueba todas las funcionalidades
6. Documenta bugs encontrados
7. Vuelve a Windows para corregirlos

## Testing en Linux sin Hardware

Aunque no tengas hardware EtherCAT real, puedes:

- ✅ Probar el WebSocket frontend ↔ backend
- ✅ Verificar que el state machine ejecuta correctamente
- ✅ Ver los logs de las acciones (simuladas)
- ✅ Validar guards y transiciones
- ✅ Probar manejo de errores

Las acciones harán logging pero NO ejecutarán I/O real:

```rust
// Esto funciona sin hardware
tracing::info!("[StateMachine Hardware] disableMotor → Output 0 = false");

// Esto requeriría hardware EtherCAT (comentado por ahora)
// ethercat.devices[0].set_output(0, false)?;
```

## Integración con Hardware Real (Futuro)

Cuando tengas hardware EtherCAT conectado, descomenta las líneas en:

[hardware_actions.rs](../server/src/socketio/statechart_namespace/hardware_actions.rs):

```rust
impl Action for SetDigitalOutputAction {
    fn execute(&self, context: &mut MachineContext) -> Result<()> {
        // ... logging ...
        
        // Descomentar estas líneas:
        // let ethercat = app_state.ethercat.lock().await;
        // ethercat.devices[0].set_output(self.output_id, self.value)?;
        
        Ok(())
    }
}
```

## Resumen

| Tarea | Windows | Linux WSL2 | Linux Nativo |
|-------|---------|------------|--------------|
| Editor visual | ✅ | ✅ | ✅ |
| Diseño de máquinas | ✅ | ✅ | ✅ |
| Exportar JSON | ✅ | ✅ | ✅ |
| Compilar servidor | ❌ | ✅ | ✅ |
| Testing WebSocket | ❌ | ✅ | ✅ |
| Ejecutar state machines | ❌ | ✅ | ✅ |
| Hardware EtherCAT | ❌ | ⚠️ | ✅ |

**Recomendación**: Usa WSL2 para desarrollo completo en Windows, o reserva una máquina Linux para testing semanal.
