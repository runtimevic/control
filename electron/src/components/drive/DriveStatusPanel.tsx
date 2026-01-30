import React from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { DriveState } from "./types";

interface DriveStatusPanelProps {
  state: DriveState;
  kvFactor: number;
  refVelocity: number;
  targetPosition: number;
  onSetEnabling: () => void;
  onDownloadKvFactor: () => void;
  onDownloadRefVelocity: () => void;
  onDownloadTargetPosition: () => void;
  onKvFactorChange: (value: number) => void;
  onRefVelocityChange: (value: number) => void;
  onTargetPositionChange: (value: number) => void;
}

export function DriveStatusPanel({
  state,
  kvFactor,
  refVelocity,
  targetPosition,
  onSetEnabling,
  onDownloadKvFactor,
  onDownloadRefVelocity,
  onDownloadTargetPosition,
  onKvFactorChange,
  onRefVelocityChange,
  onTargetPositionChange,
}: DriveStatusPanelProps) {
  const formatValue = (value: number, decimals: number = 4): string => {
    return value.toFixed(decimals);
  };

  return (
    <div className="space-y-4">
      {/* Status Display */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm">Axis Status</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3 text-sm">
          <div className="grid grid-cols-[120px_1fr] gap-2 items-center">
            <Label className="text-muted-foreground">Error:</Label>
            <div className={state.errorCode !== 0 ? "text-red-500 font-mono" : "font-mono"}>
              {state.errorCode.toString()} (0x{state.errorCode.toString(16).toUpperCase()})
            </div>
          </div>

          <div className="grid grid-cols-[120px_1fr] gap-2 items-center">
            <Label className="text-muted-foreground">Lag Error:</Label>
            <div className="font-mono">
              {formatValue(state.lagError.current)} ({formatValue(state.lagError.min, 3)}, {formatValue(state.lagError.max, 3)})
            </div>
          </div>

          <div className="grid grid-cols-[120px_1fr] gap-2 items-center">
            <Label className="text-muted-foreground">Actual Velocity:</Label>
            <div className="font-mono">{formatValue(state.actualVelocity)}</div>
          </div>

          <div className="grid grid-cols-[120px_1fr] gap-2 items-center">
            <Label className="text-muted-foreground">Setpoint Velocity:</Label>
            <div className="font-mono">{formatValue(state.setpointVelocity)}</div>
          </div>

          <div className="grid grid-cols-[120px_1fr] gap-2 items-center">
            <Label className="text-muted-foreground">Override:</Label>
            <div className="font-mono">{formatValue(state.override / 10000, 4)}</div>
          </div>

          <div className="grid grid-cols-[120px_1fr] gap-2 items-center">
            <Label className="text-muted-foreground">Output:</Label>
            <div className="font-mono">
              {formatValue(state.outputPercent * 100, 2)} / {formatValue(state.controllerOutputPercent * 100, 2)}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Status Checkboxes - Logical */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm">Status (logical)</CardTitle>
        </CardHeader>
        <CardContent className="pt-0">
          <div className="grid grid-cols-2 gap-3">
            <div className="flex items-center space-x-2">
              <Checkbox id="ready" checked={state.ready} disabled />
              <Label htmlFor="ready" className="text-sm cursor-pointer">Ready</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox id="calibrated" checked={state.calibrated} disabled />
              <Label htmlFor="calibrated" className="text-sm cursor-pointer">Calibrated</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox id="hasJob" checked={state.hasJob} disabled />
              <Label htmlFor="hasJob" className="text-sm cursor-pointer">Has Job</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox id="notMoving" checked={state.notMoving} disabled />
              <Label htmlFor="notMoving" className="text-sm cursor-pointer">Not Moving</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox id="movingFw" checked={state.movingForward} disabled />
              <Label htmlFor="movingFw" className="text-sm cursor-pointer">Moving Fw</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox id="movingBw" checked={state.movingBackward} disabled />
              <Label htmlFor="movingBw" className="text-sm cursor-pointer">Moving Bw</Label>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Status Checkboxes - Physical */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm">Status (physical)</CardTitle>
        </CardHeader>
        <CardContent className="pt-0">
          <div className="grid grid-cols-2 gap-3">
            <div className="flex items-center space-x-2">
              <Checkbox id="coupledMode" checked={state.coupledMode} disabled />
              <Label htmlFor="coupledMode" className="text-sm cursor-pointer">Coupled Mode</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox id="inTargetPos" checked={state.inTargetPos} disabled />
              <Label htmlFor="inTargetPos" className="text-sm cursor-pointer">In Target Pos</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox id="inPosRange" checked={state.inPosRange} disabled />
              <Label htmlFor="inPosRange" className="text-sm cursor-pointer">In Pos Range</Label>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Enabling */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm">Enabling</CardTitle>
        </CardHeader>
        <CardContent className="pt-0 space-y-3">
          <div className="grid grid-cols-2 gap-3">
            <div className="flex items-center space-x-2">
              <Checkbox id="controller" checked={state.controllerEnabled} disabled />
              <Label htmlFor="controller" className="text-sm cursor-pointer">Controller</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox id="feedFw" checked={state.feedFwEnabled} disabled />
              <Label htmlFor="feedFw" className="text-sm cursor-pointer">Feed Fw</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox id="feedBw" checked={state.feedBwEnabled} disabled />
              <Label htmlFor="feedBw" className="text-sm cursor-pointer">Feed Bw</Label>
            </div>
          </div>
          <Button onClick={onSetEnabling} className="w-full" variant="outline" size="sm">
            Set...
          </Button>
        </CardContent>
      </Card>

      {/* Configuration */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm">Configuration</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
          <div className="flex items-center gap-2">
            <Label htmlFor="kvFactor" className="text-sm w-24 shrink-0">Kv Factor:</Label>
            <Input
              id="kvFactor"
              type="number"
              value={kvFactor}
              onChange={(e) => onKvFactorChange(parseFloat(e.target.value))}
              className="flex-1 h-8"
              step="0.01"
            />
            <Button onClick={onDownloadKvFactor} size="sm" variant="outline" className="h-8 w-8 p-0">↓</Button>
          </div>

          <div className="flex items-center gap-2">
            <Label htmlFor="refVelocity" className="text-sm w-24 shrink-0">Ref Velocity:</Label>
            <Input
              id="refVelocity"
              type="number"
              value={refVelocity}
              onChange={(e) => onRefVelocityChange(parseFloat(e.target.value))}
              className="flex-1 h-8"
              step="0.1"
            />
            <Button onClick={onDownloadRefVelocity} size="sm" variant="outline" className="h-8 w-8 p-0">↓</Button>
          </div>

          <div className="flex items-center gap-2">
            <Label htmlFor="targetPosition" className="text-sm w-24 shrink-0">Target Pos:</Label>
            <Input
              id="targetPosition"
              type="number"
              value={targetPosition}
              onChange={(e) => onTargetPositionChange(parseFloat(e.target.value))}
              className="flex-1 h-8"
              step="1"
            />
            <Button onClick={onDownloadTargetPosition} size="sm" variant="outline" className="h-8 w-8 p-0">↓</Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
