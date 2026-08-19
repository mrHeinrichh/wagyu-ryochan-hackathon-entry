"use client";

import Image from "next/image";
import type { HealthResponse, ReceiptListItem } from "@/lib/types";
import { percentText, shortDate } from "@/lib/format";
import ThemeSwitch from "./ThemeSwitch";

type Klass = "ok" | "warn" | "bad";

function healthLines(health: HealthResponse | null): [string, string, Klass][] {
  if (!health) return [["Backend", "checking...", "warn"]];
  const ryoStatus: [string, Klass] = health.ryo_configured
    ? ["configured", "ok"]
    : health.ryo_mock_enabled
      ? ["mock mode", "warn"]
      : ["missing key", "bad"];
  return [
    ["Backend", health.status, "ok"],
    ["RYO MCP", ryoStatus[0], ryoStatus[1]],
    ["Tavily", health.tavily_configured ? "configured" : "missing key", health.tavily_configured ? "ok" : "warn"],
    ["OpenAI", health.openai_configured ? "configured" : "missing key", health.openai_configured ? "ok" : "warn"],
    ["CoinGecko", health.coingecko_configured ? "backup key" : "optional", health.coingecko_configured ? "ok" : "warn"],
    ["DeFiLlama", health.defillama_enabled ? "enabled" : "optional", health.defillama_enabled ? "ok" : "warn"],
    ["DexScreener", health.dexscreener_enabled ? "enabled" : "optional", health.dexscreener_enabled ? "ok" : "warn"],
    ["Reasoning data", health.ryo_mock_enabled ? "simulated RYO" : "real only", health.ryo_mock_enabled ? "warn" : "ok"],
    ["Watch loop", health.watch_loop_enabled ? "on" : "manual", health.watch_loop_enabled ? "ok" : "warn"],
  ];
}

interface SidebarProps {
  health: HealthResponse | null;
  healthError: string | null;
  history: ReceiptListItem[];
  onOpenReceipt: (id: string) => void;
}

export default function Sidebar({ health, healthError, history, onOpenReceipt }: SidebarProps) {
  return (
    <aside className="side-rail" aria-label="Workspace">
      <div className="brand-block">
        <div className="brand-mark">
          <Image src="/assets/ryo-mascot.png" alt="RYO-CHAN" width={46} height={46} />
        </div>
        <div>
          <h1>RYO Token Reasoning Layer</h1>
          <p>Market reads to decisions</p>
        </div>
      </div>

      <ThemeSwitch />

      <div className="source-stack">
        {healthError ? (
          <div className="source-line bad">
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

      <div className="history-block">
        <div className="rail-title">Receipts</div>
        <div className="receipt-history">
          {history.length === 0 ? (
            <div className="empty-mini">No receipts yet</div>
          ) : (
            history.map((item) => (
              <button
                key={item.id}
                type="button"
                className="history-item"
                onClick={() => onOpenReceipt(item.id)}
              >
                <strong>
                  {item.symbol} {item.signal || ""}
                </strong>
                <span>
                  {percentText(item.confidence)} - {item.data_mode} - {shortDate(item.created_at)}
                </span>
              </button>
            ))
          )}
        </div>
      </div>
    </aside>
  );
}
