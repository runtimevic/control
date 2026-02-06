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
} from "lucide-react";
import { Separator } from "@/components/ui/separator";
import { Badge } from "@/components/ui/badge";

const nodeTypes = {
  stateNode: StateNode,
};

export const StateChartEditor: React.FC = () => {
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
    exportToXState,
    importFromXState,
    setNodes,
  } = useStateChart();

  const {
    isConnected,
    loadMachine,
    sendEvent,
    currentState,
    previousState,
    availableEvents,
    loadError,
  } = useStateMachineSocket();

  const [selectedNode, setSelectedNode] = useState<StateChartNode | null>(null);
  const [selectedEdge, setSelectedEdge] = useState<StateChartEdge | null>(null);
  const [isRunning, setIsRunning] = useState(false);

  // Highlight active node when execution state changes
  useEffect(() => {
    if (!currentState) {
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
  }, [currentState, setNodes]);

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
    loadMachine(xstateConfig);
    setIsRunning(true);
    console.log("[StateChart] Running state machine in backend");
  }, [exportToXState, loadMachine]);

  const handleStop = useCallback(() => {
    setIsRunning(false);
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
  
  const handleImport = useCallback(() => {
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
  }, [importFromXState]);

  const handleSave = useCallback(() => {
    const xstateConfig = exportToXState();
    console.log("StateChart saved:", xstateConfig);
    // TODO: Save to backend or local storage
    alert("StateChart saved! Check console for JSON output.");
  }, [exportToXState]);

  const handleRun = useCallback(() => {
    const xstateConfig = exportToXState();
    console.log("Running StateChart:", xstateConfig);
    // TODO: Send to backend Rust server
    alert("Would send to backend for execution. Check console for JSON.");
  }, [exportToXState]);

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
                <Button size="sm" onClick={handleStop} variant="destructive">
                  <StopCircle className="h-4 w-4 mr-1" />
                  Stop
                </Button>
              )}
              {!isConnected && (
                <Badge variant="destructive">Disconnected</Badge>
              )}
            </div>
          </Panel>

          {/* Execution State Panel */}
          {isRunning && currentState && (
            <Panel position="top-right" className="bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 rounded-lg shadow-lg border p-3">
              <div className="space-y-2">
                <div className="text-sm font-semibold">Execution State</div>
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
          )}ton size="sm" onClick={handleSave} variant="outline">
                <Save className="h-4 w-4 mr-1" />
                Save
              </Button>
              <Button size="sm" onClick={handleExport} variant="outline">
                <Download className="h-4 w-4 mr-1" />
                Export JSON
              </Button>
              <Button size="sm" onClick={handleImport} variant="outline">
                <Upload className="h-4 w-4 mr-1" />
                Import JSON
              </Button>
              <Separator orientation="vertical" className="h-8" />
              <Button size="sm" onClick={handleRun} variant="secondary">
                <PlayCircle className="h-4 w-4 mr-1" />
                Run in Backend
              </Button>
            </div>
          </Panel>
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
