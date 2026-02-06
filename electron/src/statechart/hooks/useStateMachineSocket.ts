import { useCallback, useRef, useState, useEffect } from "react";
import { 
  createNamespaceHookImplementation, 
  EventHandler,
  NamespaceId,
  useSocketioStore
} from "@/client/socketioStore";
import { create, StoreApi } from "zustand";
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

interface StateMachineStore {
  executionState: ExecutionState | null;
  loadError: string | null;
}

function createStateMachineStore(): StoreApi<StateMachineStore> {
  return create<StateMachineStore>(() => ({
    executionState: null,
    loadError: null,
  }));
}

function stateMachineMessageHandler(store: StoreApi<StateMachineStore>): EventHandler {
  return (event: any) => {
    const eventName = event.name;
    
    try {
      if (eventName === "executionState") {
        console.log("[StateChart] Received executionState:", event.data);
        store.setState({ executionState: event.data });
      } else if (eventName === "loadStateMachineResponse") {
        const response = event.data as LoadStateMachineResponse;
        console.log("[StateChart] Load response:", response);
        if (!response.success) {
          store.setState({ loadError: response.message });
        } else {
          store.setState({ 
            loadError: null,
            executionState: response.execution_state || null 
          });
        }
      }
    } catch (error) {
      console.error(`Error processing ${eventName} event:`, error);
    }
  };
}

const stateMachineStore = createStateMachineStore();
const stateMachineImplementation = createNamespaceHookImplementation({
  createStore: () => stateMachineStore,
  createEventHandler: stateMachineMessageHandler,
});

function useStateMachineNamespace(): StateMachineStore {
  const namespaceId = useRef({ type: "statechart" } satisfies NamespaceId);
  return stateMachineImplementation(namespaceId.current);
}

/**
 * Hook for connecting to the /statechart Socket.IO namespace
 * Provides real-time state machine execution and visualization
 */
export const useStateMachineSocket = () => {
  const store = useStateMachineNamespace();
  const namespaceId: NamespaceId = { type: "statechart" };
  
  // Track connection status with local state
  const [isConnected, setIsConnected] = useState(false);
  const socketStore = useSocketioStore();
  
  // Monitor socket connection status
  useEffect(() => {
    const checkConnection = () => {
      const namespace = socketStore.getNamespace(namespaceId);
      const connected = namespace?.socket?.connected ?? false;
      setIsConnected(connected);
    };
    
    // Check immediately and then poll for changes
    checkConnection();
    const intervalId = setInterval(checkConnection, 100);
    
    return () => clearInterval(intervalId);
  }, [socketStore, namespaceId]);

  /**
   * Load a state machine configuration into the backend
   */
  const loadMachine = useCallback(
    (config: XStateConfig) => {
      const json = JSON.stringify(config);
      console.log("[StateChart] Loading state machine:", config.id);
      
      const socketStore = useSocketioStore.getState();
      const namespace = socketStore.getNamespace(namespaceId);
      if (namespace?.socket) {
        namespace.socket.emit("loadStateMachine", { config: json } as LoadStateMachineMessage);
      } else {
        console.error("[StateChart] Socket not connected");
      }
    },
    [namespaceId]
  );

  /**
   * Send an event to the running state machine
   */
  const sendEvent = useCallback(
    (event: string) => {
      console.log("[StateChart] Sending event:", event);
      
      const socketStore = useSocketioStore.getState();
      const namespace = socketStore.getNamespace(namespaceId);
      if (namespace?.socket) {
        namespace.socket.emit("sendEvent", { event } as SendEventMessage);
      } else {
        console.error("[StateChart] Socket not connected");
      }
    },
    [namespaceId]
  );

  /**
   * Get available events from current state
   */
  const availableEvents = store.executionState?.available_events || [];

  /**
   * Get current state name
   */
  const currentState = store.executionState?.current_state;

  /**
   * Get previous state name
   */
  const previousState = store.executionState?.previous_state;

  return {
    // Connection status (from real-time socket state)
    isConnected,
    
    // State machine control
    loadMachine,
    sendEvent,
    
    // Execution state
    executionState: store.executionState,
    currentState,
    previousState,
    availableEvents,
    
    // Error handling
    loadError: store.loadError,
  };
};
