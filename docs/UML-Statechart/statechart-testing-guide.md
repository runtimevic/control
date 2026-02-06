# StateChart Testing Guide

## Quick Start Test (No Hardware)

### 1. Start Backend
```bash
cd server
cargo run
```

Expected output:
```
[INFO] Socket.IO server listening on 0.0.0.0:3000
[INFO] StateChart namespace registered at /statechart
```

### 2. Start Frontend
```bash
cd electron
npm start
```

### 3. Navigate to StateChart Editor
- In the app, go to `/statechart`
- You should see an empty canvas with a single "Idle" node

### 4. Import Hardware Example
1. Click "Import JSON"
2. Select `electron/src/statechart/examples/motor-control-hardware.json`
3. Observe 6 states loaded: Stopped, CheckingSafety, Starting, Running, Stopping, Error

### 5. Run State Machine
1. Click "Run in Backend" button
2. Observe:
   - Connection status indicator: Green "Connected"
   - "Stopped" node highlighted with green ring
   - Execution panel appears showing:
     - Current: Stopped
     - Available Events: [START]

### 6. Execute State Transitions

#### Test Sequence 1: Normal Operation
```
1. Click [START] button
   → Node "CheckingSafety" highlighted
   → Available events: [SAFETY_OK, SAFETY_FAILED]
   → Backend logs: "Hardware Output 1 = false (clearRedLight)"

2. Click [SAFETY_OK] button
   → Node "Starting" highlighted
   → Available events: [MOTOR_READY, EMERGENCY_STOP, TIMEOUT]
   → Backend logs: "Hardware Output 0 = true (enableMotor)"
   → Backend logs: "Hardware Output 2 = true (setYellowLight)"

3. Click [MOTOR_READY] button
   → Node "Running" highlighted
   → Available events: [STOP, EMERGENCY_STOP, DOOR_OPENED]
   → Backend logs: "Hardware Output 3 = true (setGreenLight)"

4. Click [STOP] button
   → Node "Stopping" highlighted
   → Available events: [MOTOR_STOPPED, EMERGENCY_STOP]
   → Backend logs: "Hardware Output 2 = true (setYellowLight)"

5. Click [MOTOR_STOPPED] button
   → Node "Stopped" highlighted (back to initial)
   → Available events: [START]
   → Backend logs: "Hardware Output 1 = true (setRedLight)"
```

#### Test Sequence 2: Emergency Stop
```
1. Follow steps 1-3 from Test Sequence 1 to reach "Running"

2. Click [EMERGENCY_STOP] button
   → Node "Error" highlighted (red state)
   → Available events: [RESET]
   → Backend logs: "Hardware Output 0 = false (disableMotor)"
   → Backend logs: "Hardware Output 4 = true (triggerAlarm)"

3. Click [RESET] button
   → If guard passes: back to "Stopped"
   → If guard fails: stays in "Error" (safety not restored)
```

#### Test Sequence 3: Guard Blocking
```
1. From "CheckingSafety" state
2. Open browser console (F12)
3. Execute:
   ```javascript
   // Simulate emergency stop pressed (input goes LOW)
   // This will fail the systemSafe guard
   ```
4. Try to click [SAFETY_OK]
   → Transition blocked
   → Node stays in "CheckingSafety"
   → Backend logs: "Safety check FAILED: emergency_stop_ok=false"
```

## Expected Backend Logs

### Normal Startup
```
[INFO] Loading state machine for socket abc123
[INFO] State machine loaded successfully
[INFO] [StateMachine Hardware] logStop → Log
[INFO] [StateMachine Hardware] disableMotor → Output 0 = false
[INFO] [StateMachine Hardware] setRedLight → Output 1 = true
```

### State Transition
```
[INFO] Event 'START' triggered transition to 'CheckingSafety'
[INFO] [StateMachine Hardware] clearRedLight → Output 1 = false
[INFO] [StateMachine Hardware] logStartRequest → Log
[INFO] [StateMachine Hardware] checkEmergencyStop → Log
[INFO] [StateMachine Hardware] checkDoorClosed → Log
[INFO] [StateMachine Hardware] logSafetyCheck → Log
```

### Guard Evaluation
```
[INFO] [StateMachine Hardware] Safety check PASSED
[INFO] Event 'SAFETY_OK' triggered transition to 'Starting'
```

### Error Condition
```
[WARN] [StateMachine Hardware] Safety check FAILED: emergency_stop_ok=false, door_closed=true
[INFO] Event 'SAFETY_OK' did not trigger a transition (blocked by guard or no matching transition)
```

## Visual Verification Checklist

