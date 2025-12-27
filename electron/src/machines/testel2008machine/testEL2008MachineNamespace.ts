import { StoreApi } from "zustand";
import { create } from "zustand";
import { z } from "zod";
import {
  EventHandler,
  eventSchema,
  Event,
  handleUnhandledEventError,
  NamespaceId,
  createNamespaceHookImplementation,
  ThrottledStoreUpdater,
} from "@/client/socketioStore";
import { MachineIdentificationUnique } from "@/machines/types";

// ========== Event Schema ==========

export const stateEventDataSchema = z.object({
  led_on: z.array(z.boolean()).length(8),
  mode: z.enum(["Manual", "Home", "Automatic"]),
  machine_state: z.enum(["Stopped", "Running"]),
  automatic_delay_ms: z.number(),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);

export type StateEvent = z.infer<typeof stateEventDataSchema>;

// ========== Store ==========
export type TestEL2008MachineNamespaceStore = {
  state: StateEvent | null;
};

export const createTestEL2008MachineNamespaceStore =
  (): StoreApi<TestEL2008MachineNamespaceStore> =>
    create<TestEL2008MachineNamespaceStore>(() => ({
      state: null,
    }));

// ========== Message Handler ==========
export function testEL2008MachineMessageHandler(
  store: StoreApi<TestEL2008MachineNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<TestEL2008MachineNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (state: TestEL2008MachineNamespaceStore) => TestEL2008MachineNamespaceStore,
    ) => throttledUpdater.updateWith(updater);

    try {
      if (event.name === "StateEvent") {
        const parsed = stateEventSchema.parse(event);
        updateStore(() => ({ state: parsed.data }));
      } else {
        handleUnhandledEventError(event.name);
      }
    } catch (error) {
      console.error(`Error processing ${event.name}:`, error);
      throw error;
    }
  };
}

// ========== Namespace Hook ==========
const useTestEL2008MachineNamespaceImplementation =
  createNamespaceHookImplementation<TestEL2008MachineNamespaceStore>({
    createStore: createTestEL2008MachineNamespaceStore,
    createEventHandler: testEL2008MachineMessageHandler,
  });

export function useTestEL2008MachineNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): TestEL2008MachineNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useTestEL2008MachineNamespaceImplementation(namespaceId);
}