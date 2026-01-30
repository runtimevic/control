/**
 * @file driveNamespace.ts
 * @description Socket.io namespace implementation for servo/drive control (CiA402)
 * Follows the same pattern as winder2Namespace.ts, extruder2Namespace.ts, etc.
 */

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
import { useMemo } from "react";

// ========== Event Schema Definitions ==========

/**
 * Live drive state event (sent every 100ms from backend)
 * Replaces the polling timer approach
 */
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
  override: z.number(), // 0-10000 (100.00 = 100%)
  output_percent: z.number(),
  controller_output_percent: z.number(),
  // Status flags (logical)
  ready: z.boolean(),
  calibrated: z.boolean(),
  has_job: z.boolean(),
  not_moving: z.boolean(),
  moving_forward: z.boolean(),
  moving_backward: z.boolean(),
  // Status flags (physical)
  coupled_mode: z.boolean(),
  in_target_pos: z.boolean(),
  in_pos_range: z.boolean(),
  // Enabling
  controller_enabled: z.boolean(),
  feed_fw_enabled: z.boolean(),
  feed_bw_enabled: z.boolean(),
});

export type DriveStateEventData = z.infer<typeof driveStateEventDataSchema>;

/**
 * Event for command completion/acknowledgment
 */
export const commandAckEventDataSchema = z.object({
  command: z.string(),
  success: z.boolean(),
  error_message: z.string().optional(),
});

export type CommandAckEventData = z.infer<typeof commandAckEventDataSchema>;

/**
 * Event for movement completion
 */
export const movementCompleteEventDataSchema = z.object({
  final_position: z.number(),
  time_ms: z.number(),
});

export type MovementCompleteEventData = z.infer<
  typeof movementCompleteEventDataSchema
>;

// ========== Store Schema ==========

export interface DriveNamespaceStore {
  name: string;
  // Current state
  position: number;
  setpointPosition: number;
  velocity: number;
  errorCode: number;
  lagError: {
    current: number;
    min: number;
    max: number;
  };
  actualVelocity: number;
  setpointVelocity: number;
  override: number;
  outputPercent: number;
  controllerOutputPercent: number;
  // Status flags
  ready: boolean;
  calibrated: boolean;
  hasJob: boolean;
  notMoving: boolean;
  movingForward: boolean;
  movingBackward: boolean;
  coupledMode: boolean;
  inTargetPos: boolean;
  inPosRange: boolean;
  controllerEnabled: boolean;
  feedFwEnabled: boolean;
  feedBwEnabled: boolean;
  // Last movement time
  lastMovementTimeMs: number;
  // Connection status
  isConnected: boolean;
}

/**
 * Create initial store state
 */
export function createDriveNamespaceStore(): StoreApi<DriveNamespaceStore> {
  return create<DriveNamespaceStore>(() => ({
    name: "Drive",
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
    ready: false,
    calibrated: false,
    hasJob: false,
    notMoving: true,
    movingForward: false,
    movingBackward: false,
    coupledMode: false,
    inTargetPos: true,
    inPosRange: true,
    controllerEnabled: false,
    feedFwEnabled: false,
    feedBwEnabled: false,
    lastMovementTimeMs: 0,
    isConnected: false,
  }));
}

// ========== Event Handler ==========

/**
 * Message handler for drive namespace events
 */
export function driveMessageHandler(
  store: StoreApi<DriveNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<DriveNamespaceStore>,
): EventHandler {
  // Helper to update store through throttled updater
  const updateStore = (updater: (state: DriveNamespaceStore) => DriveNamespaceStore) => {
    throttledUpdater.updateWith(updater);
  };

  return (event: Event<any>) => {
    const eventName = event.name;
    
    try {
      if (eventName === "DriveStateEvent") {
        // Parse live drive state
        const driveStateEvent = driveStateEventDataSchema.parse(event.data);

        updateStore((state) => ({
          ...state,
          position: driveStateEvent.position,
          setpointPosition: driveStateEvent.setpoint_position,
          velocity: driveStateEvent.velocity,
          errorCode: driveStateEvent.error_code,
          lagError: {
            current: driveStateEvent.lag_error_current,
            min: driveStateEvent.lag_error_min,
            max: driveStateEvent.lag_error_max,
          },
          actualVelocity: driveStateEvent.actual_velocity,
          setpointVelocity: driveStateEvent.setpoint_velocity,
          override: driveStateEvent.override,
          outputPercent: driveStateEvent.output_percent,
          controllerOutputPercent: driveStateEvent.controller_output_percent,
          ready: driveStateEvent.ready,
          calibrated: driveStateEvent.calibrated,
          hasJob: driveStateEvent.has_job,
          notMoving: driveStateEvent.not_moving,
          movingForward: driveStateEvent.moving_forward,
          movingBackward: driveStateEvent.moving_backward,
          coupledMode: driveStateEvent.coupled_mode,
          inTargetPos: driveStateEvent.in_target_pos,
          inPosRange: driveStateEvent.in_pos_range,
          controllerEnabled: driveStateEvent.controller_enabled,
          feedFwEnabled: driveStateEvent.feed_fw_enabled,
          feedBwEnabled: driveStateEvent.feed_bw_enabled,
          isConnected: true,
        }));
      } else if (eventName === "CommandAckEvent") {
        // Parse command acknowledgment
        const commandAck = commandAckEventDataSchema.parse(event.data);
        console.log(
          `[Drive] Command ${commandAck.command} ${commandAck.success ? "succeeded" : "failed"}`,
          commandAck.error_message,
        );
        // Could update UI with toast notification here
      } else if (eventName === "MovementCompleteEvent") {
        // Parse movement completion
        const movementComplete = movementCompleteEventDataSchema.parse(event.data);
        updateStore((state) => ({
          ...state,
          lastMovementTimeMs: movementComplete.time_ms,
          position: movementComplete.final_position,
        }));
        console.log(
          `[Drive] Movement complete in ${movementComplete.time_ms}ms, final position: ${movementComplete.final_position}`,
        );
      } else {
        handleUnhandledEventError(eventName);
      }
    } catch (error) {
      console.error(`Unexpected error processing ${eventName} event:`, error);
      throw error;
    }
  };
}

/**
 * Create the Drive namespace implementation
 */
const useDriveNamespaceImplementation =
  createNamespaceHookImplementation<DriveNamespaceStore>({
    createStore: createDriveNamespaceStore,
    createEventHandler: driveMessageHandler,
  });

/**
 * Hook for a machine-specific Drive namespace
 * 
 * @example
 * ```tsx
 * function DriveControl({ machine }) {
 *   const driveState = useDriveNamespace(machine);
 *   const { emit } = useSocketEmit();
 * 
 *   const handleJogStart = (direction: string) => {
 *     emit('jog_start', { direction });
 *   };
 * 
 *   return <div>Position: {driveState.position}</div>;
 * }
 * ```
 */
export function useDriveNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): DriveNamespaceStore {
  // Generate namespace ID from validated machine ID
  const namespaceId = useMemo<NamespaceId>(
    () => ({
      type: "machine",
      machine_identification_unique,
    }),
    [machine_identification_unique],
  );

  // Use the implementation with validated namespace ID
  return useDriveNamespaceImplementation(namespaceId);
}
