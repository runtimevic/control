import React from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { 
  ChevronsLeft, 
  ChevronLeft, 
  ChevronRight, 
  ChevronsRight,
  Play,
  Square,
  RotateCcw,
  Home
} from "lucide-react";

interface JogControlsProps {
  onJogStart: (direction: 'fast-' | 'slow-' | 'slow+' | 'fast+') => void;
  onJogStop: () => void;
  onStart: () => void;
  onStop: () => void;
  onReset: () => void;
  onReference: () => void;
  disabled?: boolean;
}

export function JogControls({
  onJogStart,
  onJogStop,
  onStart,
  onStop,
  onReset,
  onReference,
  disabled = false,
}: JogControlsProps) {
  return (
    <Card>
      <CardContent className="pt-6 space-y-4">
        {/* JOG Buttons */}
        <div className="grid grid-cols-4 gap-2">
          {/* JOG Buttons - 52x52px with orange gradient like .NET */}
          <Button
            variant="default"
            className="h-[52px] w-[52px] flex flex-col justify-center items-center gap-0 bg-gradient-to-br from-orange-500 to-orange-600 hover:from-orange-600 hover:to-orange-700 text-white text-xs leading-tight p-1"
            onMouseDown={() => onJogStart('fast-')}
            onMouseUp={onJogStop}
            onMouseLeave={onJogStop}
            disabled={disabled}
          >
            <span>--</span>
            <span className="text-[10px]">F1</span>
          </Button>

          <Button
            variant="default"
            className="h-[52px] w-[52px] flex flex-col justify-center items-center gap-0 bg-gradient-to-br from-orange-500 to-orange-600 hover:from-orange-600 hover:to-orange-700 text-white text-xs leading-tight p-1"
            onMouseDown={() => onJogStart('slow-')}
            onMouseUp={onJogStop}
            onMouseLeave={onJogStop}
            disabled={disabled}
          >
            <span>-</span>
            <span className="text-[10px]">F2</span>
          </Button>

          <Button
            variant="default"
            className="h-[52px] w-[52px] flex flex-col justify-center items-center gap-0 bg-gradient-to-br from-orange-500 to-orange-600 hover:from-orange-600 hover:to-orange-700 text-white text-xs leading-tight p-1"
            onMouseDown={() => onJogStart('slow+')}
            onMouseUp={onJogStop}
            onMouseLeave={onJogStop}
            disabled={disabled}
          >
            <span>+</span>
            <span className="text-[10px]">F3</span>
          </Button>

          <Button
            variant="default"
            className="h-[52px] w-[52px] flex flex-col justify-center items-center gap-0 bg-gradient-to-br from-orange-500 to-orange-600 hover:from-orange-600 hover:to-orange-700 text-white text-xs leading-tight p-1"
            onMouseDown={() => onJogStart('fast+')}
            onMouseUp={onJogStop}
            onMouseLeave={onJogStop}
            disabled={disabled}
          >
            <span>++</span>
            <span className="text-[10px]">F4</span>
          </Button>
        </div>

        {/* Control Buttons - matching .NET colors: Start=Green, Stop=Red, Reset/Ref=Blue */}
        <div className="flex gap-1">
          <Button
            variant="default"
            className="h-[52px] w-[52px] flex flex-col justify-center items-center gap-0 bg-gradient-to-br from-green-500 to-green-700 hover:from-green-600 hover:to-green-800 text-white text-xs leading-tight p-1"
            onClick={onStart}
            disabled={disabled}
          >
            <span>Start</span>
            <span className="text-[10px]">F5</span>
          </Button>

          <Button
            variant="destructive"
            className="h-[52px] w-[52px] flex flex-col justify-center items-center gap-0 bg-gradient-to-br from-red-500 to-red-700 hover:from-red-600 hover:to-red-800 text-white text-xs leading-tight p-1"
            onClick={onStop}
            disabled={disabled}
          >
            <span>Stop</span>
            <span className="text-[10px]">F6</span>
          </Button>

          <div className="w-10" /> {/* Spacer como en el .NET */}

          <Button
            variant="default"
            className="h-[52px] w-[52px] flex flex-col justify-center items-center gap-0 bg-gradient-to-br from-blue-500 to-blue-700 hover:from-blue-600 hover:to-blue-800 text-white text-xs leading-tight p-1"
            onClick={onReset}
            disabled={disabled}
          >
            <span>Reset</span>
            <span className="text-[10px]">F8</span>
          </Button>

          <Button
            variant="default"
            className="h-[52px] w-[52px] flex flex-col justify-center items-center gap-0 bg-gradient-to-br from-blue-500 to-blue-700 hover:from-blue-600 hover:to-blue-800 text-white text-xs leading-tight p-1"
            onClick={onReference}
            disabled={disabled}
          >
            <span>Ref.</span>
            <span className="text-[10px]">F9</span>
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
