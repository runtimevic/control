import { Node, Edge } from "@xyflow/react";

export type StateType = "normal" | "initial" | "final" | "compound";

export interface StateNodeData {
  label: string;
  type: StateType;
  entry?: string[];
  exit?: string[];
  on?: Record<string, TransitionConfig>;
  description?: string;
  isActive?: boolean; // For real-time highlighting during execution
}

export interface TransitionConfig {
  target: string;
  guard?: string;
  actions?: string[];
  description?: string;
}

export interface StateChartNode extends Node {
  data: StateNodeData;
}

export interface StateChartEdge extends Edge {
  data?: {
    event?: string;
    guard?: string;
    actions?: string[];
    description?: string;
  };
}

export interface XStateConfig {
  id: string;
  initial?: string;
  states: Record<string, XStateStateConfig>;
  actionMappings?: Record<string, {
    action: string;
    value: any;
  }>;
}

export interface XStateStateConfig {
  entry?: string[];
  exit?: string[];
  on?: Record<string, XStateTransition | string>;
  type?: "final" | "compound";
}

export interface XStateTransition {
  target: string;
  guard?: string;
  actions?: string[];
}

export interface ExecutionState {
  currentState: string;
  previousState?: string;
  availableEvents: string[];
  timestamp: number;
}
