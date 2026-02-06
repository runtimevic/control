# StateChart Hardware Integration Example

## Overview

This example demonstrates a complete end-to-end state machine controlling physical hardware (motor with safety features) through the StateChart editor.

## Hardware Setup

### Required Components

1. **EtherCAT I/O Module** (e.g., Beckhoff EL2008 Digital Output)
   - Output 0: Motor Enable
   - Output 1: Red Light (Stopped)
   - Output 2: Yellow Light (Transitioning)
   - Output 3: Green Light (Running)
   - Output 4: Alarm Buzzer

2. **Inputs** (e.g., Beckhoff EL1008 Digital Input)
   - Input 0: Emergency Stop Button (NC)
   - Input 1: Door Safety Switch (NC)
   - Input 2: Motor Speed Sensor
   - Input 3: Reset Button

3. **Motor Controller** (optional)
   - Drive with EtherCAT interface (e.g., Mitsubishi MR-J4)

## State Machine Design

### States

```
┌──────────────┐
│   Stopped    │ ← Initial state, motor disabled, red light on
└──────────────┘
        │ START event
        ▼
┌──────────────┐
│CheckingSafety│ ← Verify emergency stop & door closed
└──────────────┘
        │ SAFETY_OK (guard: systemSafe)
        ▼
┌──────────────┐
│   Starting   │ ← Enable motor, yellow light, ramp up speed
└──────────────┘
        │ MOTOR_READY (guard: motorSpeedOk)
        ▼
┌──────────────┐
│   Running    │ ← Full power, green light
└──────────────┘
        │ STOP event
        ▼
┌──────────────┐
│  Stopping    │ ← Ramp down, yellow light
└──────────────┘
        │ MOTOR_STOPPED (guard: motorSpeedZero)
        ▼
┌──────────────┐
│   Stopped    │
└──────────────┘

        Any state + EMERGENCY_STOP
                ▼
        ┌──────────────┐
        │    Error     │ (final state)
        └──────────────┘
                │ RESET (guard: errorCleared)
                ▼
        ┌──────────────┐
        │   Stopped    │
        └──────────────┘
```

### Actions

| Action | Hardware Operation |
|--------|-------------------|
| `disableMotor` | Set Output 0 = LOW |
| `enableMotor` | Set Output 0 = HIGH |
| `setRedLight` | Set Output 1 = HIGH |
| `clearRedLight` | Set Output 1 = LOW |
| `setYellowLight` | Set Output 2 = HIGH |
| `clearYellowLight` | Set Output 2 = LOW |
| `setGreenLight` | Set Output 3 = HIGH |
| `clearGreenLight` | Set Output 3 = LOW |
| `triggerAlarm` | Set Output 4 = HIGH |
| `clearAlarm` | Set Output 4 = LOW |
| `rampUp` | Gradually increase motor speed |
| `rampDown` | Gradually decrease motor speed |
| `enableFullPower` | Set motor to target speed |
| `emergencyShutdown` | Immediate motor disable |

### Guards

| Guard | Condition |
|-------|-----------|
| `systemSafe` | Emergency stop released AND door closed |
| `motorSpeedOk` | Motor reached target speed (>90%) |
| `motorSpeedZero` | Motor speed < 1% of target |
| `errorCleared` | Error acknowledged AND safety conditions OK |

## Implementation

### Backend: Hardware Actions

File: `server/src/socketio/statechart_namespace/hardware_actions.rs`

```rust
pub struct SetDigitalOutputAction {
    output_id: u8,
    value: bool,
}

impl Action for SetDigitalOutputAction {
    fn execute(&self, context: &mut MachineContext) -> Result<()> {
        // Get EtherCAT device from app_state
        // ethercat.set_output(self.output_id, self.value)?;
        
        tracing::info!("Output {} = {}", self.output_id, self.value);
        context.set_bool(&format!("output_{}", self.output_id), self.value);
        Ok(())
    }
}

pub struct CheckInputGuard {
    input_id: u8,
    expected_value: bool,
}

impl Guard for CheckInputGuard {
    fn evaluate(&self, context: &MachineContext) -> Result<bool> {
        // Get EtherCAT device from app_state
        // let actual = ethercat.get_input(self.input_id)?;
        
        let actual = context
            .get_bool(&format!("input_{}", self.input_id))
            .unwrap_or(!self.expected_value);
        
        Ok(actual == self.expected_value)
    }
}
```

### Register Hardware-Specific Actions

