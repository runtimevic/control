import { createNamespaceHookImplementation } from "./socketioStore";
import { create } from "zustand";
import { useMemo, useRef } from "react";
import { MachineIdentificationUnique } from "@/machines/types";
import { useMachineMutate as useMachineMutationInternal } from "@/client/useClient";
import { ZodSchema } from "zod";

// Minimal generic machine state hook: stores latest events keyed by name
type GenericMachineState = Record<string, any>;

const createGenericMachineStore = () => create<GenericMachineState>(() => ({}));

function genericMachineMessageHandler(
  store: ReturnType<typeof createGenericMachineStore>,
) {
  return (event: any) => {
    const old = store.getState();
    const next = { ...old, [event.name]: event };
    store.setState(next);
  };
}

export function useMachineState(
  machineIdentification: MachineIdentificationUnique,
  _eventsSchema?: ZodSchema<any>,
) {
  // Stabilize the namespace ID to prevent infinite loops
  const namespaceId = useMemo(
    () => ({ 
      type: "machine" as const, 
      machine_identification_unique: machineIdentification 
    }),
    [
      machineIdentification.machine_identification.vendor,
      machineIdentification.machine_identification.machine,
      machineIdentification.serial,
    ]
  );

  // create a namespace hook implementation for this generic handler
  const hook = useMemo(() =>
    createNamespaceHookImplementation({
      createStore: () => createGenericMachineStore(),
      createEventHandler: (store) => genericMachineMessageHandler(store),
    }),
    [],
  );

  const state = hook(namespaceId);

  return {
    value: state,
    isOptimistic: false,
    isInitialized: Object.keys(state || {}).length > 0,
  } as const;
}

// Re-export mutation hook under expected name
export const useMachineMutation = useMachineMutationInternal;
