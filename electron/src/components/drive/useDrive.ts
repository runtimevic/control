/**
 * @file useDrive.ts
 * @description Custom hook for drive control operations
 * Follows the pattern of useWinder2, useExtruder2, etc.
 * 
 * Architecture:
 * - Socket.io for real-time state updates (DriveStateEvent every 100ms)
 * - HTTP REST for command execution (useMachineMutation)
 */

import { useCallback, useEffect, useMemo } from "react";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { MachineIdentificationUnique } from "@/machines/types";
import { z } from "zod";
import { useDriveNamespace } from "./driveNamespace";
import { DriveState } from "./types";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { produce } from "immer";

export interface UseDriveProps {
  machineIdentification: MachineIdentificationUnique;
}

export interface UseDriveReturn {
  // State
  state: DriveState | undefined;
  isLoading: boolean;

  // Movement commands
  jogStart: (direction: "fast-" | "slow-" | "slow+" | "fast+") => void;
  jogStop: () => void;
  start: () => void;
  stop: () => void;

  // Control commands
  reset: () => void;
  reference: () => void;

  // Configuration commands
  setEnabling: (params: {
    controller: boolean;
    feedFw: boolean;
    feedBw: boolean;
    override: number;
  }) => void;
  downloadKvFactor: (value: number) => void;
  downloadRefVelocity: (value: number) => void;
  downloadTargetPosition: (value: number) => void;

  // Movement commands
  startMovement: (params: {
    mode: string;
    targetPosition?: number;
    targetVelocity?: number;
    acceleration?: number;
  }) => void;
  setRawOutput: (value: number) => void;
}

export function useDrive({
  machineIdentification,
}: UseDriveProps): UseDriveReturn {
  // Get live state from Socket.io namespace
  const state = useDriveNamespace(machineIdentification);

  // Optimistic state for immediate UI feedback
  const stateOptimistic = useStateOptimistic<DriveState>();

  // Update optimistic state when real state changes
  useEffect(() => {
    if (state) {
      stateOptimistic.setReal(state);
    }
  }, [state]);

  // Define all command schemas with useMachineMutation
  const { request: requestJogStart } = useMachineMutation(
    z.object({ JogStart: z.enum(["fast-", "slow-", "slow+", "fast+"]) }),
  );
  const { request: requestJogStop } = useMachineMutation(z.literal("JogStop"));
  const { request: requestStart } = useMachineMutation(z.literal("Start"));
  const { request: requestStop } = useMachineMutation(z.literal("Stop"));
  const { request: requestReset } = useMachineMutation(z.literal("Reset"));
  const { request: requestReference } = useMachineMutation(
    z.literal("Reference"),
  );

  const { request: requestSetEnabling } = useMachineMutation(
    z.object({
      SetEnabling: z.object({
        controller: z.boolean(),
        feedFw: z.boolean(),
        feedBw: z.boolean(),
        override: z.number(),
      }),
    }),
  );

  const { request: requestDownloadKvFactor } = useMachineMutation(
    z.object({ DownloadKvFactor: z.number() }),
  );

  const { request: requestDownloadRefVelocity } = useMachineMutation(
    z.object({ DownloadRefVelocity: z.number() }),
  );

  const { request: requestDownloadTargetPosition } = useMachineMutation(
    z.object({ DownloadTargetPosition: z.number() }),
  );

  const { request: requestStartMovement } = useMachineMutation(
    z.object({
      StartMovement: z.object({
        mode: z.string(),
        targetPosition: z.number().optional(),
        targetVelocity: z.number().optional(),
        acceleration: z.number().optional(),
      }),
    }),
  );

  const { request: requestSetRawOutput } = useMachineMutation(
    z.object({ SetRawOutput: z.number() }),
  );

  // Helper function for optimistic updates
  const updateStateOptimistically = useCallback(
    (producer: (current: DriveState) => void, serverRequest: () => void) => {
      const currentState = stateOptimistic.value;
      if (currentState && !stateOptimistic.isOptimistic) {
        stateOptimistic.setOptimistic(produce(currentState, producer));
      }
      serverRequest();
    },
    [stateOptimistic],
  );

  // Command implementations
  const jogStart = useCallback(
    (direction: "fast-" | "slow-" | "slow+" | "fast+") => {
      requestJogStart({
        machine_identification_unique: machineIdentification,
        data: { JogStart: direction },
      });
    },
    [requestJogStart, machineIdentification],
  );

  const jogStop = useCallback(() => {
    requestJogStop({
      machine_identification_unique: machineIdentification,
      data: "JogStop",
    });
  }, [requestJogStop, machineIdentification]);

  const start = useCallback(() => {
    requestStart({
      machine_identification_unique: machineIdentification,
      data: "Start",
    });
  }, [requestStart, machineIdentification]);

  const stop = useCallback(() => {
    requestStop({
      machine_identification_unique: machineIdentification,
      data: "Stop",
    });
  }, [requestStop, machineIdentification]);

  const reset = useCallback(() => {
    requestReset({
      machine_identification_unique: machineIdentification,
      data: "Reset",
    });
  }, [requestReset, machineIdentification]);

  const reference = useCallback(() => {
    requestReference({
      machine_identification_unique: machineIdentification,
      data: "Reference",
    });
  }, [requestReference, machineIdentification]);

  const setEnabling = useCallback(
    (params: {
      controller: boolean;
      feedFw: boolean;
      feedBw: boolean;
      override: number;
    }) => {
      requestSetEnabling({
        machine_identification_unique: machineIdentification,
        data: { SetEnabling: params },
      });
    },
    [requestSetEnabling, machineIdentification],
  );

  const downloadKvFactor = useCallback(
    (value: number) => {
      requestDownloadKvFactor({
        machine_identification_unique: machineIdentification,
        data: { DownloadKvFactor: value },
      });
    },
    [requestDownloadKvFactor, machineIdentification],
  );

  const downloadRefVelocity = useCallback(
    (value: number) => {
      requestDownloadRefVelocity({
        machine_identification_unique: machineIdentification,
        data: { DownloadRefVelocity: value },
      });
    },
    [requestDownloadRefVelocity, machineIdentification],
  );

  const downloadTargetPosition = useCallback(
    (value: number) => {
      requestDownloadTargetPosition({
        machine_identification_unique: machineIdentification,
        data: { DownloadTargetPosition: value },
      });
    },
    [requestDownloadTargetPosition, machineIdentification],
  );

  const startMovement = useCallback(
    (params: {
      mode: string;
      targetPosition?: number;
      targetVelocity?: number;
      acceleration?: number;
    }) => {
      requestStartMovement({
        machine_identification_unique: machineIdentification,
        data: { StartMovement: params },
      });
    },
    [requestStartMovement, machineIdentification],
  );

  const setRawOutput = useCallback(
    (value: number) => {
      requestSetRawOutput({
        machine_identification_unique: machineIdentification,
        data: { SetRawOutput: value },
      });
    },
    [requestSetRawOutput, machineIdentification],
  );

  return {
    state: stateOptimistic.value,
    isLoading: stateOptimistic.isOptimistic,
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
    setRawOutput,
  };
}
