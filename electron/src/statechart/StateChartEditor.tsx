import React, { useCallback, useState, useEffect } from "react";
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  Panel,
  useReactFlow,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { StateNode } from "./StateNode";
import { PropertiesPanel } from "./PropertiesPanel";
import { useStateChart } from "./hooks/useStateChart";
import { useStateMachineSocket } from "./hooks/useStateMachineSocket";
import { StateChartNode, StateChartEdge } from "./types";
import { Button } from "@/components/ui/button";
import {
  Download,
  Upload,
  Plus,
  Trash2,
  Save,
  PlayCircle,
  StopCircle,
  LayoutGrid,
} from "lucide-react";
import { Separator } from "@/components/ui/separator";
import { Badge } from "@/components/ui/badge";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useMainNamespace } from "@/client/mainNamespace";
import type { MachineIdentificationUnique } from "@/machines/types";

const nodeTypes = {
  stateNode: StateNode,
};

export const StateChartEditor: React.FC = () => {
  // All hooks must be called in the same order every render
  // 1. Custom hooks first
  const {
    nodes,
    edges,
    onNodesChange,
    onEdgesChange,
    onConnect,
    addNewState,
    updateNodeData,
    updateEdgeData,
    deleteSelected,
    autoLayout,
    exportToXState,
    importFromXState,
    setNodes,
  } = useStateChart();

  // Get list of machines from main namespace
  const mainStore = useMainNamespace();
  const machines = mainStore.machines?.data?.machines || [];

  // 2. useState hooks
  const [selectedMachine, setSelectedMachine] = useState<MachineIdentificationUnique | null>(null);
  const [selectedNode, setSelectedNode] = useState<StateChartNode | null>(null);
  const [selectedEdge, setSelectedEdge] = useState<StateChartEdge | null>(null);
  const [isRunning, setIsRunning] = useState(false);
  const [autoSimulate, setAutoSimulate] = useState(false);

  // Connect to machine-specific or global statechart namespace
  const {
    isConnected,
    loadMachine,
    sendEvent,
    currentState,
    previousState,
    availableEvents,
    loadError,
  } = useStateMachineSocket(selectedMachine);

  // 3. useEffect hooks
  // Auto-simulate: Send first available event every 2 seconds
  useEffect(() => {
    if (!autoSimulate) {
      console.log("[Auto-Simulate] Disabled");
      return;
    }
    
    if (!isRunning) {
      console.log("[Auto-Simulate] Not running - click 'Run in Backend' first");
      return;
    }
    
    if (availableEvents.length === 0) {
      console.log("[Auto-Simulate] No available events");
      return;
    }

    console.log(`[Auto-Simulate] Starting with events:`, availableEvents);
    const intervalId = setInterval(() => {
      if (availableEvents.length > 0) {
        const nextEvent = availableEvents[0];
        console.log(`[Auto-Simulate] Sending event: ${nextEvent}`);
        sendEvent(nextEvent);
      }
    }, 2000);

    return () => clearInterval(intervalId);
  }, [autoSimulate, isRunning, availableEvents, sendEvent]);

  // Highlight active node when execution state changes
  useEffect(() => {
    if (!currentState || !isRunning) {
      // Clear highlighting when not running
      setNodes((nds) =>
        nds.map((node) => ({
          ...node,
          data: { ...node.data, isActive: false },
        }))
      );
      return;
    }

    setNodes((nds) =>
      nds.map((node) => ({
        ...node,
        data: {
          ...node.data,
          isActive: node.data.label === currentState,
        },
      }))
    );
  }, [currentState, isRunning, setNodes]);

  // Monitor available events for debugging
  useEffect(() => {
    console.log("[StateChart] State updated:", {
      isRunning,
      autoSimulate,
      currentState,
      availableEvents,
      isConnected,
    });
  }, [isRunning, autoSimulate, currentState, availableEvents, isConnected]);

  // 4. useCallback hooks
  const handleNodeClick = useCallback(
    (_event: React.MouseEvent, node: StateChartNode) => {
      setSelectedNode(node);
      setSelectedEdge(null);
    },
    []
  );

  const handleEdgeClick = useCallback(
    (_event: React.MouseEvent, edge: StateChartEdge) => {
      setSelectedEdge(edge);
      setSelectedNode(null);
    },
    []
  );

  const handlePaneClick = useCallback(() => {
    setSelectedNode(null);
    setSelectedEdge(null);
  }, []);

  const handleExport = useCallback(() => {
    const xstateConfig = exportToXState();
    const json = JSON.stringify(xstateConfig, null, 2);
    const blob = new Blob([json], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = "statechart.json";
    link.click();
    URL.revokeObjectURL(url);
    console.log("[StateChart] Exported state machine");
  }, [exportToXState]);

  const handleStop = useCallback(() => {
    setIsRunning(false);
    setAutoSimulate(false);
    // Clear highlighting
    setNodes((nds) =>
      nds.map((node) => ({
        ...node,
        data: { ...node.data, isActive: false },
      }))
    );
  }, [setNodes]);

  const handleSendEvent = useCallback(
    (event: string) => {
      sendEvent(event);
    },
    [sendEvent]
  );

  const handleImport = useCallback(() => {
    // Stop any running simulation first
    if (isRunning || autoSimulate) {
      console.log("[StateChart] Stopping simulation before import");
      setIsRunning(false);
      setAutoSimulate(false);
    }
    
    const input = document.createElement("input");
    input.type = "file";
    input.accept = "application/json";
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (file) {
        const reader = new FileReader();
        reader.onload = (event) => {
          try {
            const config = JSON.parse(event.target?.result as string);
            console.log("[StateChart] Importing new state machine:", config.id);
            importFromXState(config);
          } catch (error) {
            console.error("Error importing JSON:", error);
            alert("Error importing JSON file");
          }
        };
        reader.readAsText(file);
      }
    };
    input.click();
  }, [importFromXState, isRunning, autoSimulate]);

  const handleSave = useCallback(() => {
    const xstateConfig = exportToXState();
    console.log("StateChart saved:", xstateConfig);
    // TODO: Save to backend or local storage
    alert("StateChart saved! Check console for JSON output.");
  }, [exportToXState]);

  const handleRun = useCallback(() => {
    const xstateConfig = exportToXState();
    console.log("[StateChart] Loading state machine into backend:", xstateConfig);
    console.log("[StateChart] Selected machine:", selectedMachine);
    console.log("[StateChart] Auto-simulate:", autoSimulate);
    loadMachine(xstateConfig);
    setIsRunning(true);
    console.log("[StateChart] isRunning set to true");
  }, [exportToXState, loadMachine, selectedMachine, autoSimulate]);

  return (
    <div className="flex h-full">
      <div className="flex-1 relative">
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onNodeClick={handleNodeClick}
          onEdgeClick={handleEdgeClick}
          onPaneClick={handlePaneClick}
          nodeTypes={nodeTypes}
          fitView
          minZoom={0.1}
          maxZoom={4}
        >
          <Background />
          <Controls />
          <MiniMap />
          <Panel position="top-left" className="bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 rounded-lg shadow-lg border p-2">
            <div className="flex flex-wrap gap-2">
              <Button size="sm" onClick={addNewState} variant="default">
                <Plus className="h-4 w-4 mr-1" />
                Add State
              </Button>
              <Separator orientation="vertical" className="h-8" />
              <Button 
                size="sm" 
                onClick={handleImport} 
                variant="outline"
              >
                <Upload className="h-4 w-4 mr-1" />
                Import
              </Button>
              <Button size="sm" onClick={handleExport} variant="outline">
                <Download className="h-4 w-4 mr-1" />
                Export
              </Button>
              <Separator orientation="vertical" className="h-8" />
              <Button size="sm" onClick={autoLayout} variant="outline">
                <LayoutGrid className="h-4 w-4 mr-1" />
                Auto Layout
              </Button>
              <Separator orientation="vertical" className="h-8" />
              <Select 
                value={selectedMachine ? `${selectedMachine.machine_identification.vendor}/${selectedMachine.machine_identification.machine}/${selectedMachine.serial}` : "global"} 
                onValueChange={(value) => {
                  if (value === "global") {
                    setSelectedMachine(null);
                  } else {
                    const [vendor, machine, serial] = value.split('/').map(Number);
                    setSelectedMachine({ 
                      machine_identification: { vendor, machine }, 
                      serial 
                    });
                  }
                }}
              >
                <SelectTrigger className="w-[200px] h-8">
                  <SelectValue placeholder="Select machine..." />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="global">Global (Testing)</SelectItem>
                  {machines.map((m) => {
                    const mid = m.machine_identification_unique;
                    const vendor = mid.machine_identification.vendor;
                    const machine = mid.machine_identification.machine;
                    const serial = mid.serial;
                    const key = `${vendor}/${machine}/${serial}`;
                    return (
                      <SelectItem key={key} value={key}>
                        Machine {vendor}/{machine}/{serial}
                      </SelectItem>
                    );
                  })}
                </SelectContent>
              </Select>
              <Separator orientation="vertical" className="h-8" />
              {!isRunning ? (
                <Button 
                  size="sm" 
                  onClick={handleRun} 
                  variant="secondary"
                  disabled={!isConnected}
                >
                  <PlayCircle className="h-4 w-4 mr-1" />
                  Run in Backend
                </Button>
              ) : (
                <>
                  <Button size="sm" onClick={handleStop} variant="destructive">
                    <StopCircle className="h-4 w-4 mr-1" />
                    Stop
                  </Button>
                  {autoSimulate && (
                    <Badge variant="default" className="animate-pulse">
                      Auto-simulating...
                    </Badge>
                  )}
                </>
              )}
              {!isConnected && (
                <Badge variant="destructive">Disconnected</Badge>
              )}
            </div>
          </Panel>

          {/* Execution State Panel */}
          {isRunning && (
            <Panel position="top-right" className="bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 rounded-lg shadow-lg border p-3">
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <div className="text-sm font-semibold">Execution State</div>
                  <Button 
                    size="sm" 
                    onClick={handleStop} 
                    variant="destructive"
                    className="h-6 text-xs"
                  >
                    <StopCircle className="h-3 w-3 mr-1" />
                    Stop
                  </Button>
                </div>
                {currentState ? (
                  <div className="text-xs space-y-1">
                    <div>
                      <span className="text-muted-foreground">Current:</span>{" "}
                      <Badge variant="default">{currentState}</Badge>
                    </div>
                    {previousState && (
                      <div>
                        <span className="text-muted-foreground">Previous:</span>{" "}
                        <span className="font-mono">{previousState}</span>
                      </div>
                    )}
                    <div>
                      <span className="text-muted-foreground">Available Events:</span>
                    </div>
                    <div className="flex flex-wrap gap-1">
                      {availableEvents.map((event) => (
                        <Button
                          key={event}
                          size="sm"
                          variant="outline"
                          onClick={() => handleSendEvent(event)}
                          className="h-7 px-2 text-xs"
                        >
                          {event}
                        </Button>
                      ))}
                    </div>
                  </div>
                ) : (
                  <div className="text-xs text-muted-foreground">
                    Loading state machine...
                  </div>
                )}
                <Separator className="my-2" />
                <div className="flex items-center space-x-2">
                  <Checkbox
                    id="auto-simulate"
                    checked={autoSimulate}
                    onCheckedChange={(checked) => setAutoSimulate(checked === true)}
                    disabled={!currentState || availableEvents.length === 0}
                  />
                  <Label
                    htmlFor="auto-simulate"
                    className="text-xs font-normal cursor-pointer"
                  >
                    Auto-simulate (2s interval)
                  </Label>
                </div>
                {autoSimulate && (
                  <div className="text-xs text-muted-foreground pt-1 border-t">
                    💡 Click Stop button or uncheck above to pause
                  </div>
                )}
              </div>
            </Panel>
          )}

          {/* Error Panel */}
          {loadError && (
            <Panel position="bottom-center" className="bg-destructive/95 backdrop-blur supports-[backdrop-filter]:bg-destructive/90 rounded-lg shadow-lg border border-destructive p-3">
              <div className="text-sm text-destructive-foreground">
                <strong>Error:</strong> {loadError}
              </div>
            </Panel>
          )}
        </ReactFlow>
      </div>

      <div className="w-80 border-l bg-muted/40 overflow-y-auto">
        <PropertiesPanel
          selectedNode={selectedNode}
          selectedEdge={selectedEdge}
          onUpdateNode={updateNodeData}
          onUpdateEdge={updateEdgeData}
        />
      </div>
    </div>
  );
};
