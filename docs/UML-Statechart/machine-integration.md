# Integración State Machines con Hardware EtherCAT

## Arquitectura

### Antes (Simulación Global)
```
Frontend → /statechart → Backend StateChartRoom (global)
```

### Después (Por Máquina Real)
```
Frontend → /machine/{vendor}/{machine}/{serial}/statechart → Backend StateChartRoom (por máquina)
```

## Cambios Implementados

### ✅ 1. Backend - NamespaceId
Se agregó el tipo `MachineStateChart` al enum `NamespaceId`:

```rust
pub enum NamespaceId {
    Main,                                          // Namespace global
    Machine(MachineIdentificationUnique),          // Datos de máquina
    StateChart,                                    // Global (deprecar)
    MachineStateChart(MachineIdentificationUnique), // State chart por máquina
}
```

**Formato de Path:**
- Global: `/statechart` 
- Por máquina: `/machine/{vendor}/{machine}/{serial}/statechart`
  - Ejemplo: `/machine/1/54/1/statechart`

## Cambios Pendientes

### 🔨 2. Backend - Namespaces Structure

**Archivo:** `server/src/socketio/namespaces.rs`

Cambiar de:
```rust
pub struct Namespaces {
    pub main_namespace: MainRoom,
    pub statechart_namespace: StateChartRoom,  // Global - DEPRECAR
    pub machine_namespaces: HashMap<NamespaceId, Namespace>,
}
```

A:
```rust
pub struct Namespaces {
    pub main_namespace: MainRoom,
    pub machine_namespaces: HashMap<NamespaceId, Namespace>,
    pub machine_statechart_rooms: HashMap<MachineIdentificationUnique, StateChartRoom>,
}
```

### 🔨 3. Backend - Init Namespace per Machine

**Archivo:** `server/src/socketio/init.rs`

En lugar de inicializar un namespace `/statechart` global, crear namespace dinámicamente cuando una máquina se conecta:

```rust
// Para cada máquina en el sistema:
let machine_id = machine.get_machine_identification_unique();
let namespace_path = format!(
    "/machine/{}/{}/{}/statechart",
    machine_id.machine_identification.vendor,
    machine_id.machine_identification.machine,
    machine_id.serial
);

io.ns(namespace_path, move |socket: SocketRef| {
    // Manejar connections del statechart de esta máquina específica
});
```

### 🔨 4. Frontend - Selector de Máquina

**Archivo:** `electron/src/statechart/StateChartEditor.tsx`

Agregar un selector para elegir la máquina:

```tsx
const [selectedMachine, setSelectedMachine] = useState<MachineIdentificationUnique | null>(null);
const machines = useMainNamespace().machines?.data?.machines || [];

// Conectar al namespace correcto
const machineNamespaceId = selectedMachine 
  ? { type: "machine-statechart", machine_identification_unique: selectedMachine }
  : { type: "statechart" }; // Fallback al global

const { loadMachine, sendEvent, ... } = useStateMachineSocket(machineNamespaceId);
```

### 🔨 5. Frontend - NamespaceId Type

**Archivo:** `electron/src/client/socketioStore.ts`

```typescript
export type NamespaceId =
  | { type: "main" }
  | { type: "statechart" }  // Deprecar
  | { type: "machine"; machine_identification_unique: MachineIdentificationUnique }
  | { type: "machine-statechart"; machine_identification_unique: MachineIdentificationUnique };

function serializeNamespaceId(id: NamespaceId): string {
  if (id.type === "machine-statechart") {
    return `/machine/${id.machine_identification_unique.machine_identification.vendor}/${id.machine_identification_unique.machine_identification.machine}/${id.machine_identification_unique.serial}/statechart`;
  }
  // ... otros casos
}
```

## Flujo de Trabajo

### Modo Simulación (actual):
1. Backend con `--features mock-machine`
2. Frontend conecta a `/statechart` global
3. Una state machine para pruebas

### Modo Producción (objetivo):
1. Backend **sin** `--features mock-machine`
2. EtherCAT detecta máquinas físicas
3. Frontend muestra lista de máquinas disponibles
4. Usuario selecciona máquina
5. Frontend conecta a `/machine/{vendor}/{machine}/{serial}/statechart`
6. State machine controla hardware real de esa máquina específica

## Ejecutar con Hardware Real

```bash
# 1. Compilar sin mock
cd /home/runtimevic/Descargas/control
cargo build --release

# 2. Ejecutar con privilegios (EtherCAT requiere root)
sudo ./target/release/server

# 3. Frontend se conecta automáticamente
# Verás las máquinas reales en el selector
```

## Próximos Pasos

1. ☑️ Actualizar `namespaces.rs` para statechart por máquina
2. ☑️ Modificar `init.rs` para crear namespaces dinámicamente
3. ☑️ Agregar selector de máquina en StateChartEditor
4. ☑️ Actualizar NamespaceId en frontend
5. ☑️ Modificar useStateMachineSocket para aceptar machineId
6. ☑️ Probar con hardware real EtherCAT

## Beneficios

- ✅ **Escalable:** Múltiples máquinas pueden tener diferentes state machines simultáneamente
- ✅ **Aislado:** State machine de una máquina no afecta otras
- ✅ **Hardware Real:** Acciones ejecutan sobre hardware EtherCAT específico
- ✅ **Desarrollo:** Mantiene modo mock para pruebas sin hardware

