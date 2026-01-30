import { useState, useEffect, useCallback } from 'react';
import { DriveState } from './types';

interface UseDriveConnectionOptions {
  axisName: string;
  updateInterval?: number; // milliseconds, default 100
  autoConnect?: boolean;
}

interface DriveConnection {
  state: DriveState | null;
  isConnected: boolean;
  isLoading: boolean;
  error: Error | null;
  sendCommand: (command: string, params?: any) => Promise<void>;
  connect: () => void;
  disconnect: () => void;
}

/**
 * Custom hook for managing drive/servo connection and state updates
 * Handles both REST API polling and WebSocket connections
 */
export function useDriveConnection({
  axisName,
  updateInterval = 100,
  autoConnect = true,
}: UseDriveConnectionOptions): DriveConnection {
  const [state, setState] = useState<DriveState | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  // Fetch current state from backend
  const fetchState = useCallback(async () => {
    try {
      // TODO: Replace with actual API endpoint
      // const response = await fetch(`/api/servo/${axisName}/state`);
      // if (!response.ok) throw new Error(`HTTP ${response.status}`);
      // const data = await response.json();
      // setState(data);
      // setIsConnected(true);
      // setError(null);
      
      // Mock data for now
      if (!state) {
        setState({
          name: axisName,
          position: 0,
          setpointPosition: 0,
          velocity: 0,
          errorCode: 0,
          lagError: { current: 0, min: 0, max: 0 },
          actualVelocity: 0,
          setpointVelocity: 0,
          override: 10000,
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
      }
      setIsConnected(true);
    } catch (err) {
      setError(err as Error);
      setIsConnected(false);
    }
  }, [axisName, state]);

  // Send command to backend
  const sendCommand = useCallback(async (command: string, params?: any) => {
    try {
      // TODO: Replace with actual API endpoint
      // const response = await fetch(`/api/servo/${axisName}/command`, {
      //   method: 'POST',
      //   headers: { 'Content-Type': 'application/json' },
      //   body: JSON.stringify({ command, params }),
      // });
      // if (!response.ok) throw new Error(`HTTP ${response.status}`);
      
      console.log(`[useDriveConnection] Command: ${command}`, params);
      
      // Update local state for immediate feedback (optimistic update)
      if (command === 'jog_start') {
        setState((prev) => prev ? {
          ...prev,
          movingForward: params.direction.includes('+'),
          movingBackward: params.direction.includes('-'),
          notMoving: false,
        } : null);
      } else if (command === 'jog_stop') {
        setState((prev) => prev ? {
          ...prev,
          movingForward: false,
          movingBackward: false,
          notMoving: true,
        } : null);
      } else if (command === 'set_enabling') {
        setState((prev) => prev ? {
          ...prev,
          controllerEnabled: params.controller,
          feedFwEnabled: params.feedFw,
          feedBwEnabled: params.feedBw,
          override: params.override,
        } : null);
      }
    } catch (err) {
      console.error('Failed to send command:', err);
      throw err;
    }
  }, [axisName]);

  // Connect to backend
  const connect = useCallback(() => {
    setIsLoading(true);
    fetchState().finally(() => setIsLoading(false));
  }, [fetchState]);

  // Disconnect from backend
  const disconnect = useCallback(() => {
    setIsConnected(false);
    setState(null);
  }, []);

  // Auto-connect and setup update timer
  useEffect(() => {
    if (!autoConnect) return;

    connect();

    const timer = setInterval(fetchState, updateInterval);

    return () => {
      clearInterval(timer);
    };
  }, [autoConnect, connect, fetchState, updateInterval]);

  return {
    state,
    isConnected,
    isLoading,
    error,
    sendCommand,
    connect,
    disconnect,
  };
}
