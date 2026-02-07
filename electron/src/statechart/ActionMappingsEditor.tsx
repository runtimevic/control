import React, { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Plus, Trash2 } from "lucide-react";
import { Separator } from "@/components/ui/separator";

interface ActionMapping {
  actionName: string;
  mutationType: string;
  parameters: Record<string, any>;
}

interface ActionMappingsEditorProps {
  mappings: Record<string, { action: string; value: Record<string, any> }>;
  onChange: (
    mappings: Record<string, { action: string; value: Record<string, any> }>
  ) => void;
}

export const ActionMappingsEditor: React.FC<ActionMappingsEditorProps> = ({
  mappings,
  onChange,
}) => {
  const [editingMappings, setEditingMappings] = useState<ActionMapping[]>(
    Object.entries(mappings).map(([actionName, { action, value }]) => ({
      actionName,
      mutationType: action,
      parameters: value,
    }))
  );

  const handleAddMapping = () => {
    setEditingMappings([
      ...editingMappings,
      {
        actionName: "",
        mutationType: "SetLed",
        parameters: { index: 0, on: true },
      },
    ]);
  };

  const handleRemoveMapping = (index: number) => {
    const newMappings = editingMappings.filter((_, i) => i !== index);
    setEditingMappings(newMappings);
    updateParentMappings(newMappings);
  };

  const handleMutationTypeChange = (index: number, mutationType: string) => {
    const newMappings = [...editingMappings];
    newMappings[index].mutationType = mutationType;

    // Set default parameters based on mutation type
    switch (mutationType) {
      case "SetLed":
        newMappings[index].parameters = { index: 0, on: true };
        break;
      case "SetAllLeds":
        newMappings[index].parameters = { on: true };
        break;
      case "SetMode":
        newMappings[index].parameters = { mode: "Manual" };
        break;
      case "Start":
      case "Stop":
        newMappings[index].parameters = {};
        break;
      default:
        newMappings[index].parameters = {};
    }

    setEditingMappings(newMappings);
    updateParentMappings(newMappings);
  };

  const handleActionNameChange = (index: number, actionName: string) => {
    const newMappings = [...editingMappings];
    newMappings[index].actionName = actionName;
    setEditingMappings(newMappings);
    updateParentMappings(newMappings);
  };

  const handleParameterChange = (
    index: number,
    paramName: string,
    value: any
  ) => {
    const newMappings = [...editingMappings];
    newMappings[index].parameters[paramName] = value;
    setEditingMappings(newMappings);
    updateParentMappings(newMappings);
  };

  const updateParentMappings = (newMappings: ActionMapping[]) => {
    const mappingsObj: Record<
      string,
      { action: string; value: Record<string, any> }
    > = {};
    newMappings.forEach((mapping) => {
      if (mapping.actionName) {
        mappingsObj[mapping.actionName] = {
          action: mapping.mutationType,
          value: mapping.parameters,
        };
      }
    });
    onChange(mappingsObj);
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Action Mappings</CardTitle>
        <CardDescription>
          Connect state chart actions to hardware commands
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {editingMappings.map((mapping, index) => (
          <div key={index} className="space-y-2 p-4 border rounded-lg">
            <div className="flex justify-between items-start">
              <div className="flex-1 space-y-3">
                <div>
                  <Label htmlFor={`action-${index}`}>Action Name</Label>
                  <Input
                    id={`action-${index}`}
                    placeholder="e.g., activateRedLight"
                    value={mapping.actionName}
                    onChange={(e) =>
                      handleActionNameChange(index, e.target.value)
                    }
                  />
                </div>

                <div>
                  <Label htmlFor={`mutation-${index}`}>Hardware Command</Label>
                  <Select
                    value={mapping.mutationType}
                    onValueChange={(value) =>
                      handleMutationTypeChange(index, value)
                    }
                  >
                    <SelectTrigger id={`mutation-${index}`}>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="SetLed">SetLed</SelectItem>
                      <SelectItem value="SetAllLeds">SetAllLeds</SelectItem>
                      <SelectItem value="SetMode">SetMode</SelectItem>
                      <SelectItem value="Start">Start</SelectItem>
                      <SelectItem value="Stop">Stop</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                {/* Parameters based on mutation type */}
                {mapping.mutationType === "SetLed" && (
                  <div className="grid grid-cols-2 gap-2">
                    <div>
                      <Label htmlFor={`index-${index}`}>LED Index</Label>
                      <Input
                        id={`index-${index}`}
                        type="number"
                        min="0"
                        max="7"
                        value={mapping.parameters.index}
                        onChange={(e) =>
                          handleParameterChange(
                            index,
                            "index",
                            parseInt(e.target.value)
                          )
                        }
                      />
                    </div>
                    <div>
                      <Label htmlFor={`on-${index}`}>State</Label>
                      <Select
                        value={mapping.parameters.on ? "true" : "false"}
                        onValueChange={(value) =>
                          handleParameterChange(index, "on", value === "true")
                        }
                      >
                        <SelectTrigger id={`on-${index}`}>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="true">ON</SelectItem>
                          <SelectItem value="false">OFF</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </div>
                )}

                {mapping.mutationType === "SetAllLeds" && (
                  <div>
                    <Label htmlFor={`all-on-${index}`}>State</Label>
                    <Select
                      value={mapping.parameters.on ? "true" : "false"}
                      onValueChange={(value) =>
                        handleParameterChange(index, "on", value === "true")
                      }
                    >
                      <SelectTrigger id={`all-on-${index}`}>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="true">ON</SelectItem>
                        <SelectItem value="false">OFF</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                )}

                {mapping.mutationType === "SetMode" && (
                  <div>
                    <Label htmlFor={`mode-${index}`}>Mode</Label>
                    <Select
                      value={mapping.parameters.mode}
                      onValueChange={(value) =>
                        handleParameterChange(index, "mode", value)
                      }
                    >
                      <SelectTrigger id={`mode-${index}`}>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="Manual">Manual</SelectItem>
                        <SelectItem value="Automatic">Automatic</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                )}
              </div>

              <Button
                variant="ghost"
                size="icon"
                onClick={() => handleRemoveMapping(index)}
                className="ml-2"
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            </div>

            {index < editingMappings.length - 1 && (
              <Separator className="mt-4" />
            )}
          </div>
        ))}

        <Button onClick={handleAddMapping} variant="outline" className="w-full">
          <Plus className="h-4 w-4 mr-2" />
          Add Action Mapping
        </Button>
      </CardContent>
    </Card>
  );
};
