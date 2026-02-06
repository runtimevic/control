import React from "react";
import { StateChartNode, StateChartEdge, StateType } from "./types";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Plus, Trash2 } from "lucide-react";

interface PropertiesPanelProps {
  selectedNode?: StateChartNode | null;
  selectedEdge?: StateChartEdge | null;
  onUpdateNode: (id: string, data: Partial<StateChartNode["data"]>) => void;
  onUpdateEdge: (id: string, data: Partial<StateChartEdge["data"]>) => void;
}

export const PropertiesPanel: React.FC<PropertiesPanelProps> = ({
  selectedNode,
  selectedEdge,
  onUpdateNode,
  onUpdateEdge,
}) => {
  if (!selectedNode && !selectedEdge) {
    return (
      <div className="flex h-full items-center justify-center p-4 text-muted-foreground">
        Select a node or edge to edit properties
      </div>
    );
  }

  if (selectedNode) {
    return <NodeProperties node={selectedNode} onUpdate={onUpdateNode} />;
  }

  if (selectedEdge) {
    return <EdgeProperties edge={selectedEdge} onUpdate={onUpdateEdge} />;
  }

  return null;
};

const NodeProperties: React.FC<{
  node: StateChartNode;
  onUpdate: (id: string, data: Partial<StateChartNode["data"]>) => void;
}> = ({ node, onUpdate }) => {
  const handleLabelChange = (label: string) => {
    onUpdate(node.id, { label });
  };

  const handleTypeChange = (type: StateType) => {
    onUpdate(node.id, { type });
  };

  const handleDescriptionChange = (description: string) => {
    onUpdate(node.id, { description });
  };

  const handleArrayChange = (
    field: "entry" | "exit",
    index: number,
    value: string
  ) => {
    const current = node.data[field] || [];
    const updated = [...current];
    updated[index] = value;
    onUpdate(node.id, { [field]: updated });
  };

  const handleAddToArray = (field: "entry" | "exit") => {
    const current = node.data[field] || [];
    onUpdate(node.id, { [field]: [...current, ""] });
  };

  const handleRemoveFromArray = (field: "entry" | "exit", index: number) => {
    const current = node.data[field] || [];
    onUpdate(node.id, { [field]: current.filter((_, i) => i !== index) });
  };

  return (
    <div className="space-y-4 p-4">
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">State Properties</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="label">Label</Label>
            <Input
              id="label"
              value={node.data.label}
              onChange={(e) => handleLabelChange(e.target.value)}
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="type">Type</Label>
            <Select value={node.data.type} onValueChange={handleTypeChange}>
              <SelectTrigger id="type">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="normal">Normal</SelectItem>
                <SelectItem value="initial">Initial</SelectItem>
                <SelectItem value="final">Final</SelectItem>
                <SelectItem value="compound">Compound</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <Label htmlFor="description">Description</Label>
            <Textarea
              id="description"
              value={node.data.description || ""}
              onChange={(e) => handleDescriptionChange(e.target.value)}
              placeholder="State description..."
            />
          </div>

          <ActionArrayEditor
            label="Entry Actions"
            items={node.data.entry || []}
            onChange={(index, value) => handleArrayChange("entry", index, value)}
            onAdd={() => handleAddToArray("entry")}
            onRemove={(index) => handleRemoveFromArray("entry", index)}
          />

          <ActionArrayEditor
            label="Exit Actions"
            items={node.data.exit || []}
            onChange={(index, value) => handleArrayChange("exit", index, value)}
            onAdd={() => handleAddToArray("exit")}
            onRemove={(index) => handleRemoveFromArray("exit", index)}
          />
        </CardContent>
      </Card>
    </div>
  );
};

const EdgeProperties: React.FC<{
  edge: StateChartEdge;
  onUpdate: (id: string, data: Partial<StateChartEdge["data"]>) => void;
}> = ({ edge, onUpdate }) => {
  const data = edge.data || {};

  const handleEventChange = (event: string) => {
    onUpdate(edge.id, { ...data, event });
  };

  const handleGuardChange = (guard: string) => {
    onUpdate(edge.id, { ...data, guard });
  };

  const handleDescriptionChange = (description: string) => {
    onUpdate(edge.id, { ...data, description });
  };

  const handleActionChange = (index: number, value: string) => {
    const actions = [...(data.actions || [])];
    actions[index] = value;
    onUpdate(edge.id, { ...data, actions });
  };

  const handleAddAction = () => {
    onUpdate(edge.id, { ...data, actions: [...(data.actions || []), ""] });
  };

  const handleRemoveAction = (index: number) => {
    const actions = (data.actions || []).filter((_, i) => i !== index);
    onUpdate(edge.id, { ...data, actions });
  };

  return (
    <div className="space-y-4 p-4">
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Transition Properties</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="event">Event</Label>
            <Input
              id="event"
              value={data.event || ""}
              onChange={(e) => handleEventChange(e.target.value)}
              placeholder="EVENT_NAME"
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="guard">Guard</Label>
            <Input
              id="guard"
              value={data.guard || ""}
              onChange={(e) => handleGuardChange(e.target.value)}
              placeholder="condition"
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="description">Description</Label>
            <Textarea
              id="description"
              value={data.description || ""}
              onChange={(e) => handleDescriptionChange(e.target.value)}
              placeholder="Transition description..."
            />
          </div>

          <ActionArrayEditor
            label="Actions"
            items={data.actions || []}
            onChange={handleActionChange}
            onAdd={handleAddAction}
            onRemove={handleRemoveAction}
          />
        </CardContent>
      </Card>
    </div>
  );
};

const ActionArrayEditor: React.FC<{
  label: string;
  items: string[];
  onChange: (index: number, value: string) => void;
  onAdd: () => void;
  onRemove: (index: number) => void;
}> = ({ label, items, onChange, onAdd, onRemove }) => {
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <Label>{label}</Label>
        <Button size="sm" variant="outline" onClick={onAdd}>
          <Plus className="h-4 w-4" />
        </Button>
      </div>
      <div className="space-y-2">
        {items.map((item, index) => (
          <div key={index} className="flex gap-2">
            <Input
              value={item}
              onChange={(e) => onChange(index, e.target.value)}
              placeholder="action name"
            />
            <Button
              size="icon"
              variant="ghost"
              onClick={() => onRemove(index)}
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>
        ))}
      </div>
    </div>
  );
};
