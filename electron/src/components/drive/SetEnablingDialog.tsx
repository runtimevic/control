import React, { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface SetEnablingDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  currentEnabling: {
    controller: boolean;
    feedFw: boolean;
    feedBw: boolean;
  };
  currentOverride: number; // 0-10000 (100.00 = 100%)
  onConfirm: (enabling: {
    controller: boolean;
    feedFw: boolean;
    feedBw: boolean;
    override: number;
  }) => void;
}

/**
 * Set Enabling Dialog - Modal para configurar enabling del controlador
 * Replica la funcionalidad del diálogo "Set Enabling" de TwinSharp .NET
 */
export function SetEnablingDialog({
  open,
  onOpenChange,
  currentEnabling,
  currentOverride,
  onConfirm,
}: SetEnablingDialogProps) {
  const [controller, setController] = useState(currentEnabling.controller);
  const [feedFw, setFeedFw] = useState(currentEnabling.feedFw);
  const [feedBw, setFeedBw] = useState(currentEnabling.feedBw);
  const [override, setOverride] = useState((currentOverride / 100).toFixed(2));

  // Reset state when dialog opens
  React.useEffect(() => {
    if (open) {
      setController(currentEnabling.controller);
      setFeedFw(currentEnabling.feedFw);
      setFeedBw(currentEnabling.feedBw);
      setOverride((currentOverride / 100).toFixed(2));
    }
  }, [open, currentEnabling, currentOverride]);

  const handleAll = () => {
    setController(true);
    setFeedFw(true);
    setFeedBw(true);
    setOverride("100.00");
  };

  const handleConfirm = () => {
    const overrideValue = Math.max(0, Math.min(10000, Math.round(parseFloat(override) * 100)));
    onConfirm({
      controller,
      feedFw,
      feedBw,
      override: overrideValue,
    });
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[400px]">
        <DialogHeader>
          <DialogTitle>Set Enabling</DialogTitle>
          <DialogDescription>
            Configure controller enabling and override settings
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* Enabling Checkboxes */}
          <div className="space-y-3">
            <div className="flex items-center space-x-2">
              <Checkbox
                id="dialog-controller"
                checked={controller}
                onCheckedChange={(checked) => setController(checked as boolean)}
              />
              <Label htmlFor="dialog-controller" className="text-sm cursor-pointer">
                Controller Enable
              </Label>
            </div>

            <div className="flex items-center space-x-2">
              <Checkbox
                id="dialog-feedFw"
                checked={feedFw}
                onCheckedChange={(checked) => setFeedFw(checked as boolean)}
              />
              <Label htmlFor="dialog-feedFw" className="text-sm cursor-pointer">
                Feed Forward Enable
              </Label>
            </div>

            <div className="flex items-center space-x-2">
              <Checkbox
                id="dialog-feedBw"
                checked={feedBw}
                onCheckedChange={(checked) => setFeedBw(checked as boolean)}
              />
              <Label htmlFor="dialog-feedBw" className="text-sm cursor-pointer">
                Feed Backward Enable
              </Label>
            </div>
          </div>

          {/* Override Input */}
          <div className="space-y-2">
            <Label htmlFor="override" className="text-sm">
              Override (%)
            </Label>
            <Input
              id="override"
              type="number"
              value={override}
              onChange={(e) => setOverride(e.target.value)}
              min="0"
              max="100"
              step="0.01"
              className="font-mono"
            />
            <p className="text-xs text-muted-foreground">
              Range: 0.00 - 100.00
            </p>
          </div>
        </div>

        <DialogFooter className="gap-2">
          <Button variant="outline" onClick={handleAll}>
            All
          </Button>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleConfirm}>OK</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
