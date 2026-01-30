import React, { useEffect, useState } from "react";
import { cn } from "@/lib/utils";

interface BarberPoleProps {
  active?: boolean;
  size?: number;
  className?: string;
}

/**
 * BarberPole component - Animated diagonal stripes indicating axis movement
 * Matches the NcAxisHeader BarberPole from TwinSharp .NET
 */
export function BarberPole({ active = false, size = 36, className }: BarberPoleProps) {
  const [offset, setOffset] = useState(0);

  useEffect(() => {
    if (!active) {
      setOffset(0);
      return;
    }

    // Animate diagonal stripes when active (matching .NET animation)
    const interval = setInterval(() => {
      setOffset((prev) => (prev + 1) % 20);
    }, 50);

    return () => clearInterval(interval);
  }, [active]);

  return (
    <div
      className={cn(
        "relative overflow-hidden rounded border-2 border-border",
        active ? "border-blue-500" : "border-muted",
        className
      )}
      style={{ width: size, height: size }}
    >
      <svg
        width={size}
        height={size}
        className={cn(
          "absolute inset-0",
          active ? "opacity-100" : "opacity-30"
        )}
      >
        <defs>
          <pattern
            id="barber-stripes"
            patternUnits="userSpaceOnUse"
            width="20"
            height="20"
            patternTransform={`rotate(45) translate(${offset} 0)`}
          >
            <rect x="0" y="0" width="10" height="20" fill={active ? "#3b82f6" : "#555"} />
            <rect x="10" y="0" width="10" height="20" fill={active ? "#1d4ed8" : "#333"} />
          </pattern>
        </defs>
        <rect width={size} height={size} fill="url(#barber-stripes)" />
      </svg>
      
      {/* Center indicator */}
      <div className="absolute inset-0 flex items-center justify-center">
        <div
          className={cn(
            "w-3 h-3 rounded-full",
            active ? "bg-green-500 animate-pulse" : "bg-gray-500"
          )}
        />
      </div>
    </div>
  );
}
