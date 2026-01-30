import React from "react";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { BarberPole } from "./BarberPole";

interface DriveAxisHeaderProps {
  name: string;
  actualPosition: number;
  setpointPosition: number;
  isMoving?: boolean;
}

/**
 * DriveAxisHeader component - Top header showing axis name, position, and movement indicator
 * Matches NcAxisHeader from TwinSharp .NET (464x48px)
 */
export function DriveAxisHeader({
  name,
  actualPosition,
  setpointPosition,
  isMoving = false,
}: DriveAxisHeaderProps) {
  return (
    <div className="flex items-center gap-3 p-2 bg-card border rounded-lg">
      {/* BarberPole - 36x36px */}
      <BarberPole active={isMoving} size={36} />

      <div className="flex flex-col gap-1 flex-1">
        <Label className="text-xs text-muted-foreground">{name}</Label>
        
        <div className="flex gap-2 items-center">
          {/* Actual Position - Large display like .NET (Font 16pt, 264x36px) */}
          <Input
            value={actualPosition.toFixed(4)}
            readOnly
            className="h-9 text-base font-mono font-semibold bg-muted flex-1 max-w-[264px]"
          />
          
          {/* Setpoint Position - Smaller display (150x23px) */}
          <div className="flex items-center gap-1">
            <Label className="text-xs text-muted-foreground whitespace-nowrap">Setpoint:</Label>
            <Input
              value={setpointPosition.toFixed(4)}
              readOnly
              className="h-6 text-xs font-mono bg-muted w-[150px]"
            />
          </div>
        </div>
      </div>
    </div>
  );
}
