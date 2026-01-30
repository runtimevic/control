import React, { useEffect, useState } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";

type Props = {
  value: number;
  onSubmit: (v: number) => void;
  disabled?: boolean;
  step?: number;
  min?: number;
  max?: number;
  unit?: string;
};

export function NumberInputWithSubmit({
  value,
  onSubmit,
  disabled,
  step = 1,
  min,
  max,
  unit,
}: Props) {
  const [internal, setInternal] = useState<string>(String(value ?? ""));

  useEffect(() => {
    setInternal(String(value ?? ""));
  }, [value]);

  const submitValue = () => {
    const parsed = Number(internal);
    if (Number.isFinite(parsed)) {
      onSubmit(parsed);
    }
  };

  return (
    <div className="flex items-center gap-2">
      <Input
        type="number"
        value={internal}
        onChange={(e) => setInternal(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") submitValue();
        }}
        step={String(step)}
        min={min !== undefined ? String(min) : undefined}
        max={max !== undefined ? String(max) : undefined}
        disabled={disabled}
      />
      {unit && <span className="text-neutral-500">{unit}</span>}
      <Button onClick={submitValue} disabled={disabled}>
        Set
      </Button>
    </div>
  );
}
