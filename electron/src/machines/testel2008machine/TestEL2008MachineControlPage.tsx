import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React, { useState } from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Label } from "@/control/Label";
import { useTestEL2008Machine } from "./useTestEL2008Machine";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

export function TestEL2008MachineControlPage() {
  const { 
    state, 
    setLed, 
    setAllLeds, 
    setMode, 
    start, 
    stop, 
    reset, 
    setAutomaticDelay 
  } = useTestEL2008Machine();

  const [delayInput, setDelayInput] = useState<string>("");

  const safeState = state ?? { 
    led_on: [false, false, false, false, false, false, false, false],
    mode: "Manual" as const,
    machine_state: "Stopped" as const,
    automatic_delay_ms: 500,
  };

  const isManualMode = safeState.mode === "Manual";
  const isStopped = safeState.machine_state === "Stopped";
  const isRunning = safeState.machine_state === "Running";
  const canControlOutputs = isManualMode && isRunning;

  const handleDelayChange = () => {
    const delay = parseInt(delayInput);
    if (!isNaN(delay) && delay > 0) {
      setAutomaticDelay(delay);
      setDelayInput("");
    }
  };

  return (
    <Page>
      <ControlGrid columns={2}>
        {/* Mode Selection */}
        <ControlCard title="Mode Selection">
          <Label label="Operating Mode">
            <SelectionGroup<"Manual" | "Home" | "Automatic">
              value={safeState.mode}
              orientation="horizontal"
              className="grid grid-cols-3 gap-2"
              options={{
                Manual: {
                  children: "Manual",
                  icon: "lu:Hand",
                  isActiveClassName: "bg-blue-600",
                  disabled: !isStopped,
                },
                Home: {
                  children: "Home",
                  icon: "lu:Home",
                  isActiveClassName: "bg-yellow-600",
                  disabled: !isStopped,
                },
                Automatic: {
                  children: "Automatic",
                  icon: "lu:Cog",
                  isActiveClassName: "bg-purple-600",
                  disabled: !isStopped,
                },
              }}
              onChange={(value) => setMode(value)}
            />
          </Label>
          {!isStopped && (
            <p className="text-sm text-yellow-500 mt-2">
              Stop the machine to change mode
            </p>
          )}
        </ControlCard>

        {/* Control Buttons */}
        <ControlCard title="Machine Control">
          <div className="grid grid-cols-3 gap-4">
            <Button
              onClick={start}
              disabled={!isStopped}
              className={`h-20 ${!isStopped ? 'opacity-50' : 'bg-green-600 hover:bg-green-700'}`}
            >
              <div className="flex flex-col items-center gap-2">
                <span className="text-2xl">▶</span>
                <span>Start</span>
              </div>
            </Button>
            
            <Button
              onClick={stop}
              disabled={isStopped}
              className={`h-20 ${isStopped ? 'opacity-50' : 'bg-red-600 hover:bg-red-700'}`}
            >
              <div className="flex flex-col items-center gap-2">
                <span className="text-2xl">⏸</span>
                <span>Stop</span>
              </div>
            </Button>
            
            <Button
              onClick={reset}
              className="h-20 bg-orange-600 hover:bg-orange-700"
            >
              <div className="flex flex-col items-center gap-2">
                <span className="text-2xl">↻</span>
                <span>Reset</span>
              </div>
            </Button>
          </div>

          <div className="mt-4 p-4 bg-gray-800 rounded">
            <p className="text-sm">
              <span className="font-semibold">Status: </span>
              <span className={safeState.machine_state === "Running" ? "text-green-400" : "text-gray-400"}>
                {safeState.machine_state}
              </span>
            </p>
            <p className="text-sm mt-1">
              <span className="font-semibold">Mode: </span>
              <span className="text-blue-400">{safeState.mode}</span>
            </p>
          </div>
        </ControlCard>

        {/* Automatic Mode Configuration */}
        {safeState.mode === "Automatic" && (
          <ControlCard title="Automatic Mode Settings">
            <Label label={`Step Delay: ${safeState.automatic_delay_ms}ms`}>
              <div className="flex gap-2">
                <Input
                  type="number"
                  value={delayInput}
                  onChange={(e) => setDelayInput(e.target.value)}
                  placeholder={`Current: ${safeState.automatic_delay_ms}ms`}
                  min="50"
                  step="50"
                  className="flex-1"
                />
                <Button
                  onClick={handleDelayChange}
                  disabled={!delayInput || isNaN(parseInt(delayInput))}
                  className="whitespace-nowrap"
                >
                  Set Delay
                </Button>
              </div>
            </Label>
            <p className="text-sm text-gray-400 mt-2">
              Controls the time between each output change in automatic cycle
            </p>
          </ControlCard>
        )}

        {/* LED Status Display */}
        <ControlCard title="Output Status">
          <div className="grid grid-cols-4 gap-4">
            {safeState.led_on.map((led, index) => (
              <div
                key={index}
                className={`p-4 rounded text-center transition-colors ${
                  led ? "bg-green-600" : "bg-gray-700"
                }`}
              >
                <div className="text-sm font-semibold">OUT {index + 1}</div>
                <div className="text-xs mt-1">{led ? "ON" : "OFF"}</div>
              </div>
            ))}
          </div>
        </ControlCard>

        {/* Manual LED Controls - Only visible in Manual mode */}
        {isManualMode && (
          <>
            <ControlCard title="Manual LED Controls">
              {!isRunning && (
                <p className="text-sm text-yellow-500 mb-4">
                  Press Start to enable manual controls
                </p>
              )}
              <div className="grid grid-cols-2 gap-6">
                {safeState.led_on.map((led, index) => (
                  <Label key={index} label={`LED ${index + 1}`}>
                    <SelectionGroup<"On" | "Off">
                      value={led ? "On" : "Off"}
                      orientation="vertical"
                      className="grid h-full grid-cols-2 gap-2"
                      options={{
                        Off: {
                          children: "Off",
                          icon: "lu:CirclePause",
                          isActiveClassName: "bg-red-600",
                          className: "h-full",
                          disabled: !canControlOutputs,
                        },
                        On: {
                          children: "On",
                          icon: "lu:CirclePlay",
                          isActiveClassName: "bg-green-600",
                          className: "h-full",
                          disabled: !canControlOutputs,
                        },
                      }}
                      onChange={(value) => setLed(index, value === "On")}
                    />
                  </Label>
                ))}
              </div>
            </ControlCard>

            <ControlCard title="Master LED Control">
              {!isRunning && (
                <p className="text-sm text-yellow-500 mb-4">
                  Press Start to enable manual controls
                </p>
              )}
              <SelectionGroup<"On" | "Off">
                value={safeState.led_on.every(Boolean) ? "On" : "Off"}
                orientation="horizontal"
                options={{
                  Off: { 
                    children: "Turn All Off",
                    disabled: !canControlOutputs,
                  },
                  On: { 
                    children: "Turn All On",
                    disabled: !canControlOutputs,
                  },
                }}
                onChange={(value) => setAllLeds(value === "On")}
              />
            </ControlCard>
          </>
        )}
      </ControlGrid>
    </Page>
  );
}