```rust
pub fn register_actions(registry: &mut ActionRegistry) {
    // Motor control
    registry.register(Arc::new(SetDigitalOutputAction::new(0, false))); // disableMotor
    registry.register(Arc::new(SetDigitalOutputAction::new(0, true)));  // enableMotor
    
    // Lights
    registry.register(Arc::new(SetDigitalOutputAction::new(1, true)));  // setRedLight
    registry.register(Arc::new(SetDigitalOutputAction::new(1, false))); // clearRedLight
    registry.register(Arc::new(SetDigitalOutputAction::new(2, true)));  // setYellowLight
    registry.register(Arc::new(SetDigitalOutputAction::new(2, false))); // clearYellowLight
    registry.register(Arc::new(SetDigitalOutputAction::new(3, true)));  // setGreenLight
    registry.register(Arc::new(SetDigitalOutputAction::new(3, false))); // clearGreenLight
    
    // Alarm
    registry.register(Arc::new(SetDigitalOutputAction::new(4, true)));  // triggerAlarm
    registry.register(Arc::new(SetDigitalOutputAction::new(4, false))); // clearAlarm
}

pub fn register_guards(registry: &mut GuardRegistry) {
    // Safety guards
    registry.register(Arc::new(CheckInputGuard::new(0, true)));  // emergencyStopOk
    registry.register(Arc::new(CheckInputGuard::new(1, true)));  // doorClosed
    
    // Composite safety guard
    registry.register(Arc::new(SystemSafeGuard));
}
```

## Testing Without Hardware (Simulation)

### 1. Create State Machine in Editor

1. Open electron app: `cd electron && npm start`
2. Navigate to `/statechart`
3. Import `motor-control-hardware.json` from examples
4. Review states and transitions visually

### 2. Run Backend

```bash
cd server
cargo run
```

### 3. Execute State Machine

1. Click "Run in Backend"
2. Observe initial state: **Stopped** (red highlighted)
3. Check available events: `[START]`

### 4. Trigger Events

Click event buttons in sequence:

```
START → CheckingSafety
  ↓
SAFETY_OK → Starting
  ↓
MOTOR_READY → Running
  ↓
STOP → Stopping
  ↓
MOTOR_STOPPED → Stopped
```

### 5. Test Emergency Stop

From any state:
- Click `EMERGENCY_STOP` → observe immediate transition to Error state
- All outputs disabled
- Alarm triggered
- Click `RESET` (after clearing error) → back to Stopped

### 6. Simulated Input Updates

Use browser console to inject input values:

```javascript
// Simulate emergency stop pressed
socket.emit('updateContext', { 
  variable: 'input_0', 
  value: false 
});

// Simulate door opened
socket.emit('updateContext', { 
  variable: 'input_1', 
  value: false 
});
```

## Testing With Real Hardware

### 1. Configure EtherCAT Devices

In `server/config.toml`:

```toml
[ethercat]
interface = "eth0"  # Your EtherCAT interface

[[ethercat.devices]]
type = "EL2008"
position = 0
alias = "outputs"

[[ethercat.devices]]
type = "EL1008"
position = 1
alias = "inputs"
```

### 2. Modify Hardware Actions

Update `hardware_actions.rs` to use real EtherCAT:

```rust
impl Action for SetDigitalOutputAction {
    fn execute(&self, context: &mut MachineContext) -> Result<()> {
        // Get app_state with EtherCAT access
        let ethercat = context.get_ethercat()?;
        ethercat.outputs.set_output(self.output_id, self.value)?;
        
        tracing::info!("Hardware Output {} = {}", self.output_id, self.value);
        Ok(())
    }
}
```

### 3. Read Real Inputs

Create periodic task to update context with sensor values:

```rust
async fn update_inputs_task(state_machine: Arc<Mutex<StateMachine>>, ethercat: Arc<EthercatDevice>) {
    loop {
        let emergency_stop = ethercat.inputs.get_input(0)?;
        let door_closed = ethercat.inputs.get_input(1)?;
        
        let mut machine = state_machine.lock().await;
        machine.context_mut().set_bool("input_0", emergency_stop);
        machine.context_mut().set_bool("input_1", door_closed);
        
        // Check if safety violated during operation
        if machine.current_state() == "Running" {
            if !emergency_stop || !door_closed {
                machine.send("EMERGENCY_STOP")?;
            }
        }
        
        smol::Timer::after(Duration::from_millis(50)).await;
    }
}
```

### 4. Safety Verification

Before production use:

- [ ] Test emergency stop from all states
- [ ] Verify door interlock prevents start
- [ ] Confirm controlled ramp down on normal stop
- [ ] Validate alarm triggers on error
- [ ] Test reset procedure
- [ ] Verify all outputs return to safe state on disconnect

## Monitoring & Debugging

### Real-Time Visualization

The React Flow editor shows:
- ✅ **Green ring** around current state
- 📍 Previous state in execution panel
- 🎯 Available events as clickable buttons
- ⏱️ Timestamp of last transition

### Backend Logs

```bash
# Filter state machine logs
cargo run 2>&1 | grep StateMachine

# Expected output:
[INFO StateMachine] Transitioning from 'Stopped' to 'CheckingSafety'
[INFO StateMachine] Hardware Output 1 = false (clearRedLight)
[INFO StateMachine] Transitioning from 'CheckingSafety' to 'Starting'
[INFO StateMachine] Hardware Output 0 = true (enableMotor)
...
```

