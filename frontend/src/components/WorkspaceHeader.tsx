import { Activity, BookOpen, FlaskConical, LoaderCircle, Map } from "lucide-react";
import type { HealthResponse, RunMode } from "@/lib/types";
import ThemeSwitch from "./ThemeSwitch";

interface WorkspaceHeaderProps {
  health: HealthResponse | null;
  running: boolean;
  mode: RunMode | null;
  onStartTour: () => void;
  onOpenFaq: () => void;
}

export default function WorkspaceHeader({ health, running, mode, onStartTour, onOpenFaq }: WorkspaceHeaderProps) {
  const status = running
    ? { label: "Analyzing", tone: "running", icon: LoaderCircle }
    : mode?.status === "unavailable"
      ? { label: "Run unavailable", tone: "bad", icon: Activity }
      : health?.ryo_configured
        ? { label: "Live sources", tone: "ok", icon: Activity }
        : health?.ryo_mock_enabled
          ? { label: "Demo mode", tone: "warn", icon: FlaskConical }
          : { label: health ? "RYO key needed" : "Connecting", tone: "warn", icon: Activity };
  const StatusIcon = status.icon;

  return (
    <header className="workspace-header">
      <div className="workspace-title">
        <p className="eyebrow">Research workspace</p>
        <h2>Global market pulse</h2>
      </div>
      <div className="workspace-actions">
        <div className="workspace-help-actions">
          <button type="button" className="header-action" onClick={onOpenFaq}>
            <BookOpen aria-hidden="true" />
            <span>FAQ</span>
          </button>
          <button type="button" className="header-action" onClick={onStartTour}>
            <Map aria-hidden="true" />
            <span>Tour</span>
          </button>
        </div>
        <span className={`workspace-live ${status.tone}`} aria-live="polite">
          <StatusIcon className={running ? "spin" : undefined} aria-hidden="true" />
          {status.label}
        </span>
        <ThemeSwitch />
      </div>
    </header>
  );
}