- [ ] Initial state (Stopped) is highlighted in green on load
- [ ] Clicking event button updates highlighting to new state
- [ ] Previous state is shown in execution panel
- [ ] Available events update after each transition
- [ ] Error state uses red border
- [ ] Connection indicator shows "Connected" (green badge)
- [ ] Disconnecting backend shows "Disconnected" (red badge)

## Performance Checks

### Latency
- Measure time from clicking event to visual update
- Expected: < 100ms on local network
- Test: Click event → observe timestamp in execution panel

### Concurrent Users
1. Open two browser windows
2. Both connect to `/statechart`
3. Load same state machine in both
4. Each operates independently (separate socket.id)

## Debugging Commands

### Check WebSocket Connection
```javascript
// In browser console
socket.connected // should be true
socket.id // shows your socket ID
```

### Manually Send Event
```javascript
socket.emit('sendEvent', { event: 'START' });
```

### Inspect Execution State
```javascript
socket.on('executionState', (state) => {
  console.table(state);
});
```

### Backend: Check Active Machines
Add endpoint to inspect active state machines:
```rust
// In rest API
pub async fn get_active_machines(state: Arc<SharedState>) -> Json<Vec<String>> {
    let machines = state.socketio_setup.namespaces
        .read().await
        .statechart_namespace
        .machines.lock().await;
    
    Json(machines.keys().cloned().collect())
}
```

## Common Issues & Solutions

### Issue: "Run in Backend" button disabled
- **Cause:** WebSocket not connected
- **Solution:** 
  1. Check backend is running (`cargo run`)
  2. Check port 3000 is not blocked
  3. Refresh frontend app

### Issue: No events available after transition
- **Cause:** Current state has no outgoing transitions
- **Solution:** Check state machine JSON for `on` field in current state

### Issue: Transition doesn't trigger
- **Cause:** Guard condition failed
- **Solution:** 
  1. Check backend logs for guard evaluation
  2. Verify context variables have expected values
  3. Add debug logs in guard implementation

### Issue: Multiple states highlighted
- **Cause:** Frontend not clearing previous highlighting
- **Solution:** Click "Stop" button and restart

### Issue: Backend crashes on event
- **Cause:** Action or guard panic
- **Solution:**
  1. Check backend logs for panic trace
  2. Verify all actions/guards are registered
  3. Ensure action names in JSON match registered names exactly

## Advanced Testing

### Load Testing
```bash
# Install artillery
npm install -g artillery

# Create test script
cat > load-test.yml <<EOF
config:
  target: "http://localhost:3000"
  phases:
    - duration: 60
      arrivalRate: 10
  socketio:
    transports: ['websocket']

scenarios:
  - engine: socketio
    flow:
      - emit:
          channel: "loadStateMachine"
          data:
            config: '{"id":"test",...}'
      - think: 1
      - emit:
          channel: "sendEvent"
          data:
            event: "START"
EOF

# Run test
artillery run load-test.yml
```

### Memory Leak Testing
```bash
# Monitor memory over time
watch -n 5 'ps aux | grep cargo'

# Expected: Memory stable after initial allocations
```

### Guard Stress Test
```javascript
// Rapidly toggle guard condition
setInterval(() => {
  const value = Math.random() > 0.5;
  // Update context variable
  console.log(`Setting guard condition to ${value}`);
}, 100);
```

## Test Checklist

### Functional Tests
- [ ] Load state machine from JSON
- [ ] Initial state is entered correctly
- [ ] Events trigger transitions
- [ ] Guards block transitions when false
- [ ] Guards allow transitions when true
- [ ] Actions execute in correct order (exit → transition → entry)
- [ ] Final states terminate correctly
- [ ] Error states can be reset
- [ ] Multiple state machines can run concurrently

### Integration Tests
- [ ] Frontend highlights correct node
- [ ] Available events update correctly
- [ ] WebSocket reconnects after disconnect
- [ ] State persists during reconnection
- [ ] Multiple clients don't interfere

### Edge Cases
- [ ] Empty state machine
- [ ] State with no transitions
- [ ] Self-loops (state to same state)
- [ ] Multiple transitions from same event
- [ ] Rapid event firing
- [ ] Malformed JSON rejection
- [ ] Unknown action/guard names

### Performance Tests
- [ ] 1000 transitions/second
- [ ] 100 concurrent state machines
- [ ] Complex state machine (50+ states)
- [ ] Large context (1000+ variables)

## Next Steps

After successful testing:
1. Add hardware-specific actions in `hardware_actions.rs`
2. Integrate with actual EtherCAT devices
3. Add input polling for sensor data
4. Implement timeout transitions
5. Add state machine persistence
6. Create production monitoring dashboard
