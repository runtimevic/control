import { useEffect, useState, useCallback } from "react";
import { useSocketioStore } from "@/client/socketioStore";
import type { ExecutionState, XStateConfig } from "../types";

interface LoadStateMachineMessage {
  config: string;
}

interface SendEventMessage {
  event: string;
}

interface LoadStateMachineResponse {
  success: boolean;
  message: string;
  execution_state?: ExecutionState;
}

/**
 * Hook for connecting to the /statechart Socket.IO namespace
 * Provides real-time state machine execution and visualization
 */
export const useStateMachineSocket = () => {
  const [executionState, setExecutionState] = useState<ExecutionState | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);

  // Connect to /statechart namespace
  const { socket } = useSocketioStore("/statechart");

  useEffect(() => {
    if (!socket) return;

    // Connection status
    setIsConnected(socket.connected);
    socket.on("connect", () => setIsConnected(true));
    socket.on("disconnect", () => setIsConnected(false));

    // Listen for execution state updates
    socket.on("executionState", (state: ExecutionState) => {
      console.log("[StateChart] Received executionState:", state);
      setExecutionState(state);
    });

    // Listen for load response
    socket.on("loadStateMachineResponse", (response: LoadStateMachineResponse) => {
      console.log("[StateChart] Load response:", response);
      if (!response.success) {
        setLoadError(response.message);
      } else {
        setLoadError(null);
        if (response.execution_state) {
          setExecutionState(response.execution_state);
        }
      }
    });

    return () => {
      socket.off("connect");
      socket.off("disconnect");
      socket.off("executionState");
      socket.off("loadStateMachineResponse");
    };
  }, [socket]);

  /**
   * Load a state machine configuration into the backend
   */
  const loadMachine = useCallback(
    (config: XStateConfig) => {
      if (!socket) {
        console.error("[StateChart] Socket not connected");
        return;
      }

      const json = JSON.stringify(config);
      console.log("[StateChart] Loading state machine:", config.id);
      
      socket.emit("loadStateMachine", { config: json } as LoadStateMachineMessage);
    },
    [socket]
  );

  /**
   * Send an event to the running state machine
   */
  const sendEvent = useCallback(
    (event: string) => {
      if (!socket) {
        console.error("[StateChart] Socket not connected");
        return;
      }

      if (!executionState) {
        console.warn("[StateChart] No state machine loaded");
        return;
      }

      console.log("[StateChart] Sending event:", event);
      socket.emit("sendEvent", { event } as SendEventMessage);
    },
    [socket, executionState]
  );

  /**
   * Get available events from current state
   */
  const availableEvents = executionState?.available_events || [];

  /**
   * Get current state name
   */
  const currentState = executionState?.current_state;

  /**
   * Get previous state name
   */
  const previousState = executionState?.previous_state;

  return {
    // Connection status
    isConnected,
    
    // State machine control
    loadMachine,
    sendEvent,
    
    // Execution state
    executionState,
    currentState,
    previousState,
    availableEvents,
    
    // Error handling
    loadError,
  };
};