### Context Inspection

Add to `StateChartEditor.tsx`:

```tsx
<pre>
  {JSON.stringify(executionState, null, 2)}
</pre>
```

## Advanced Features

### 1. Speed Ramping

```rust
pub struct RampMotorAction {
    target_speed: f64,
    ramp_time_ms: u64,
}

impl Action for RampMotorAction {
    fn execute(&self, context: &mut MachineContext) -> Result<()> {
        let current_speed = context.get_float("motor_speed").unwrap_or(0.0);
        
        // Spawn async task to gradually increase speed
        smol::spawn(async move {
            let steps = 100;
            let step_time = self.ramp_time_ms / steps;
            let step_speed = (self.target_speed - current_speed) / steps as f64;
            
            for i in 0..steps {
                let speed = current_speed + (step_speed * i as f64);
                // ethercat.set_speed(speed)?;
                smol::Timer::after(Duration::from_millis(step_time)).await;
            }
        }).detach();
        
        Ok(())
    }
}
```

### 2. Timeout Transitions

Add timer in Starting state:

```rust
// In on_load_state_machine after creating machine
if machine.current_state() == "Starting" {
    let machine_clone = Arc::clone(&machine);
    smol::spawn(async move {
        smol::Timer::after(Duration::from_secs(5)).await;
        
        let mut m = machine_clone.lock().await;
        if m.current_state() == "Starting" {
            let _ = m.send("TIMEOUT");
        }
    }).detach();
}
```

### 3. Metrics Collection

Track state durations:

```rust
impl StateMachine {
    pub fn metrics(&self) -> StateMetrics {
        StateMetrics {
            transitions: self.transition_count,
            time_in_state: self.state_duration(),
            total_runtime: self.total_time(),
        }
    }
}
```

## Production Checklist

- [ ] All actions tested with real hardware
- [ ] Guards verify actual sensor states
- [ ] Emergency stop tested from every state
- [ ] Watchdog timer for state machine heartbeat
- [ ] Log all state transitions to file
- [ ] Persist state across restarts (for recovery)
- [ ] Add state machine validation before loading
- [ ] Implement user permissions for dangerous transitions
- [ ] Add rate limiting on event processing
- [ ] Create backup/restore for state machine configs

## Troubleshooting

### Issue: State machine not loading
- Check JSON syntax with online validator
- Verify all action/guard names are registered
- Check backend logs for parsing errors

### Issue: Transitions not triggering
- Verify guard conditions in context
- Check available_events in execution state
- Ensure event name matches exactly (case-sensitive)

### Issue: Hardware not responding
- Verify EtherCAT network is initialized
- Check device addresses in config
- Test outputs with simple test program first
- Ensure action names match registered actions

### Issue: Safety checks failing
- Check input wiring (NC vs NO)
- Verify input polling is running
- Test guards independently before loading machine
- Add debug logs in guard evaluation

## Example Session

```
1. User opens StateChart editor
2. Imports motor-control-hardware.json
3. Reviews states and transitions visually
4. Clicks "Run in Backend"
   → Backend creates StateMachine
   → Registers 20+ hardware actions
   → Enters "Stopped" state
   → Emits executionState
5. Frontend highlights "Stopped" node (green ring)
6. Shows available event: [START]
7. User clicks START button
   → Frontend emits sendEvent('START')
   → Backend processes event
   → Executes exit actions (clearRedLight)
   → Executes transition actions (logStartRequest)
   → Transitions to "CheckingSafety"
   → Executes entry actions (checkEmergencyStop, checkDoorClosed)
   → Emits new executionState
8. Frontend updates to highlight "CheckingSafety"
9. Shows available events: [SAFETY_OK, SAFETY_FAILED]
10. User clicks SAFETY_OK
    → systemSafe guard evaluated (checks input_0 && input_1)
    → If true: transition to "Starting"
    → enableMotor action sets Output 0 = HIGH
    → setYellowLight action sets Output 2 = HIGH
    → Motor begins spinning
11. After 2 seconds, motor reaches speed
    → Backend automatically triggers MOTOR_READY
    → motorSpeedOk guard passes
    → Transition to "Running"
    → setGreenLight, enableFullPower
12. System runs normally...
13. User presses physical emergency stop button
    → Input 0 goes LOW
    → Backend polling detects change
    → Injects EMERGENCY_STOP event
    → Immediate transition to "Error"
    → emergencyShutdown action disables motor
    → triggerAlarm action sets Output 4 = HIGH
14. User fixes issue, presses reset button
    → Frontend clicks RESET
    → errorCleared guard checks safety is restored
    → Transitions back to "Stopped"
    → Ready for next cycle
```

## Next Steps

1. Extend with more complex state machines (multi-motor coordination)
2. Add sub-states for detailed control sequences
3. Implement history states for pause/resume
4. Create library of reusable state patterns
5. Build template generator for common machine types
6. Add simulation mode with virtual hardware
