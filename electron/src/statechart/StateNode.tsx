import React, { memo } from "react";
import { Handle, Position, NodeProps } from "@xyflow/react";
import { StateNodeData } from "./types";
import { cn } from "@/lib/utils";

export const StateNode = memo(({ data, selected }: NodeProps<StateNodeData>) => {
  const isInitial = data.type === "initial";
  const isFinal = data.type === "final";
  const isCompound = data.type === "compound";
  const isActive = data.isActive || false;

  return (
    <div
      className={cn(
        "rounded-lg border-2 bg-background px-4 py-3 shadow-md transition-all",
        selected && "ring-2 ring-primary ring-offset-2",
        isActive && "ring-4 ring-green-400 ring-offset-2 bg-green-50 dark:bg-green-950",
        isInitial && "border-green-500",
        isFinal && "border-red-500",
        isCompound && "border-blue-500",
        !isInitial && !isFinal && !isCompound && "border-gray-300"
      )}
    >
      <Handle
        type="target"
        position={Position.Top}
        className="!bg-gray-400"
      />

      <div className="flex flex-col gap-1">
        <div className="flex items-center gap-2">
          {isInitial && (
            <div className="h-2 w-2 rounded-full bg-green-500" />
          )}
          {isFinal && (
            <div className="h-3 w-3 rounded-full border-2 border-red-500">
              <div className="h-full w-full rounded-full bg-red-500" />
            </div>
          )}
          <div className="font-semibold text-sm">{data.label}</div>
        </div>

        {data.entry && data.entry.length > 0 && (
          <div className="text-xs text-muted-foreground">
            entry: {data.entry.join(", ")}
          </div>
        )}

        {data.exit && data.exit.length > 0 && (
          <div className="text-xs text-muted-foreground">
            exit: {data.exit.join(", ")}
          </div>
        )}
      </div>

      <Handle
        type="source"
        position={Position.Bottom}
        className="!bg-gray-400"
      />
    </div>
  );
});

StateNode.displayName = "StateNode";
