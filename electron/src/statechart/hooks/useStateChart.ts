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

    // Create nodes
    Object.entries(config.states).forEach(([stateName, stateConfig], index) => {
      const nodeId = `${nodeIdCounter++}`;
      labelToId[stateName] = nodeId;

      const isInitial = config.initial === stateName;
      const isFinal = stateConfig.type === "final";
      const isCompound = stateConfig.type === "compound";

      newNodes.push({
        id: nodeId,
        type: "stateNode",
        position: {
          x: 100 + (index % 4) * 200,
          y: 100 + Math.floor(index / 4) * 150,
        },
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
            newEdges.push({
              id: `e${sourceId}-${targetId}`,
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
    exportToXState,
    importFromXState,
    setNodes,
  };
};
