# UML StateChart Editor

Editor visual para diseñar máquinas de estados que pueden ser ejecutadas en el backend Rust.

## Características

- ✨ Editor gráfico interactivo usando React Flow
- 🎨 Diseño visual de estados y transiciones
- 📋 Panel de propiedades para configurar estados y transiciones
- 💾 Exportación/importación en formato JSON compatible con XState
- 🚀 Integración con backend Rust (preparado para ejecución)

## Componentes

### `StateChartEditor.tsx`
Componente principal del editor que integra:
- Canvas de React Flow con drag & drop
- Toolbar con acciones (agregar estado, eliminar, exportar, etc.)
- Panel de propiedades lateral

### `StateNode.tsx`
Componente visual para nodos de estado con:
- Estados normales, iniciales, finales y compuestos
- Visualización de entry/exit actions
- Estilos diferenciados por tipo

### `PropertiesPanel.tsx`
Panel lateral para editar:
- **Estados**: Label, tipo, descripción, entry/exit actions
- **Transiciones**: Event, guard, actions, descripción

### `useStateChart.ts`
Hook personalizado que gestiona:
- Nodos y edges del diagrama
- Exportación a formato XState JSON
- Importación desde JSON
- Operaciones de edición (agregar, actualizar, eliminar)

## Tipos de Estados

1. **Normal** (gris): Estado estándar
2. **Initial** (verde): Estado inicial de la máquina
3. **Final** (rojo): Estado terminal
4. **Compound** (azul): Estado que puede contener sub-estados

## Uso

### Acceder al editor

Navega a `/statechart` en la aplicación.

### Crear un StateChart

1. Click en "Add State" para agregar nuevos estados
2. Arrastra desde el punto inferior de un estado para crear transiciones
3. Click en un estado o transición para editar sus propiedades en el panel derecho
4. Configura entry/exit actions, guards y eventos

### Exportar

- **Export JSON**: Descarga el StateChart como archivo JSON
- **Save**: Guarda en el estado actual (preparado para backend)
- **Run in Backend**: Envía el JSON al servidor Rust para ejecución

### Ejemplos Incluidos

#### Traffic Light (traffic-light.json)
Estado simple de semáforo con tres estados:
- Red → TIMER → Green
- Green → TIMER → Yellow
- Yellow → TIMER → Red

#### Motor Control (motor-control.json)
Sistema básico de control de motor con estados de seguridad.

#### Motor Control Hardware (motor-control-hardware.json)
**🔥 Ejemplo completo con integración de hardware**

Máquina de estados completa para control de motor con:
- ✅ 6 estados: Stopped, CheckingSafety, Starting, Running, Stopping, Error
- ✅ Verificaciones de seguridad (emergency stop, door interlock)
- ✅ Control de outputs (motor enable, traffic lights, alarm)
- ✅ Guards para condiciones de hardware (speed sensors, safety inputs)
- ✅ Manejo robusto de errores y recuperación

**Ver documentación completa:**
- [Hardware Example Guide](../../../docs/statechart-hardware-example.md)
- [Testing Guide](../../../docs/statechart-testing-guide.md)

### Formato JSON (XState)

```json
{
  "id": "machine",
  "initial": "Idle",
  "states": {
    "Idle": {
      "entry": ["logEntry"],
      "on": {
        "START": {
          "target": "Running",
          "guard": "isReady",
          "actions": ["startMotor"]
        }
      }
    },
    "Running": {
      "on": {
        "STOP": "Idle"
      },
      "type": "normal"
    }
  }
}
```

## Integración con Backend

### ✅ Implementado

El backend Rust está completamente integrado:

1. **Namespace `/statechart`** - Socket.IO para comunicación bidireccional
2. **StateMachine Interpreter** - Ejecutor XState-compatible en Rust
3. *Uso en Producción

```rust
// Backend: server/src/socketio/statechart_namespace/
pub struct StateChartRoom {
    machines: Arc<Mutex<HashMap<String, StateMachine>>>,
}

impl StateChartRoom {
    pub async fn on_load_state_machine(...) {
        let machine = StateMachine::from_json(&config)?;
        hardware_actions::register_actions(machine.actions_mut());
        hardware_actions::register_guards(machine.guards_mut());
        // ... store and emit initial state
    }
}
```

```typescript
// Frontend: hooks/useStateMachineSocket.ts
const { loadMachine, sendEvent, currentState, availableEvents } = useStateMachineSocket();

// Load machine
loadMachine(xstateConfig);

// Send event
sendEvent('START');

// Observe state changes
useEffect(() => {
  console.log('Current state:', currentState);
}, [currentState]);
```

###x] Editor visual con React Flow
- [x] Panel de propiedades para configuración
- [x] Exportación/importación JSON XState
- [x] Implementar intérprete XState en Rust
- [x] WebSocket para comunicación bidireccional
- [x] Visualización de ejecución en tiempo real
- [x] Actions/Guards personalizadas pre-definidas
- [x] Ejemplo completo con hardware
- [ ] Estados compuestos con sub-máquinas
- [ ] Historial de estados y debugging
- [ ] Plantillas de máquinas comunes (Start/Stop, secuencias, etc.)
- [ ] Persistencia de state machines
- [ ] Simulador de hardware virtual
- [ ] Métricas y observabilidad

## Testing

### Quick Start
```bash
# Terminal 1: Backend
cd server && cargo run

# Terminal 2: Frontend  
cd electron && npm start

# Navigate to /statechart
# Import motor-control-hardware.json
# Click "Run in Backend"
# Click event buttons to trigger transitions
```

Ver [Testing Guide](../../../docs/statechart-testing-guide.md) para tests completos.
// En control-core
pub struct StateMachine {
    current_state: String,
    states: HashMap<String, StateConfig>,
}

pub struct ExecutionState {
    current_state: String,
    available_events: Vec<String>,
    timestamp: u64,
}
```

### Comunicación

```typescript
// Frontend envía StateChart JSON
socket.emit('loadStateMachine', xstateConfig);

// Backend envía estado de ejecución
socket.on('executionState', (state: ExecutionState) => {
  // Actualizar visualización
});
```

## Roadmap

- [ ] Implementar intérprete XState en Rust
- [ ] WebSocket para comunicación bidireccional
- [ ] Visualización de ejecución en tiempo real
- [ ] Estados compuestos con sub-máquinas
- [ ] Historial de estados y debugging
- [ ] Guards y actions personalizadas pre-definidas
- [ ] Plantillas de máquinas comunes (Start/Stop, secuencias, etc.)

## Referencias

- [XState Documentation](https://xstate.js.org/)
- [React Flow Documentation](https://reactflow.dev/)
- [TwinCAT UML StateChart](https://infosys.beckhoff.com/content/1033/tf1910_tc3_uml/)
