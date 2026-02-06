import React from "react";
import { StateChartEditor } from "./StateChartEditor";

export const StateChartEditorPage: React.FC = () => {
  return (
    <div className="h-screen w-full flex flex-col">
      <div className="border-b bg-background px-6 py-3">
        <h1 className="text-2xl font-bold">UML StateChart Editor</h1>
        <p className="text-sm text-muted-foreground">
          Design and export state machines for your control system
        </p>
      </div>
      <div className="flex-1">
        <StateChartEditor />
      </div>
    </div>
  );
};
