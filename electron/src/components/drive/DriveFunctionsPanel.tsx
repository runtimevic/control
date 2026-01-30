import React, { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { 
  MovementMode, 
  MOVEMENT_MODES, 
  DriveControlParams,
  RawDriveOutputMode,
  SetPositionMode,
  SetTargetMode 
} from "./types";

interface DriveFunctionsPanelProps {
  params: DriveControlParams;
  lastTime: number;
  onParamsChange: (params: Partial<DriveControlParams>) => void;
  onStart: () => void;
  onStop: () => void;
  onStartRawOutput: (mode: RawDriveOutputMode, value: number) => void;
  onStopRawOutput: () => void;
  onSetActualPosition: (mode: SetPositionMode, value: number) => void;
  onSetTargetPosition: (mode: SetTargetMode, value: number) => void;
}

export function DriveFunctionsPanel({
  params,
  lastTime,
  onParamsChange,
  onStart,
  onStop,
  onStartRawOutput,
  onStopRawOutput,
  onSetActualPosition,
  onSetTargetPosition,
}: DriveFunctionsPanelProps) {
  const [movementMode, setMovementMode] = useState<MovementMode>('Absolute');
  const [rawOutputMode, setRawOutputMode] = useState<RawDriveOutputMode>('Percent');
  const [rawOutputValue, setRawOutputValue] = useState<number>(0);
  const [actualPosMode, setActualPosMode] = useState<SetPositionMode>('Absolute');
  const [actualPosValue, setActualPosValue] = useState<number>(0);
  const [targetPosMode, setTargetPosMode] = useState<SetTargetMode>('Absolute');
  const [targetPosValue, setTargetPosValue] = useState<number>(0);

  // Determine which parameters to show based on movement mode
  const showStandardUI = [
    'Absolute', 'Relative', 'EndlessPos', 'EndlessNeg', 'Modulo',
    'ModuloShortestWay', 'ModuloPosDirection', 'ModuloNegDirection',
    'JogPos', 'JogNeg', 'PlusOne', 'PlusTenth', 'PlusHundredth', 'PlusThousandth',
    'MinusOne', 'MinusTenth', 'MinusHundredth', 'MinusThousandth'
  ].includes(movementMode);

  return (
    <div className="space-y-4">
      {/* Movement Mode Selection */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm">Movement Mode</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="startMode">Start Mode:</Label>
            <Select value={movementMode} onValueChange={(v) => setMovementMode(v as MovementMode)}>
              <SelectTrigger id="startMode">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {MOVEMENT_MODES.map((mode) => (
                  <SelectItem key={mode.value} value={mode.value}>
                    {mode.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Movement Parameters */}
          {showStandardUI && (
            <div className="space-y-3">
              <div className="space-y-2">
                <Label htmlFor="targetPos">Target Position:</Label>
                <Input
                  id="targetPos"
                  type="number"
                  value={params.targetPosition}
                  onChange={(e) => onParamsChange({ targetPosition: parseFloat(e.target.value) })}
                  step="1"
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="targetVel">Target Velocity:</Label>
                <Input
                  id="targetVel"
                  type="number"
                  value={params.targetVelocity}
                  onChange={(e) => onParamsChange({ targetVelocity: parseFloat(e.target.value) })}
                  step="0.1"
                />
              </div>

              <div className="space-y-2">
                <div className="flex items-center space-x-2">
                  <Checkbox
                    id="enableAccel"
                    checked={params.enableAcceleration}
                    onCheckedChange={(checked) => onParamsChange({ enableAcceleration: checked as boolean })}
                  />
                  <Label htmlFor="enableAccel" className="cursor-pointer">Enable Acceleration</Label>
                </div>
                <Input
                  type="number"
                  value={params.acceleration}
                  onChange={(e) => onParamsChange({ acceleration: parseFloat(e.target.value) })}
                  disabled={!params.enableAcceleration}
                  step="0.1"
                />
              </div>

              <div className="space-y-2">
                <div className="flex items-center space-x-2">
                  <Checkbox
                    id="enableDecel"
                    checked={params.enableDeceleration}
                    onCheckedChange={(checked) => onParamsChange({ enableDeceleration: checked as boolean })}
                  />
                  <Label htmlFor="enableDecel" className="cursor-pointer">Enable Deceleration</Label>
                </div>
                <Input
                  type="number"
                  value={params.deceleration}
                  onChange={(e) => onParamsChange({ deceleration: parseFloat(e.target.value) })}
                  disabled={!params.enableDeceleration}
                  step="0.1"
                />
              </div>

              <div className="space-y-2">
                <div className="flex items-center space-x-2">
                  <Checkbox
                    id="enableJerk"
                    checked={params.enableJerk}
                    onCheckedChange={(checked) => onParamsChange({ enableJerk: checked as boolean })}
                  />
                  <Label htmlFor="enableJerk" className="cursor-pointer">Enable Jerk</Label>
                </div>
                <Input
                  type="number"
                  value={params.jerk}
                  onChange={(e) => onParamsChange({ jerk: parseFloat(e.target.value) })}
                  disabled={!params.enableJerk}
                  step="0.1"
                />
              </div>
            </div>
          )}

          <div className="flex gap-2 pt-2">
            <Button onClick={onStart} className="flex-1 bg-green-600 hover:bg-green-700">
              START
            </Button>
            <Button onClick={onStop} variant="destructive" className="flex-1">
              STOP
            </Button>
          </div>

          <div className="text-sm text-muted-foreground">
            Last Time: <span className="font-mono">{lastTime.toFixed(3)} s</span>
          </div>
        </CardContent>
      </Card>

      {/* Raw Drive Output */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm">Raw Drive Output</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="grid grid-cols-2 gap-2">
            <div className="space-y-2">
              <Label htmlFor="rawMode">Mode:</Label>
              <Select value={rawOutputMode} onValueChange={(v) => setRawOutputMode(v as RawDriveOutputMode)}>
                <SelectTrigger id="rawMode">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="Percent">Percent</SelectItem>
                  <SelectItem value="Velocity">Velocity</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="rawValue">Value:</Label>
              <Input
                id="rawValue"
                type="number"
                value={rawOutputValue}
                onChange={(e) => setRawOutputValue(parseFloat(e.target.value))}
                step="1"
              />
            </div>
          </div>
          <div className="flex gap-2">
            <Button 
              onClick={() => onStartRawOutput(rawOutputMode, rawOutputValue)} 
              className="flex-1"
              size="sm"
            >
              Start Output
            </Button>
            <Button onClick={onStopRawOutput} variant="outline" className="flex-1" size="sm">
              Stop Output
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Set Actual Position */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm">Set Actual Position</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="grid grid-cols-2 gap-2">
            <div className="space-y-2">
              <Label htmlFor="actualPosMode">Type:</Label>
              <Select value={actualPosMode} onValueChange={(v) => setActualPosMode(v as SetPositionMode)}>
                <SelectTrigger id="actualPosMode">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="Absolute">Absolute</SelectItem>
                  <SelectItem value="Relative">Relative</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="actualPosValue">Value:</Label>
              <Input
                id="actualPosValue"
                type="number"
                value={actualPosValue}
                onChange={(e) => setActualPosValue(parseFloat(e.target.value))}
                step="1"
              />
            </div>
          </div>
          <Button 
            onClick={() => onSetActualPosition(actualPosMode, actualPosValue)}
            className="w-full"
            size="sm"
          >
            Set Position
          </Button>
        </CardContent>
      </Card>

      {/* Set Target Position */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm">Set Target Position</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="grid grid-cols-2 gap-2">
            <div className="space-y-2">
              <Label htmlFor="targetPosMode">Type:</Label>
              <Select value={targetPosMode} onValueChange={(v) => setTargetPosMode(v as SetTargetMode)}>
                <SelectTrigger id="targetPosMode">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="Absolute">Absolute</SelectItem>
                  <SelectItem value="Relative">Relative</SelectItem>
                  <SelectItem value="EndlessPos">Endless +</SelectItem>
                  <SelectItem value="EndlessNeg">Endless -</SelectItem>
                  <SelectItem value="Modulo">Modulo</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="targetPosValue">Value:</Label>
              <Input
                id="targetPosValue"
                type="number"
                value={targetPosValue}
                onChange={(e) => setTargetPosValue(parseFloat(e.target.value))}
                step="1"
              />
            </div>
          </div>
          <Button 
            onClick={() => onSetTargetPosition(targetPosMode, targetPosValue)}
            className="w-full"
            size="sm"
          >
            Set Target
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
