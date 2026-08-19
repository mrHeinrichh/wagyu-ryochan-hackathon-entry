"use client";

import { useCallback, useEffect, useState } from "react";
import { api, type TokensResponse } from "@/lib/api";
import { findCard } from "@/lib/receipt";
import type {
  DecisionReceipt,
  HealthResponse,
  PulseRequestBody,
  ReceiptListItem,
  RunMode,
  TokenInfo,
  WatchItem,
} from "@/lib/types";
import { useAppStore } from "@/store/app";
import Sidebar from "./Sidebar";
import ControlPanel from "./ControlPanel";
import Feed from "./Feed";
import ReceiptDetail from "./ReceiptDetail";
import EvidenceMatrix from "./EvidenceMatrix";
import WorkspaceHeader from "./WorkspaceHeader";
import DecisionCharts from "./DecisionCharts";

export default function Dashboard() {
  const receipt = useAppStore((s) => s.currentReceipt);
  const selectedCardId = useAppStore((s) => s.selectedCardId);
  const setReceipt = useAppStore((s) => s.setReceipt);
  const sidebarCollapsed = useAppStore((s) => s.sidebarCollapsed);

  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [healthError, setHealthError] = useState<string | null>(null);
  const [tokens, setTokens] = useState<TokenInfo[]>([]);
  const [tokenCatalog, setTokenCatalog] = useState<TokensResponse | null>(null);
  const [tokenCatalogError, setTokenCatalogError] = useState<string | null>(null);
  const [tokensLoading, setTokensLoading] = useState(true);
  const [history, setHistory] = useState<ReceiptListItem[]>([]);
  const [watchlist, setWatchlist] = useState<WatchItem[]>([]);
  const [running, setRunning] = useState(false);
  const [feedMode, setFeedMode] = useState<RunMode | null>(null);
  const [feedError, setFeedError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const loadHistory = useCallback(async () => {
    try {
      setHistory(await api.receipts());
    } catch {
      setHistory([]);
    }
  }, []);

  const loadWatchlist = useCallback(async () => {
    try {
      setWatchlist(await api.watchlist());
    } catch {
      setWatchlist([]);
    }
  }, []);

  // Initial data loads.
  useEffect(() => {
    api.health().then(setHealth).catch((error: Error) => setHealthError(error.message));
    api
      .tokens()
      .then((payload) => {
        setTokenCatalog(payload);
        setTokens(payload.tokens);
      })
      .catch((error: Error) => {
        setTokenCatalogError(error.message);
        setTokens([]);
      })
      .finally(() => setTokensLoading(false));
    loadHistory();
    loadWatchlist();
  }, [loadHistory, loadWatchlist]);

  useEffect(() => {
    if (!notice) return;
    const timeout = window.setTimeout(() => setNotice(null), 3200);
    return () => window.clearTimeout(timeout);
  }, [notice]);

  const runPulse = useCallback(
    async (body: PulseRequestBody) => {
      setRunning(true);
      setFeedError(null);
      setFeedMode({ status: "partial", mode: "running" });
      try {
        const result = await api.reason(body);
        setReceipt(result);
        setFeedMode({ status: result.status, mode: result.data_mode });
        await loadHistory();
      } catch (error) {
        setFeedMode({ status: "unavailable", mode: "failed" });
        setFeedError((error as Error).message);
      } finally {
        setRunning(false);
      }
    },
    [setReceipt, loadHistory],
  );

  const openReceipt = useCallback(
    async (id: string) => {
      try {
        const result = await api.receipt(id);
        setReceipt(result);
        setFeedMode({ status: result.status, mode: result.data_mode });
      } catch (error) {
        setFeedError((error as Error).message);
      }
    },
    [setReceipt],
  );

  const addWatch = useCallback(
    async (symbol: string) => {
      try {
        await api.addWatch({ symbol, interval_minutes: 15 });
        await loadWatchlist();
        setNotice(`${symbol.toUpperCase()} added to watch mode.`);
      } catch (error) {
        setNotice((error as Error).message);
      }
    },
    [loadWatchlist],
  );

  const removeWatch = useCallback(
    async (symbol: string) => {
      try {
        await api.removeWatch(symbol);
        await loadWatchlist();
        setNotice(`${symbol.toUpperCase()} removed from watch mode.`);
      } catch (error) {
        setNotice((error as Error).message);
      }
    },
    [loadWatchlist],
  );

  const selectedCard = receipt ? findCard(receipt, selectedCardId) : undefined;

  return (
    <div className={sidebarCollapsed ? "app-shell sidebar-collapsed" : "app-shell"}>
      <a className="skip-link" href="#main-workbench">
        Skip to workspace
      </a>
      <Sidebar
        health={health}
        healthError={healthError}
        history={history}
        onOpenReceipt={openReceipt}
      />
      <main id="main-workbench" className="workbench" tabIndex={-1}>
        <WorkspaceHeader health={health} running={running} mode={feedMode} />
        <ControlPanel
          tokens={tokens}
          tokenCatalog={tokenCatalog}
          tokenCatalogError={tokenCatalogError}
          tokensLoading={tokensLoading}
          watchlist={watchlist}
          running={running}
          onRun={runPulse}
          onWatch={addWatch}
          onRemoveWatch={removeWatch}
        />
        <div className="workspace-grid">
          <div className="primary-stack">
            <Feed receipt={receipt} mode={feedMode} error={feedError} />
            <DecisionCharts receipt={receipt} card={selectedCard} />
          </div>
          <ReceiptDetail receipt={receipt} card={selectedCard} />
        </div>
        <EvidenceMatrix receipt={receipt} />
      </main>
      <div className={notice ? "toast visible" : "toast"} role="status" aria-live="polite">
        {notice}
      </div>
    </div>
  );
}
