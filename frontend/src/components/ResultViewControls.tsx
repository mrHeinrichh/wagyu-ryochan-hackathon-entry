"use client";

import { Columns2, PanelLeft, PanelRight, PanelsTopLeft } from "lucide-react";

export type ResultView = "split" | "decision" | "news";

interface ResultViewControlsProps {
  value: ResultView;
  onChange: (view: ResultView) => void;
}

const VIEW_OPTIONS = [
  { value: "split", label: "Both", title: "Show decision and news", icon: Columns2 },
  { value: "decision", label: "Decision", title: "Focus decision", icon: PanelLeft },
  { value: "news", label: "News", title: "Focus news", icon: PanelRight },
] as const;

export default function ResultViewControls({ value, onChange }: ResultViewControlsProps) {
  return (
    <div className="result-viewbar">
      <div className="result-view-title">
        <PanelsTopLeft aria-hidden="true" />
        <span>Result view</span>
      </div>
      <div className="result-view-switch" role="group" aria-label="Choose result panels">
        {VIEW_OPTIONS.map((option) => {
          const Icon = option.icon;
          return (
            <button
              key={option.value}
              type="button"
              aria-pressed={value === option.value}
              title={option.title}
              onClick={() => onChange(option.value)}
            >
              <Icon aria-hidden="true" />
              <span>{option.label}</span>
            </button>
          );
        })}
      </div>
    </div>
  );
}
