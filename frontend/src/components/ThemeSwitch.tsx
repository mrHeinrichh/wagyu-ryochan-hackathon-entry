"use client";

import { useEffect } from "react";
import { useAppStore } from "@/store/app";
import type { ThemeChoice } from "@/lib/types";

const OPTIONS: { value: ThemeChoice; label: string; icon: React.ReactNode }[] = [
  {
    value: "light",
    label: "Light",
    icon: (
      <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
        <circle cx="12" cy="12" r="4.2" fill="none" stroke="currentColor" strokeWidth="1.7" />
        <g stroke="currentColor" strokeWidth="1.7" strokeLinecap="round">
          <line x1="12" y1="2.6" x2="12" y2="5.2" />
          <line x1="12" y1="18.8" x2="12" y2="21.4" />
          <line x1="2.6" y1="12" x2="5.2" y2="12" />
          <line x1="18.8" y1="12" x2="21.4" y2="12" />
          <line x1="5.4" y1="5.4" x2="7.2" y2="7.2" />
          <line x1="16.8" y1="16.8" x2="18.6" y2="18.6" />
          <line x1="18.6" y1="5.4" x2="16.8" y2="7.2" />
          <line x1="7.2" y1="16.8" x2="5.4" y2="18.6" />
        </g>
      </svg>
    ),
  },
  {
    value: "system",
    label: "System",
    icon: (
      <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
        <rect x="3" y="4.5" width="18" height="12" rx="1.6" fill="none" stroke="currentColor" strokeWidth="1.7" />
        <line x1="9" y1="20" x2="15" y2="20" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" />
        <line x1="12" y1="16.5" x2="12" y2="20" stroke="currentColor" strokeWidth="1.7" />
      </svg>
    ),
  },
  {
    value: "dark",
    label: "Dark",
    icon: (
      <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
        <path
          d="M20 14.2A8 8 0 1 1 9.8 4a6.4 6.4 0 0 0 10.2 10.2Z"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.7"
          strokeLinejoin="round"
        />
      </svg>
    ),
  },
];

export default function ThemeSwitch() {
  const theme = useAppStore((s) => s.theme);
  const setTheme = useAppStore((s) => s.setTheme);

  // Sync the persisted theme onto the document on mount.
  useEffect(() => {
    setTheme(theme);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div className="theme-switch" role="group" aria-label="Theme">
      {OPTIONS.map((option) => (
        <button
          key={option.value}
          type="button"
          title={option.label}
          aria-label={option.label}
          aria-pressed={theme === option.value}
          className={theme === option.value ? "active" : undefined}
          onClick={() => setTheme(option.value)}
        >
          {option.icon}
        </button>
      ))}
    </div>
  );
}
