import { Activity, Database, FlaskConical, LoaderCircle } from "lucide-react";
import type { HealthResponse, RunMode } from "@/lib/types";

interface WorkspaceHeaderProps {
  health: HealthResponse | null;
  running: boolean;
  mode: RunMode | null;
}

export default function WorkspaceHeader({ health, running, mode }: WorkspaceHeaderProps) {
  const ryoLabel = health?.ryo_configured
    ? "RYO live"
    : health?.ryo_mock_enabled
      ? "RYO simulation"
      : "RYO unavailable";
  const ryoTone = health?.ryo_configured ? "ok" : health?.ryo_mock_enabled ? "warn" : "bad";
  const runLabel = running ? "Analyzing" : mode ? `${mode.status} / ${mode.mode}` : "Ready";

  return (
    <header className="workspace-header">
      <div className="workspace-title">
        <p className="eyebrow">Global token news pulse</p>
        <h2>Market intelligence</h2>
      </div>
      <div className="workspace-status" aria-label="Workspace status">
        <span className={`status-chip ${health?.status === "ok" ? "ok" : "warn"}`}>
          <Activity aria-hidden="true" />
          Backend {health?.status ?? "checking"}
        </span>
        <span className={`status-chip ${ryoTone}`}>
          {health?.ryo_mock_enabled ? <FlaskConical aria-hidden="true" /> : <Database aria-hidden="true" />}
          {ryoLabel}
        </span>
        <span className={`status-chip ${running ? "running" : "neutral"}`} aria-live="polite">
          {running ? <LoaderCircle className="spin" aria-hidden="true" /> : <Activity aria-hidden="true" />}
          {runLabel}
        </span>
      </div>
    </header>
  );
}
