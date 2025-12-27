import { toastError } from "@/components/Toast";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { testEL2008MachineSerialRoute } from "@/routes/routes";
import { MachineIdentificationUnique } from "@/machines/types";
import { useTestEL2008MachineNamespace, StateEvent } from "./testEL2008MachineNamespace";
import { useMachineMutate } from "@/client/useClient";
import { produce } from "immer";
import { useEffect, useMemo } from "react";
import { testEL2008Machine } from "@/machines/properties";
import { z } from "zod";
export function useTestEL2008Machine() {
  const { serial: serialString } = testEL2008MachineSerialRoute.useParams();

  // Memoize machine identification
  const machineIdentification: MachineIdentificationUnique = useMemo(() => {
    const serial = parseInt(serialString);

    if (isNaN(serial)) {
      toastError(
        "Invalid Serial Number",
        `"${serialString}" is not a valid serial number.`,
      );

      return {
        machine_identification: { vendor: 0, machine: 0 },
        serial: 0,
      };
    }

    return {
      machine_identification: testEL2008Machine.machine_identification,
      serial,
    };
  }, [serialString]);

  // Namespace state from backend
  const { state } = useTestEL2008MachineNamespace(machineIdentification);

  // Optimistic state
  const stateOptimistic = useStateOptimistic<StateEvent>();

  useEffect(() => {
    if (state) stateOptimistic.setReal(state);
  }, [state, stateOptimistic]);

  // Generic mutation sender
  const { request: sendMutation } = useMachineMutate(
    z.object({
      action: z.string(),
      value: z.any(),
    }),
  );

  const updateStateOptimistically = (
    producer: (current: StateEvent) => void,
    serverRequest?: () => void,
  ) => {
    const currentState = stateOptimistic.value;
    if (currentState)
      stateOptimistic.setOptimistic(produce(currentState, producer));
    serverRequest?.();
  };

  const setLed = (index: number, on: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.led_on[index] = on;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetLed", value: { index, on } },
        }),
    );
  };

  const setAllLeds = (on: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.led_on = [on, on, on, on, on, on, on, on];
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetAllLeds", value: { on } },
        }),
    );
  };

  const setMode = (mode: "Manual" | "Home" | "Automatic") => {
    updateStateOptimistically(
      (current) => {
        current.mode = mode;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetMode", value: { mode } },
        }),
    );
  };

  const start = () => {
    updateStateOptimistically(
      (current) => {
        current.machine_state = "Running";
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "Start", value: null },
        }),
    );
  };

  const stop = () => {
    updateStateOptimistically(
      (current) => {
        current.machine_state = "Stopped";
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "Stop", value: null },
        }),
    );
  };

  const reset = () => {
    updateStateOptimistically(
      (current) => {
        current.machine_state = "Stopped";
        current.led_on = [false, false, false, false, false, false, false, false];
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "Reset", value: null },
        }),
    );
  };

  const setAutomaticDelay = (delay_ms: number) => {
    updateStateOptimistically(
      (current) => {
        current.automatic_delay_ms = delay_ms;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetAutomaticDelay", value: { delay_ms } },
        }),
    );
  };

  return {
    state: stateOptimistic.value,
    setLed,
    setAllLeds,
    setMode,
    start,
    stop,
    reset,
    setAutomaticDelay,
  };
}