import { Page } from "@/components/Page";
import { ControlGrid } from "@/control/ControlGrid";
import { ControlCard } from "@/control/ControlCard";
import { Label } from "@/control/Label";
import { Value } from "@/components/Value";
import { Button } from "@/components/ui/button";
import { NumberInputWithSubmit } from "@/components/NumberInputWithSubmit";
import { MachineIdentificationUnique } from "@/machines/types";
import { useServoTestMachine } from "./useServoTestMachine";

interface ServoTestMachineControlProps {
  machineIdentification: MachineIdentificationUnique;
}

export function ServoTestMachineControl({
  machineIdentification,
}: ServoTestMachineControlProps) {
  const {
    driveState,
    isLoading,
    isDisabled,
    jogForward,
    jogBackward,
    jogStop,
    start,
    stop,
    reset,
    reference,
    downloadTargetPosition,
    downloadKvFactor,
    downloadRefVelocity,
    setProfileVelocity,
  } = useServoTestMachine(machineIdentification);

  return (
    <Page>
      <ControlGrid>
        {/* Status Card */}
        <ControlCard title="Drive Status">
          <div className="grid grid-cols-2 gap-4">
            <Label>
              Position
              <Value
                value={driveState?.position.toFixed(2) ?? "—"}
                unit="units"
              />
            </Label>
            <Label>
              Setpoint
              <Value
                value={driveState?.setpoint_position.toFixed(2) ?? "—"}
                unit="units"
              />
            </Label>
            <Label>
              Velocity
              <Value
                value={driveState?.velocity.toFixed(2) ?? "—"}
                unit="units/s"
              />
            </Label>
            <Label>
              Error Code
              <Value value={driveState?.error_code?.toString() ?? "—"} />
            </Label>
          </div>
        </ControlCard>

        {/* Status Flags */}
        <ControlCard title="Status Flags">
          <div className="grid grid-cols-3 gap-2 text-sm">
            <div
              className={
                driveState?.ready ? "text-green-500" : "text-gray-400"
              }
            >
              {driveState?.ready ? "✓" : "○"} Ready
            </div>
            <div
              className={
                driveState?.calibrated ? "text-green-500" : "text-gray-400"
              }
            >
              {driveState?.calibrated ? "✓" : "○"} Calibrated
            </div>
            <div
              className={
                driveState?.controller_enabled
                  ? "text-green-500"
                  : "text-gray-400"
              }
            >
              {driveState?.controller_enabled ? "✓" : "○"} Enabled
            </div>
            <div
              className={
                driveState?.in_target_pos ? "text-green-500" : "text-gray-400"
              }
            >
              {driveState?.in_target_pos ? "✓" : "○"} In Position
            </div>
            <div
              className={
                driveState?.moving_forward ? "text-blue-500" : "text-gray-400"
              }
            >
              {driveState?.moving_forward ? "▶" : "○"} Forward
            </div>
            <div
              className={
                driveState?.moving_backward ? "text-blue-500" : "text-gray-400"
              }
            >
              {driveState?.moving_backward ? "◀" : "○"} Backward
            </div>
          </div>
        </ControlCard>

        {/* Basic Controls */}
        <ControlCard title="Basic Controls">
          <div className="flex flex-wrap gap-2">
            <Button onClick={start} disabled={isDisabled || isLoading}>
              Start
            </Button>
            <Button onClick={stop} disabled={isDisabled || isLoading}>
              Stop
            </Button>
            <Button onClick={reset} disabled={isDisabled || isLoading}>
              Reset
            </Button>
            <Button onClick={reference} disabled={isDisabled || isLoading}>
              Reference
            </Button>
          </div>
        </ControlCard>

        {/* Jog Controls */}
        <ControlCard title="Jog Control">
          <div className="flex flex-wrap gap-2">
            <Button
              onPointerDown={jogForward}
              onPointerUp={jogStop}
              onPointerLeave={jogStop}
              disabled={isDisabled || isLoading}
            >
              Jog Forward
            </Button>
            <Button
              onPointerDown={jogBackward}
              onPointerUp={jogStop}
              onPointerLeave={jogStop}
              disabled={isDisabled || isLoading}
            >
              Jog Backward
            </Button>
          </div>
        </ControlCard>

        {/* Position Control */}
        <ControlCard title="Position Control (CSP Mode)">
          <div className="space-y-4">
            <Label>
              Target Position
              <NumberInputWithSubmit
                value={driveState?.setpoint_position ?? 0}
                onSubmit={downloadTargetPosition}
                disabled={isDisabled || isLoading}
                step={1}
                unit="units"
              />
            </Label>
            <Label>
              Profile Velocity
              <NumberInputWithSubmit
                value={3000}
                onSubmit={setProfileVelocity}
                disabled={isDisabled || isLoading}
                step={100}
                min={100}
                max={50000}
                unit="units/s"
              />
            </Label>
            <p className="text-xs text-muted-foreground">
              CSP mode: Set profile velocity, then target position. The servo moves at the configured velocity.
            </p>
          </div>
        </ControlCard>

        {/* Parameters */}
        <ControlCard title="Parameters">
          <div className="space-y-4">
            <Label>
              KV Factor
              <NumberInputWithSubmit
                value={1.0}
                onSubmit={downloadKvFactor}
                disabled={isDisabled || isLoading}
                step={0.1}
                min={0.1}
                max={10}
              />
            </Label>
            <Label>
              Reference Velocity
              <NumberInputWithSubmit
                value={100}
                onSubmit={downloadRefVelocity}
                disabled={isDisabled || isLoading}
                step={10}
                min={1}
                max={1000}
                unit="units/s"
              />
            </Label>
          </div>
        </ControlCard>

        {/* Detailed State */}
        <ControlCard title="Detailed State">
          <div className="grid grid-cols-2 gap-4 text-sm">
            <Label>
              Lag Error
              <Value
                value={driveState?.lag_error_current.toFixed(3) ?? "—"}
                unit="units"
              />
            </Label>
            <Label>
              Override
              <Value
                value={
                  driveState?.override
                    ? `${(driveState.override / 100).toFixed(0)}%`
                    : "—"
                }
              />
            </Label>
            <Label>
              Output
              <Value
                value={
                  driveState?.output_percent
                    ? `${driveState.output_percent.toFixed(1)}%`
                    : "—"
                }
              />
            </Label>
            <Label>
              Actual Velocity
              <Value
                value={driveState?.actual_velocity.toFixed(2) ?? "—"}
                unit="units/s"
              />
            </Label>
          </div>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
