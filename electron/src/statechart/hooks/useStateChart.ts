import { useCallback, useState } from "react";
import {
  addEdge,
  applyEdgeChanges,
  applyNodeChanges,
  Connection,
  EdgeChange,
  NodeChange,
} from "@xyflow/react";
import {
  StateChartNode,
  StateChartEdge,
  XStateConfig,
  StateNodeData,
} from "../types";

export const useStateChart = () => {
  const [nodes, setNodes] = useState<StateChartNode[]>([
    {
      id: "1",
      type: "stateNode",
      position: { x: 250, y: 100 },
      data: {
        label: "Idle",
        type: "initial",
        entry: [],
        exit: [],
      },
    },
  ]);

  const [edges, setEdges] = useState<StateChartEdge[]>([]);

  const onNodesChange = useCallback(
    (changes: NodeChange[]) => setNodes((nds) => applyNodeChanges(changes, nds)),
    []
  );

  const onEdgesChange = useCallback(
    (changes: EdgeChange[]) => setEdges((eds) => applyEdgeChanges(changes, eds)),
    []
  );

  const onConnect = useCallback(
    (connection: Connection) =>
      setEdges((eds) =>
        addEdge(
          {
            ...connection,
            type: "smoothstep",
            data: {
              event: "NEW_EVENT",
              guard: "",
              actions: [],
            },
          },
          eds
        )
      ),
    []
  );

  const addNewState = useCallback(() => {
    const newId = `${nodes.length + 1}`;
    const newNode: StateChartNode = {
      id: newId,
      type: "stateNode",
      position: {
        x: Math.random() * 400 + 100,
        y: Math.random() * 400 + 100,
      },
      data: {
        label: `State ${newId}`,
        type: "normal",
        entry: [],
        exit: [],
      },
    };
    setNodes((nds) => [...nds, newNode]);
  }, [nodes]);

  const updateNodeData = useCallback(
    (nodeId: string, data: Partial<StateNodeData>) => {
      setNodes((nds) =>
        nds.map((node) =>
          node.id === nodeId ? { ...node, data: { ...node.data, ...data } } : node
        )
      );
    },
    []
  );

  const updateEdgeData = useCallback(
    (edgeId: string, data: Partial<StateChartEdge["data"]>) => {
      setEdges((eds) =>
        eds.map((edge) =>
          edge.id === edgeId ? { ...edge, data: { ...edge.data, ...data } } : edge
        )
      );
    },
    []
  );

  const deleteSelected = useCallback(() => {
    setNodes((nds) => nds.filter((node) => !node.selected));
    setEdges((eds) => eds.filter((edge) => !edge.selected));
  }, []);

  const autoLayout = useCallback(() => {
    setNodes((currentNodes) => {
      const nodeCount = currentNodes.length;
      const useCircularLayout = nodeCount <= 6;
      
      return currentNodes.map((node, index) => {
        let position;
        
        if (useCircularLayout) {
          // Circular layout
          const radius = 250;
          const centerX = 400;
          const centerY = 300;
          const angle = (index / nodeCount) * 2 * Math.PI - Math.PI / 2;
          position = {
            x: centerX + radius * Math.cos(angle),
            y: centerY + radius * Math.sin(angle),
          };
        } else {
          // Grid layout with more spacing
          const cols = Math.ceil(Math.sqrt(nodeCount));
          position = {
            x: 150 + (index % cols) * 350,
            y: 150 + Math.floor(index / cols) * 250,
          };
        }
        
        return {
          ...node,
          position,
        };
      });
    });
  }, []);

  const exportToXState = useCallback((): XStateConfig => {
    const initialNode = nodes.find((n) => n.data.type === "initial");

    const states: XStateConfig["states"] = {};

    nodes.forEach((node) => {
      const stateConfig: XStateConfig["states"][string] = {};

      if (node.data.entry && node.data.entry.length > 0) {
        stateConfig.entry = node.data.entry;
      }

      if (node.data.exit && node.data.exit.length > 0) {
        stateConfig.exit = node.data.exit;
      }

      if (node.data.type === "final") {
        stateConfig.type = "final";
      }

      if (node.data.type === "compound") {
        stateConfig.type = "compound";
      }

      // Add transitions from edges
      const outgoingEdges = edges.filter((e) => e.source === node.id);
      if (outgoingEdges.length > 0) {
        stateConfig.on = {};
        outgoingEdges.forEach((edge) => {
          const event = edge.data?.event || "EVENT";
          const targetNode = nodes.find((n) => n.id === edge.target);
          if (targetNode) {
            const transition: any = { target: targetNode.data.label };

            if (edge.data?.guard) {
              transition.guard = edge.data.guard;
            }

            if (edge.data?.actions && edge.data.actions.length > 0) {
              transition.actions = edge.data.actions;
            }

            stateConfig.on![event] = transition;
          }
        });
      }

      states[node.data.label] = stateConfig;
    });

    return {
      id: "machine",
      initial: initialNode?.data.label || nodes[0]?.data.label,
      states,
    };
  }, [nodes, edges]);

  const importFromXState = useCallback((config: XStateConfig) => {
    const newNodes: StateChartNode[] = [];
    const newEdges: StateChartEdge[] = [];
    let nodeIdCounter = 1;
    const labelToId: Record<string, string> = {};

    const stateCount = Object.keys(config.states).length;
    
    // Choose layout based on number of states
    const useCircularLayout = stateCount <= 6;
    
    // Create nodes
    Object.entries(config.states).forEach(([stateName, stateConfig], index) => {
      const nodeId = `${nodeIdCounter++}`;
      labelToId[stateName] = nodeId;

      const isInitial = config.initial === stateName;
      const isFinal = stateConfig.type === "final";
      const isCompound = stateConfig.type === "compound";

      let position;
      
      if (useCircularLayout) {
        // Circular layout for small state machines
        const radius = 200;
        const centerX = 400;
        const centerY = 300;
        const angle = (index / stateCount) * 2 * Math.PI - Math.PI / 2;
        position = {
          x: centerX + radius * Math.cos(angle),
          y: centerY + radius * Math.sin(angle),
        };
      } else {
        // Grid layout for larger state machines with more spacing
        const cols = Math.ceil(Math.sqrt(stateCount));
        position = {
          x: 150 + (index % cols) * 300,
          y: 150 + Math.floor(index / cols) * 200,
        };
      }

      newNodes.push({
        id: nodeId,
        type: "stateNode",
        position,
        data: {
          label: stateName,
          type: isInitial ? "initial" : isFinal ? "final" : isCompound ? "compound" : "normal",
          entry: stateConfig.entry || [],
          exit: stateConfig.exit || [],
        },
      });
    });

    // Create edges
    Object.entries(config.states).forEach(([stateName, stateConfig]) => {
      const sourceId = labelToId[stateName];
      if (stateConfig.on) {
        Object.entries(stateConfig.on).forEach(([event, transition]) => {
          const targetLabel = typeof transition === "string" ? transition : transition.target;
          const targetId = labelToId[targetLabel];

          if (targetId) {
            // Use event name in edge ID to ensure uniqueness
            const edgeId = `e${sourceId}-${targetId}-${event}`;
            newEdges.push({
              id: edgeId,
              source: sourceId,
              target: targetId,
              type: "smoothstep",
              data: {
                event,
                guard: typeof transition === "object" ? transition.guard : undefined,
                actions: typeof transition === "object" ? transition.actions : undefined,
              },
            });
          }
        });
      }
    });

    setNodes(newNodes);
    setEdges(newEdges);
  }, []);

  return {
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
  };
};
