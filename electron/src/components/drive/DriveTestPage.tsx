import React from "react";
import { DriveControl } from "@/components/drive";
// import { useDrive } from "@/components/drive"; // Uncomment when backend is ready

/**
 * Test page for drive control component
 * Replicates TwinSharp .NET drive control dashboard
 * 
 * ARCHITECTURE: Socket.io (real-time events) - NOT polling
 * Follows the same pattern as winder2, extruder2, buffer1, laser1, mock1
 */
export function DriveTestPage() {
  // ========== Option 1: Mock mode (current - for testing without backend) ==========
  const handleCommand = (command: string, params?: any) => {
    console.log('[DriveTest]', command, params);
    // In production, this would be handled by Socket.io
  };

  // ========== Option 2: Socket.io with useDrive hook (uncomment when backend ready) ==========
  /*
  const machine_id = {
    machine_identification: { vendor: 0x766, machine: 0x402 }, // Lichuan LC10E
    serial: 1,
  };
  
  const {
    state,
    jogStart,
    jogStop,
    start,
    stop,
    reset,
    reference,
    setEnabling,
    downloadKvFactor,
    downloadRefVelocity,
    downloadTargetPosition,
    startMovement,
    stopMovement,
    startRawOutput,
    stopRawOutput,
    setActualPosition,
    setTargetPosition,
    isDisabled,
    isLoading,
  } = useDrive(machine_id);

  // Convert state to DriveState format for DriveControl
  const driveState = {
    name: state.name,
    position: state.position,
    setpointPosition: state.setpointPosition,
    velocity: state.velocity,
    errorCode: state.errorCode,
    lagError: state.lagError,
    actualVelocity: state.actualVelocity,
    setpointVelocity: state.setpointVelocity,
    override: state.override,
    outputPercent: state.outputPercent,
    controllerOutputPercent: state.controllerOutputPercent,
    ready: state.ready,
    calibrated: state.calibrated,
    hasJob: state.hasJob,
    notMoving: state.notMoving,
    movingForward: state.movingForward,
    movingBackward: state.movingBackward,
    coupledMode: state.coupledMode,
    inTargetPos: state.inTargetPos,
    inPosRange: state.inPosRange,
    controllerEnabled: state.controllerEnabled,
    feedFwEnabled: state.feedFwEnabled,
    feedBwEnabled: state.feedBwEnabled,
  };

  if (isLoading) {
    return (
      <div className="container mx-auto p-6">
        <p>Connecting to servo...</p>
      </div>
    );
  }

  if (!state.isConnected) {
    return (
      <div className="container mx-auto p-6">
        <p className="text-red-500">Not connected to backend</p>
      </div>
    );
  }
  */

  return (
    <div className="container mx-auto p-6">
      <div className="mb-4">
        <h1 className="text-2xl font-bold">Drive Control Test</h1>
        <p className="text-muted-foreground">
          CiA402 servo/drive control dashboard (TwinSharp .NET migration)
        </p>
        <p className="text-xs text-muted-foreground mt-2">
          💡 <strong>Tip:</strong> Use F1-F9 keys for shortcuts. Press F1-F4 and hold for JOG.
        </p>
        <div className="mt-2 p-3 bg-blue-50 dark:bg-blue-950 rounded-md border border-blue-200 dark:border-blue-800">
          <p className="text-sm font-semibold text-blue-900 dark:text-blue-100">
            🔌 Architecture: Socket.io (Real-time)
          </p>
          <p className="text-xs text-blue-700 dark:text-blue-300 mt-1">
            Backend sends <code>DriveStateEvent</code> every 100ms. Frontend uses <code>useDrive()</code> hook.
            ThrottledStoreUpdater limits re-renders to ~30 FPS.
          </p>
          <p className="text-xs text-blue-700 dark:text-blue-300 mt-1">
            Same pattern as: winder2, extruder2, buffer1, laser1, mock1
          </p>
        </div>
      </div>

      <DriveControl 
        axisName="Test Axis 1" 
        onCommand={handleCommand}
        // When using useDrive, commands are sent via Socket.io emit()
        // The onCommand prop becomes optional
      />
    </div>
  );
}
