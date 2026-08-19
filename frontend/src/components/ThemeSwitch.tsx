"use client";

import { Monitor, Moon, Sun } from "lucide-react";
import { useEffect } from "react";
import { useAppStore } from "@/store/app";
import type { ThemeChoice } from "@/lib/types";

const OPTIONS: { value: ThemeChoice; label: string; icon: typeof Sun }[] = [
  { value: "light", label: "Light", icon: Sun },
  { value: "system", label: "System", icon: Monitor },
  { value: "dark", label: "Dark", icon: Moon },
];

export default function ThemeSwitch() {
  const theme = useAppStore((state) => state.theme);
  const setTheme = useAppStore((state) => state.setTheme);

  useEffect(() => {
    setTheme(theme);
    // The initial persisted value only needs to be applied once after hydration.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div className="theme-switch" role="group" aria-label="Theme">
      {OPTIONS.map((option) => {
        const Icon = option.icon;
        return (
          <button
            key={option.value}
            type="button"
            title={`${option.label} theme`}
            aria-label={`${option.label} theme`}
            aria-pressed={theme === option.value}
            className={theme === option.value ? "active" : undefined}
            onClick={() => setTheme(option.value)}
          >
            <Icon aria-hidden="true" />
          </button>
        );
      })}
    </div>
  );
}
