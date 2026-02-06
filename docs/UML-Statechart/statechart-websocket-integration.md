# WebSocket Integration: StateChart Real-Time Visualization

## Overview

The `/statechart` Socket.IO namespace enables real-time communication between the React Flow editor (frontend) and the Rust state machine interpreter (backend).

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Frontend (Electron)                      │
│                                                                   │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  StateChartEditor.tsx                                       │ │
│  │  - Visual editor with React Flow                           │ │
│  │  - Export/Import XState JSON                               │ │
│  │  - Real-time node highlighting                             │ │
│  └────────────────────────────────────────────────────────────┘ │
│                              │                                    │
│                              ▼                                    │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  useStateMachineSocket.ts                                   │ │
│  │  - Socket.IO connection to /statechart                     │ │
│  │  - loadMachine(config)                                     │ │
│  │  - sendEvent(event)                                        │ │
│  │  - Listen to executionState updates                        │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
                    Socket.IO (/statechart)
                              │
┌─────────────────────────────────────────────────────────────────┐
│                         Backend (Rust)                           │
│                                                                   │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  StateChartRoom                                             │ │
│  │  - Manages StateMachine instances per socket               │ │
│  │  - on_load_state_machine()                                 │ │
│  │  - on_send_event()                                         │ │
│  │  - on_disconnect()                                         │ │
│  └────────────────────────────────────────────────────────────┘ │
│                              │                                    │
│                              ▼                                    │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  control_core::state_machine::StateMachine                  │ │
│  │  - Parse XState JSON                                       │ │
│  │  - Execute transitions                                     │ │
│  │  - Run actions & evaluate guards                          │ │
│  │  - Generate ExecutionState                                │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Socket.IO Events

### Client → Server

#### 1. `loadStateMachine`
Load XState JSON configuration into backend:
```typescript
socket.emit('loadStateMachine', {
  config: JSON.stringify(xstateConfig)
});
```

**Backend Response:** `loadStateMachineResponse`

#### 2. `sendEvent`
Trigger state transition by sending event:
```typescript
socket.emit('sendEvent', {
  event: 'START'
});
```

### Server → Client

#### 1. `loadStateMachineResponse`
Response after loading state machine:
```typescript
{
  success: boolean,
  message: string,
  execution_state?: ExecutionState
}
```

#### 2. `executionState`
Real-time updates when state changes:
```typescript
{
  current_state: string,
  previous_state?: string,
  available_events: string[],
  timestamp: number
}
```

## Frontend Implementation

### Hook: `useStateMachineSocket()`

```typescript
const {
  isConnected,        // Socket connection status
  loadMachine,        // fn(config: XStateConfig) => void
  sendEvent,          // fn(event: string) => void
  currentState,       // string | null
  previousState,      // string | null
  availableEvents,    // string[]
  loadError,          // string | null
} = useStateMachineSocket();
```

### Real-Time Highlighting

When `executionState` updates, the active node is highlighted:

```typescript
useEffect(() => {
  if (currentState) {
    setNodes(nodes => nodes.map(node => ({
      ...node,
      data: {
        ...node.data,
        isActive: node.data.label === currentState
      }
    })));
  }
}, [currentState]);
```

### Event Buttons

Available events are displayed as clickable buttons:

```tsx
{availableEvents.map(event => (
  <Button onClick={() => sendEvent(event)}>
    {event}
  </Button>
))}
```

## Backend Implementation

### Namespace: `/statechart`

Registered in `server/src/socketio/init.rs`:

```rust
io.ns("/statechart", move |socket: SocketRef| {
    let room = app_state.socketio_setup.namespaces
        .blocking_read()
        .statechart_namespace
        .clone();

    socket.on("loadStateMachine", /* handler */);
    socket.on("sendEvent", /* handler */);
    socket.on_disconnect(/* handler */);
});
```

### StateChartRoom

```rust
pub struct StateChartRoom {
    machines: Arc<Mutex<HashMap<String, StateMachine>>>,
}
```

- **One StateMachine per socket connection**
- Stored by `socket.id`
- Cleaned up on disconnect

### Hardware Actions

Custom actions can be registered in `hardware_actions.rs`:

```rust
impl Action for SetDigitalOutputAction {
    fn execute(&self, context: &mut MachineContext) -> Result<()> {
        // Control actual hardware here
        Ok(())
    }
}
```

## Usage Flow

1. **Design:** User creates state chart in React Flow editor
2. **Export:** Click "Run in Backend" → exports to XState JSON
3. **Load:** Frontend sends JSON via `loadStateMachine`
4. **Backend:** Parses JSON, creates StateMachine, registers actions/guards
5. **Execute:** Backend enters initial state, emits `executionState`
6. **Visualize:** Frontend highlights active node
7. **Interact:** User clicks event button (e.g., "START")
8. **Transition:** Backend processes event, changes state, emits new `executionState`
9. **Update:** Frontend updates highlighting to new active state

## Files Added

### Backend
- `server/src/socketio/statechart_namespace/mod.rs` - Namespace handlers
- `server/src/socketio/statechart_namespace/hardware_actions.rs` - Custom actions/guards
- `server/src/socketio/namespace_id.rs` - Added `StateChart` enum variant
- `server/src/socketio/namespaces.rs` - Added `statechart_namespace` field
- `server/src/socketio/init.rs` - Registered namespace handlers

### Frontend
- `electron/src/statechart/hooks/useStateMachineSocket.ts` - Socket.IO hook
- Updated `electron/src/statechart/StateChartEditor.tsx` - Integration
- Updated `electron/src/statechart/types.ts` - Added `isActive` field
- Updated `electron/src/statechart/StateNode.tsx` - Active node highlighting

## Testing

### Start Backend
```bash
cd server
cargo run
```

### Start Frontend
```bash
cd electron
npm start
```

### Test Flow
1. Navigate to `/statechart`
2. Create simple state machine (idle → running)
3. Click "Run in Backend"
4. Observe: initial state highlighted in green
5. Click available event button (e.g., "START")
6. Observe: transition to new state, highlighting updates

## Future Enhancements

- [ ] Persist state machines to database
- [ ] Multiple concurrent state machines
- [ ] History/replay of transitions
- [ ] Debug panel showing action execution
- [ ] Performance metrics per state/transition
- [ ] Integration with hardware EtherCAT outputs
- [ ] Guard conditions based on sensor inputs
