import { useCallback, useState, useEffect } from "react";
import { 
  NamespaceId,
  useSocketioStore
} from "@/client/socketioStore";
import { create } from "zustand";
import type { ExecutionState, XStateConfig } from "../types";
import type { MachineIdentificationUnique } from "@/machines/types";

interface LoadStateMachineMessage {
  config: string;
  machine_id?: MachineIdentificationUnique;
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
 * 
 * @param machineId - Optional machine identification. If provided, connects to machine-specific namespace
 */
export const useStateMachineSocket = (machineId?: MachineIdentificationUnique | null) => {
  const namespaceId: NamespaceId = machineId 
    ? { type: "machine-statechart", machine_identification_unique: machineId }
    : { type: "statechart" };
  
  console.log("[useStateMachineSocket] Creating hook with machineId:", machineId);
  console.log("[useStateMachineSocket] NamespaceId:", namespaceId);
  
  const [executionState, setExecutionState] = useState<ExecutionState | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const { 
    getNamespace,
    hasNamespace,
    initNamespace,
    incrementNamespace,
    decrementNamespace,
  } = useSocketioStore();
  
  // Initialize namespace if needed
  useEffect(() => {
    console.log("[useStateMachineSocket] Initializing namespace:", namespaceId);
    if (!hasNamespace(namespaceId)) {
      console.log("[useStateMachineSocket] Namespace doesn't exist, creating it");
      initNamespace(
        namespaceId,
        () => create(() => ({ executionState: null, loadError: null })),
        () => () => {} // Empty event handler, we handle events manually below
      );
    } else {
      incrementNamespace(namespaceId);
    }
    
    return () => {
      decrementNamespace(namespaceId);
    };
  }, [namespaceId, hasNamespace, initNamespace, incrementNamespace, decrementNamespace]);
  
  // Setup event listener for this namespace
  useEffect(() => {
    const namespace = getNamespace(namespaceId);
    
    if (!namespace?.socket) {
      console.log("[StateChart] No socket available yet");
      return;
    }
    
    const socket = namespace.socket;
    console.log("[StateChart] Setting up event listener, socket connected:", socket.connected);
    
    const handleEvent = (event: any) => {
      try {
        console.log("[StateChart] Raw event received:", event);
        const eventName = event.name;
        
        if (eventName === "executionState") {
          console.log("[StateChart] Received executionState:", event.data);
          setExecutionState(event.data);
        } else if (eventName === "loadStateMachineResponse") {
          const response = event.data as LoadStateMachineResponse;
          console.log("[StateChart] Load response:", response);
          if (!response.success) {
            setLoadError(response.message);
          } else {
            setLoadError(null);
            if (response.execution_state) {
              console.log("[StateChart] Setting execution state from response:", response.execution_state);
              setExecutionState(response.execution_state);
            }
          }
        } else {
          console.log("[StateChart] Unknown event:", eventName);
        }
      } catch (error) {
        console.error("Error processing event:", error);
      }
    };
    
    socket.on("event", handleEvent);
    
    return () => {
      socket.off("event", handleEvent);
    };
  }, [namespaceId, getNamespace]);
  
  // Monitor socket connection status
  useEffect(() => {
    const namespace = getNamespace(namespaceId);
    
    if (!namespace?.socket) {
      setIsConnected(false);
      return;
    }
    
    const socket = namespace.socket;
    
    const updateConnectionStatus = () => {
      setIsConnected(socket.connected);
      console.log(`[StateChart] Connection status: ${socket.connected}`);
    };
    
    // Set initial status
    updateConnectionStatus();
    
    // Listen to connection events
    socket.on("connect", updateConnectionStatus);
    socket.on("disconnect", updateConnectionStatus);
    
    // Poll as backup in case events are missed
    const intervalId = setInterval(updateConnectionStatus, 1000);
    
    return () => {
      socket.off("connect", updateConnectionStatus);
      socket.off("disconnect", updateConnectionStatus);
      clearInterval(intervalId);
    };
  }, [namespaceId, getNamespace]);

  /**
   * Load a state machine configuration into the backend
   */
  const loadMachine = useCallback(
    (config: XStateConfig) => {
      const json = JSON.stringify(config);
      console.log("[StateChart] Loading state machine:", config.id, "for machine:", machineId);
      console.log("[StateChart] Config JSON length:", json.length);
      
      const namespace = getNamespace(namespaceId);
      if (namespace?.socket) {
        console.log("[StateChart] Socket status:", {
          connected: namespace.socket.connected,
          disconnected: namespace.socket.disconnected,
        });
        const message: LoadStateMachineMessage = {
          config: json,
          ...(machineId ? { machine_id: machineId } : {}),
        };
        
        console.log("[StateChart] Emitting loadStateMachine message:", message);
        namespace.socket.emit("loadStateMachine", message);
        console.log("[StateChart] Message emitted, waiting for response...");
      } else {
        console.error("[StateChart] Socket not connected - namespace:", namespaceId);
      }
    },
    [namespaceId, machineId, getNamespace]
  );

  /**
   * Send an event to the running state machine
   */
  const sendEvent = useCallback(
    (event: string) => {
      console.log("[StateChart] Sending event:", event);
      
      const namespace = getNamespace(namespaceId);
      if (namespace?.socket) {
        namespace.socket.emit("sendEvent", { event } as SendEventMessage);
      } else {
        console.error("[StateChart] Socket not connected");
      }
    },
    [namespaceId, getNamespace]
  );

  /**
   * Get available events from current state
   */
  const availableEvents = executionState?.availableEvents || [];

  /**
   * Get current state name
   */
  const currentState = executionState?.currentState;

  /**
   * Get previous state name
   */
  const previousState = executionState?.previousState;

  return {
    // Connection status (from real-time socket state)
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
