import { useCallback, useMemo } from "react";
import { z } from "zod";
import { useMachineState, useMachineMutation } from "@/client/machine";
import { MachineIdentificationUnique } from "@/machines/types";

// DriveStateEvent schema matching backend
export const driveStateEventDataSchema = z.object({
  position: z.number(),
  setpoint_position: z.number(),
  velocity: z.number(),
  error_code: z.number(),
  lag_error_current: z.number(),
  lag_error_min: z.number(),
  lag_error_max: z.number(),
  actual_velocity: z.number(),
  setpoint_velocity: z.number(),
  override: z.number(),
  output_percent: z.number(),
  controller_output_percent: z.number(),
  // Status flags
  ready: z.boolean(),
  calibrated: z.boolean(),
  has_job: z.boolean(),
  not_moving: z.boolean(),
  moving_forward: z.boolean(),
  moving_backward: z.boolean(),
  coupled_mode: z.boolean(),
  in_target_pos: z.boolean(),
  in_pos_range: z.boolean(),
  controller_enabled: z.boolean(),
  feed_fw_enabled: z.boolean(),
  feed_bw_enabled: z.boolean(),
});

export type DriveStateEventData = z.infer<typeof driveStateEventDataSchema>;

// Command acknowledgment event
const commandAckEventDataSchema = z.object({
  command: z.string(),
  success: z.boolean(),
  error_message: z.string().nullable().optional(),
});

export type CommandAckEventData = z.infer<typeof commandAckEventDataSchema>;

// Movement complete event
const movementCompleteEventDataSchema = z.object({
  final_position: z.number(),
  time_ms: z.number(),
});

export type MovementCompleteEventData = z.infer<
  typeof movementCompleteEventDataSchema
>;

// Mutation schemas
const jogStartMutation = z.object({
  JogStart: z.string(),
});

const setEnablingMutation = z.object({
  SetEnabling: z.object({
    controller: z.boolean(),
    feedFw: z.boolean(),
    feedBw: z.boolean(),
    override: z.number(),
  }),
});

const downloadKvFactorMutation = z.object({
  DownloadKvFactor: z.number(),
});

const downloadRefVelocityMutation = z.object({
  DownloadRefVelocity: z.number(),
});

const downloadTargetPositionMutation = z.object({
  DownloadTargetPosition: z.number(),
});

const startMovementMutation = z.object({
  StartMovement: z.object({
    mode: z.string(),
    targetPosition: z.number().optional(),
    targetVelocity: z.number().optional(),
    acceleration: z.number().optional(),
  }),
});

const setRawOutputMutation = z.object({
  SetRawOutput: z.number(),
});

// Events schema
const servoTestMachineEventsSchema = z.object({
  DriveStateEvent: driveStateEventDataSchema,
  CommandAckEvent: commandAckEventDataSchema.optional(),
  MovementCompleteEvent: movementCompleteEventDataSchema.optional(),
});

export type ServoTestMachineEvents = z.infer<
  typeof servoTestMachineEventsSchema
>;

export function useServoTestMachine(
  machineIdentification: MachineIdentificationUnique,
) {
  const stateOptimistic = useMachineState(
    machineIdentification,
    servoTestMachineEventsSchema,
  );

  const state = stateOptimistic.value;

  // Mutation hooks
  const { request: requestJogStart } = useMachineMutation(jogStartMutation);
  const { request: requestJogStop } = useMachineMutation(z.literal("JogStop"));
  const { request: requestStart } = useMachineMutation(z.literal("Start"));
  const { request: requestStop } = useMachineMutation(z.literal("Stop"));
  const { request: requestReset } = useMachineMutation(z.literal("Reset"));
  const { request: requestReference } = useMachineMutation(
    z.literal("Reference"),
  );
  const { request: requestSetEnabling } = useMachineMutation(
    setEnablingMutation,
  );
  const { request: requestDownloadKvFactor } = useMachineMutation(
    downloadKvFactorMutation,
  );
  const { request: requestDownloadRefVelocity } = useMachineMutation(
    downloadRefVelocityMutation,
  );
  const { request: requestDownloadTargetPosition } = useMachineMutation(
    downloadTargetPositionMutation,
  );
  const { request: requestStartMovement } = useMachineMutation(
    startMovementMutation,
  );
  const { request: requestSetRawOutput } = useMachineMutation(
    setRawOutputMutation,
  );

  // Action handlers
  const jogForward = useCallback(() => {
    requestJogStart({
      machine_identification_unique: machineIdentification,
      data: { JogStart: "forward" },
    });
  }, [machineIdentification, requestJogStart]);

  const jogBackward = useCallback(() => {
    requestJogStart({
      machine_identification_unique: machineIdentification,
      data: { JogStart: "backward" },
    });
  }, [machineIdentification, requestJogStart]);

  const jogStop = useCallback(() => {
    requestJogStop({
      machine_identification_unique: machineIdentification,
      data: "JogStop",
    });
  }, [machineIdentification, requestJogStop]);

  const start = useCallback(() => {
    requestStart({
      machine_identification_unique: machineIdentification,
      data: "Start",
    });
  }, [machineIdentification, requestStart]);

  const stop = useCallback(() => {
    requestStop({
      machine_identification_unique: machineIdentification,
      data: "Stop",
    });
  }, [machineIdentification, requestStop]);

  const reset = useCallback(() => {
    requestReset({
      machine_identification_unique: machineIdentification,
      data: "Reset",
    });
  }, [machineIdentification, requestReset]);

  const reference = useCallback(() => {
    requestReference({
      machine_identification_unique: machineIdentification,
      data: "Reference",
    });
  }, [machineIdentification, requestReference]);

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
    [machineIdentification, requestSetEnabling],
  );

  const downloadKvFactor = useCallback(
    (value: number) => {
      requestDownloadKvFactor({
        machine_identification_unique: machineIdentification,
        data: { DownloadKvFactor: value },
      });
    },
    [machineIdentification, requestDownloadKvFactor],
  );

  const downloadRefVelocity = useCallback(
    (value: number) => {
      requestDownloadRefVelocity({
        machine_identification_unique: machineIdentification,
        data: { DownloadRefVelocity: value },
      });
    },
    [machineIdentification, requestDownloadRefVelocity],
  );

  const downloadTargetPosition = useCallback(
    (value: number) => {
      requestDownloadTargetPosition({
        machine_identification_unique: machineIdentification,
        data: { DownloadTargetPosition: value },
      });
    },
    [machineIdentification, requestDownloadTargetPosition],
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
    [machineIdentification, requestStartMovement],
  );

  const setRawOutput = useCallback(
    (value: number) => {
      requestSetRawOutput({
        machine_identification_unique: machineIdentification,
        data: { SetRawOutput: value },
      });
    },
    [machineIdentification, requestSetRawOutput],
  );

  // Calculate loading states
  const isLoading = stateOptimistic.isOptimistic;
  const isDisabled = !stateOptimistic.isInitialized;

  return {
    state,
    driveState: state?.DriveStateEvent,
    isLoading,
    isDisabled,
    // Actions
    jogForward,
    jogBackward,
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
