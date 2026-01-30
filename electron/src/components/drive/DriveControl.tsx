import React, { useState, useEffect, useRef } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { DriveAxisHeader } from "./DriveAxisHeader";
import { DriveStatusPanel } from "./DriveStatusPanel";
import { JogControls } from "./JogControls";
import { DriveFunctionsPanel } from "./DriveFunctionsPanel";
import { SetEnablingDialog } from "./SetEnablingDialog";
import { DriveState, DriveControlParams, RawDriveOutputMode, SetPositionMode, SetTargetMode } from "./types";

interface DriveControlProps {
  axisName?: string;
  // Optional: API callbacks for real backend integration
  onCommand?: (command: string, params?: any) => void;
}

/**
 * DriveControl component - Main axis control interface
 * Matches NcAxis from TwinSharp .NET with TabControl (Online/Functions)
 * Size: 516x539px in .NET, responsive in React
 */
export function DriveControl({ axisName = "Axis 1", onCommand }: DriveControlProps) {
  // Mock state - replace with real API data
  const [driveState, setDriveState] = useState<DriveState>({
    name: axisName,
    position: 123.4567,
    setpointPosition: 123.4000,
    velocity: 0,
    errorCode: 0,
    lagError: { current: 0.0123, min: -0.005, max: 0.015 },
    actualVelocity: 0,
    setpointVelocity: 0,
    override: 10000, // 100.00%
    outputPercent: 0,
    controllerOutputPercent: 0,
    ready: true,
    calibrated: false,
    hasJob: false,
    notMoving: true,
    movingForward: false,
    movingBackward: false,
    coupledMode: false,
    inTargetPos: true,
    inPosRange: true,
    controllerEnabled: true,
    feedFwEnabled: true,
    feedBwEnabled: true,
  });

  const [params, setParams] = useState<DriveControlParams>({
    targetPosition: 0,
    targetVelocity: 10,
    acceleration: 100,
    deceleration: 100,
    jerk: 1000,
    enableAcceleration: true,
    enableDeceleration: true,
    enableJerk: false,
  });

  const [kvFactor, setKvFactor] = useState(1.0);
  const [refVelocity, setRefVelocity] = useState(10.0);
  const [targetPosition, setTargetPosition] = useState(0);
  const [lastTime, setLastTime] = useState(0);
  const [enablingDialogOpen, setEnablingDialogOpen] = useState(false);
  const [isConnected, setIsConnected] = useState(false);
  const jogActiveRef = useRef<string | null>(null);
  const updateTimerRef = useRef<NodeJS.Timeout | null>(null);

  // Command handlers
  const handleJogStart = (direction: 'fast-' | 'slow-' | 'slow+' | 'fast+') => {
    console.log('Jog start:', direction);
    onCommand?.('jog_start', { direction });
  };

  const handleJogStop = () => {
    console.log('Jog stop');
    onCommand?.('jog_stop');
  };

  const handleStart = () => {
    console.log('Start axis');
    onCommand?.('start');
  };

  const handleStop = () => {
    console.log('Stop axis');
    onCommand?.('stop');
  };

  const handleReset = () => {
    console.log('Reset axis');
    onCommand?.('reset');
  };

  const handleReference = () => {
    console.log('Reference axis');
    onCommand?.('reference');
  };

  const handleSetEnabling = () => {
    setEnablingDialogOpen(true);
  };

  const handleEnablingConfirm = (enabling: {
    controller: boolean;
    feedFw: boolean;
    feedBw: boolean;
    override: number;
  }) => {
    console.log('Set enabling confirmed:', enabling);
    setDriveState((prev) => ({
      ...prev,
      controllerEnabled: enabling.controller,
      feedFwEnabled: enabling.feedFw,
      feedBwEnabled: enabling.feedBw,
      override: enabling.override,
    }));
    onCommand?.('set_enabling', enabling);
  };

  const handleMovementStart = () => {
    console.log('Movement start with params:', params);
    onCommand?.('movement_start', params);
  };

  const handleMovementStop = () => {
    console.log('Movement stop');
    onCommand?.('movement_stop');
  };

  const handleStartRawOutput = (mode: RawDriveOutputMode, value: number) => {
    console.log('Start raw output:', mode, value);
    onCommand?.('raw_output_start', { mode, value });
  };

  const handleStopRawOutput = () => {
    console.log('Stop raw output');
    onCommand?.('raw_output_stop');
  };

  const handleSetActualPosition = (mode: SetPositionMode, value: number) => {
    console.log('Set actual position:', mode, value);
    onCommand?.('set_actual_position', { mode, value });
  };

  const handleSetTargetPosition = (mode: SetTargetMode, value: number) => {
    console.log('Set target position:', mode, value);
    onCommand?.('set_target_position', { mode, value });
  };

  const handleDownloadKvFactor = () => {
    console.log('Download Kv factor:', kvFactor);
    onCommand?.('download_kv_factor', { value: kvFactor });
  };

  const handleDownloadRefVelocity = () => {
    console.log('Download ref velocity:', refVelocity);
    onCommand?.('download_ref_velocity', { value: refVelocity });
  };

  const handleDownloadTargetPosition = () => {
    console.log('Download target position:', targetPosition);
    onCommand?.('download_target_position', { value: targetPosition });
  };

  // Keyboard shortcuts (F1-F9)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Only handle function keys
      if (!e.key.startsWith('F')) return;
      
      // Prevent default browser behavior for F keys
      e.preventDefault();

      switch (e.key) {
        case 'F1':
          jogActiveRef.current = 'fast-';
          handleJogStart('fast-');
          break;
        case 'F2':
          jogActiveRef.current = 'slow-';
          handleJogStart('slow-');
          break;
        case 'F3':
          jogActiveRef.current = 'slow+';
          handleJogStart('slow+');
          break;
        case 'F4':
          jogActiveRef.current = 'fast+';
          handleJogStart('fast+');
          break;
        case 'F5':
          handleStart();
          break;
        case 'F6':
          handleStop();
          break;
        case 'F8':
          handleReset();
          break;
        case 'F9':
          handleReference();
          break;
      }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      // Stop JOG on key release (F1-F4)
      if (e.key === 'F1' || e.key === 'F2' || e.key === 'F3' || e.key === 'F4') {
        e.preventDefault();
        if (jogActiveRef.current) {
          handleJogStop();
          jogActiveRef.current = null;
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
    };
  }, [driveState.ready]); // Re-bind when ready state changes

  // Update timer (100ms) - matches .NET tmrUpdate_Tick
  useEffect(() => {
    const fetchDriveState = async () => {
      try {
        // TODO: Replace with actual API call
        // const response = await fetch(`/api/servo/${axisName}/state`);
        // const data = await response.json();
        // setDriveState(data);
        // setIsConnected(true);
        
        // For now, simulate data updates (mock)
        if (Math.random() < 0.1) { // 10% chance to update position
          setDriveState((prev) => ({
            ...prev,
            position: prev.position + (Math.random() - 0.5) * 0.001,
            actualVelocity: prev.movingForward ? 10 : prev.movingBackward ? -10 : 0,
          }));
        }
      } catch (error) {
        console.error('Failed to fetch drive state:', error);
        setIsConnected(false);
      }
    };

    // Start update timer
    updateTimerRef.current = setInterval(fetchDriveState, 100);

    return () => {
      if (updateTimerRef.current) {
        clearInterval(updateTimerRef.current);
        updateTimerRef.current = null;
      }
    };
  }, [axisName]); // Re-create timer if axis name changes

  return (
    <div className="space-y-4 max-w-2xl">
      {/* Axis Header - Matching NcAxisHeader (464x48px) */}
      <DriveAxisHeader
        name={driveState.name}
        actualPosition={driveState.position}
        setpointPosition={driveState.setpointPosition}
        isMoving={driveState.movingForward || driveState.movingBackward}
      />

      {/* Tab Control - Matching NcAxis TabControl (Online/Functions) */}
      <Tabs defaultValue="online" className="w-full">
        <TabsList className="grid w-full grid-cols-2">
          <TabsTrigger value="online">Online</TabsTrigger>
          <TabsTrigger value="functions">Functions</TabsTrigger>
        </TabsList>

        {/* Online Tab - NcOnline (498x427px in .NET) */}
        <TabsContent value="online" className="space-y-4 mt-4">
          <DriveStatusPanel
            state={driveState}
            kvFactor={kvFactor}
            refVelocity={refVelocity}
            targetPosition={targetPosition}
            onSetEnabling={handleSetEnabling}
            onDownloadKvFactor={handleDownloadKvFactor}
            onDownloadRefVelocity={handleDownloadRefVelocity}
            onDownloadTargetPosition={handleDownloadTargetPosition}
            onKvFactorChange={setKvFactor}
            onRefVelocityChange={setRefVelocity}
            onTargetPositionChange={setTargetPosition}
          />

          <JogControls
            onJogStart={handleJogStart}
            onJogStop={handleJogStop}
            onStart={handleStart}
            onStop={handleStop}
            onReset={handleReset}
            onReference={handleReference}
            disabled={!driveState.ready}
          />
        </TabsContent>

        {/* Functions Tab - NcFunctions (498x513px in .NET) */}
        <TabsContent value="functions" className="mt-4">
          <DriveFunctionsPanel
            params={params}
            lastTime={lastTime}
            onParamsChange={(newParams) => setParams({ ...params, ...newParams })}
            onStart={handleMovementStart}
            onStop={handleMovementStop}
            onStartRawOutput={handleStartRawOutput}
            onStopRawOutput={handleStopRawOutput}
            onSetActualPosition={handleSetActualPosition}
            onSetTargetPosition={handleSetTargetPosition}
          />
        </TabsContent>
      </Tabs>

      {/* Set Enabling Dialog */}
      <SetEnablingDialog
        open={enablingDialogOpen}
        onOpenChange={setEnablingDialogOpen}
        currentEnabling={{
          controller: driveState.controllerEnabled,
          feedFw: driveState.feedFwEnabled,
          feedBw: driveState.feedBwEnabled,
        }}
        currentOverride={driveState.override}
        onConfirm={handleEnablingConfirm}
      />
    </div>
  );
}
