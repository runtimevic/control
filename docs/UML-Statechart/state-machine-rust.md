# State Machine Interpreter - Rust Implementation

Intérprete completo de máquinas de estados compatible con formato JSON de XState.

## Características

- ✅ **Parseo JSON XState** - Compatible con el formato estándar
- ✅ **Estados con Entry/Exit Actions** - Acciones al entrar/salir de estados
- ✅ **Transiciones con Guards** - Condiciones para permitir transiciones
- ✅ **Transiciones con Actions** - Acciones durante transiciones
- ✅ **Estados Finales** - Detección de estados terminales
- ✅ **Contexto de Ejecución** - Variables runtime (bool, int, float, string)
- ✅ **Registro de Actions/Guards** - Sistema extensible y personalizable
- ✅ **Cola de Eventos** - Procesamiento asíncrono de eventos
- ✅ **Thread-Safe** - Actions y Guards son Send + Sync

## Ubicación

```
control-core/src/state_machine/
├── mod.rs          - Módulo principal, re-exports
├── types.rs        - Tipos y estructuras
├── actions.rs      - Sistema de actions y guards
└── machine.rs      - Ejecutor de la máquina de estados
```

## Uso Básico

### 1. Crear desde JSON

```rust
use control_core::state_machine::StateMachine;

let json = r#"
{
    "id": "my-machine",
    "initial": "idle",
    "states": {
        "idle": {
            "on": { "START": "running" }
        },
        "running": {
            "on": { "STOP": "idle" }
        }
    }
}
"#;

let mut machine = StateMachine::from_json(json)?;
```

### 2. Registrar Actions Personalizadas

```rust
use control_core::state_machine::{Action, MachineContext};
use std::sync::Arc;

struct MyAction;

impl Action for MyAction {
    fn execute(&self, context: &mut MachineContext) -> anyhow::Result<()> {
        println!("Executing custom action");
        context.set_bool("action_executed", true);
        Ok(())
    }

    fn name(&self) -> &str {
        "myAction"
    }
}

// Registrar
machine.actions_mut().register(Arc::new(MyAction));
```

### 3. Registrar Guards Personalizados

```rust
use control_core::state_machine::{Guard, MachineContext};
use std::sync::Arc;

struct SafetyGuard;

impl Guard for SafetyGuard {
    fn evaluate(&self, context: &MachineContext) -> anyhow::Result<bool> {
        Ok(!context.get_bool("emergency_stop").unwrap_or(false))
    }

    fn name(&self) -> &str {
        "safetyOk"
    }
}

// Registrar
machine.guards_mut().register(Arc::new(SafetyGuard));
```

### 4. Enviar Eventos

```rust
// Enviar evento directamente
machine.send("START")?;

// Verificar estado actual
assert_eq!(machine.current_state(), "running");

// Enviar múltiples eventos
machine.send("STOP")?;
machine.send("START")?;
```

### 5. Usar Contexto

```rust
// Establecer valores
machine.context_mut().set_bool("ready", true);
machine.context_mut().set_int("count", 42);
machine.context_mut().set_float("speed", 3.14);
machine.context_mut().set_string("status", "OK".to_string());

// Leer valores
if machine.context().get_bool("ready").unwrap_or(false) {
    println!("System is ready");
}
```

### 6. Estado de Ejecución

```rust
let state = machine.execution_state();
println!("Current: {}", state.current_state);
println!("Previous: {:?}", state.previous_state);
println!("Available events: {:?}", state.available_events);
println!("Timestamp: {}", state.timestamp);
```

## Formato JSON

### Transición Simple

```json
{
    "idle": {
        "on": {
            "START": "running"
        }
    }
}
```

### Transición Completa (con guard y actions)

```json
{
    "idle": {
        "on": {
            "START": {
                "target": "running",
                "guard": "safetyOk",
                "actions": ["logStart", "enableMotor"]
            }
        }
    }
}
```

### Entry/Exit Actions

```json
{
    "running": {
        "entry": ["logEntry", "startTimer"],
        "exit": ["logExit", "stopTimer"],
        "on": {
            "STOP": "idle"
        }
    }
}
```

### Estado Final

```json
{
    "done": {
        "type": "final"
    }
}
```

## Ejemplo Completo: Control de Motor

Ver: `control-core/examples/state_machine_demo.rs`

```bash
cargo run --example state_machine_demo
```

## Integración con Server

El módulo está diseñado para integrarse con el servidor Socket.IO:

1. **Envío de JSON desde frontend**
2. **Creación de StateMachine en backend**
3. **Registro de actions/guards específicos del hardware**
4. **Procesamiento de eventos desde WebSocket**
5. **Envío de ExecutionState al frontend para visualización**

## Tests

```bash
# Tests unitarios
cargo test --lib state_machine

# Test standalone de parsing
cargo run --bin state_machine_test
```

## Dependencias Añadidas

```toml
[dependencies]
serde = "1.0"
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
```

## Roadmap

- [x] Parser JSON XState
- [x] Ejecutor de máquinas de estados
- [x] Actions y Guards personalizables
- [x] Contexto de ejecución
- [x] Estados finales
- [ ] Estados compuestos (sub-máquinas)
- [ ] Historial de transiciones
- [ ] Persistencia de estado
- [ ] Actions/Guards pre-definidos comunes
- [ ] Integración con Socket.IO en server
- [ ] Métricas y observabilidad

## Referencias

- [XState Documentation](https://xstate.js.org/docs/)
- [State Charts (David Harel)](https://www.sciencedirect.com/science/article/pii/0167642387900359)
