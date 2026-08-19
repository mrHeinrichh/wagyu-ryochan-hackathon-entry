"use client";

import Image from "next/image";
import {
  Activity,
  ChevronDown,
  Clock3,
  History,
  PanelLeftClose,
  PanelLeftOpen,
} from "lucide-react";
import { useRef } from "react";
import type { HealthResponse, ReceiptListItem } from "@/lib/types";
import { percentText, shortDate } from "@/lib/format";
import { useAppStore } from "@/store/app";
import ThemeSwitch from "./ThemeSwitch";

type Klass = "ok" | "warn" | "bad";

function healthLines(health: HealthResponse | null): [string, string, Klass][] {
  if (!health) return [["Backend", "checking", "warn"]];
  const ryoStatus: [string, Klass] = health.ryo_configured
    ? ["live", "ok"]
    : health.ryo_mock_enabled
      ? ["simulation", "warn"]
      : ["missing key", "bad"];
  return [
    ["Backend", health.status, health.status === "ok" ? "ok" : "warn"],
    ["RYO MCP", ryoStatus[0], ryoStatus[1]],
    ["Tavily", health.tavily_configured ? "ready" : "missing key", health.tavily_configured ? "ok" : "bad"],
    ["OpenAI", health.openai_configured ? "ready" : "missing key", health.openai_configured ? "ok" : "bad"],
    ["CoinGecko", health.coingecko_configured ? "ready" : "optional", health.coingecko_configured ? "ok" : "warn"],
    ["DeFiLlama", health.defillama_enabled ? "enabled" : "optional", health.defillama_enabled ? "ok" : "warn"],
    ["Watch loop", health.watch_loop_enabled ? "running" : "manual", health.watch_loop_enabled ? "ok" : "warn"],
  ];
}

interface SidebarProps {
  health: HealthResponse | null;
  healthError: string | null;
  history: ReceiptListItem[];
  onOpenReceipt: (id: string) => void;
}

export default function Sidebar({ health, healthError, history, onOpenReceipt }: SidebarProps) {
  const collapsed = useAppStore((state) => state.sidebarCollapsed);
  const toggleSidebar = useAppStore((state) => state.toggleSidebar);
  const historySection = useRef<HTMLDetailsElement>(null);

  const openReceipt = (id: string) => {
    onOpenReceipt(id);
    if (window.matchMedia("(max-width: 900px)").matches) {
      historySection.current?.removeAttribute("open");
    }
  };

  return (
    <aside className="side-rail" aria-label="Workspace">
      <div className="rail-brand-row">
        <div className="brand-block">
          <div className="brand-mark">
            <Image src="/assets/ryo-mascot.png" alt="Wagyu" width={44} height={44} priority />
          </div>
          <div className="brand-copy">
            <h1>Wagyu</h1>
            <p>Token news pulse</p>
          </div>
        </div>
        <button
          className="icon-button rail-toggle"
          type="button"
          title={collapsed ? "Expand navigation" : "Collapse navigation"}
          aria-label={collapsed ? "Expand navigation" : "Collapse navigation"}
          onClick={toggleSidebar}
        >
          {collapsed ? <PanelLeftOpen aria-hidden="true" /> : <PanelLeftClose aria-hidden="true" />}
        </button>
      </div>

      <ThemeSwitch />

      <details className="rail-section connections-section">
        <summary>
          <span className="rail-summary-title">
            <Activity aria-hidden="true" />
            <span className="rail-summary-label">Connections</span>
          </span>
          <span className="rail-summary-meta">
            <span className={`status-dot ${health?.status === "ok" ? "ok" : "warn"}`} />
            <ChevronDown className="chevron" aria-hidden="true" />
          </span>
        </summary>
        <div className="rail-content source-stack">
          {healthError ? (
            <div className="source-line bad" role="alert">
              <span>Backend</span>
              <strong>{healthError}</strong>
            </div>
          ) : (
            healthLines(health).map(([label, value, klass]) => (
              <div key={label} className={`source-line ${klass}`}>
                <span>{label}</span>
                <strong>{value}</strong>
              </div>
            ))
          )}
        </div>
      </details>

      <details ref={historySection} className="rail-section history-section">
        <summary>
          <span className="rail-summary-title">
            <History aria-hidden="true" />
            <span className="rail-summary-label">Receipts</span>
          </span>
          <span className="rail-summary-meta">
            <span className="count-badge">{history.length}</span>
            <ChevronDown className="chevron" aria-hidden="true" />
          </span>
        </summary>
        <div className="rail-content receipt-history">
          {history.length === 0 ? (
            <div className="empty-mini">No receipts yet</div>
          ) : (
            history.map((item) => (
              <button
                key={item.id}
                type="button"
                className="history-item"
                onClick={() => openReceipt(item.id)}
              >
                <span className="history-primary">
                  <strong>{item.symbol}</strong>
                  <span className={`signal-text ${item.signal.toLowerCase()}`}>{item.signal || "PENDING"}</span>
                </span>
                <span className="history-meta">
                  <span>{percentText(item.confidence)}</span>
                  <span>
                    <Clock3 aria-hidden="true" /> {shortDate(item.created_at)}
                  </span>
                </span>
              </button>
            ))
          )}
        </div>
      </details>
    </aside>
  );
}
