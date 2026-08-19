"use client";

import { useCallback, useEffect, useState } from "react";
import { api } from "@/lib/api";
import { findCard } from "@/lib/receipt";
import type {
  DecisionReceipt,
  HealthResponse,
  PulseRequestBody,
  ReceiptListItem,
  TokenInfo,
  WatchItem,
} from "@/lib/types";
import { useAppStore } from "@/store/app";
import Sidebar from "./Sidebar";
import ControlPanel from "./ControlPanel";
import Feed from "./Feed";
import ReceiptDetail from "./ReceiptDetail";
import EvidenceMatrix from "./EvidenceMatrix";

type FeedMode = { status: string; mode: string } | null;

export default function Dashboard() {
  const receipt = useAppStore((s) => s.currentReceipt);
  const selectedCardId = useAppStore((s) => s.selectedCardId);
  const setReceipt = useAppStore((s) => s.setReceipt);

  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [healthError, setHealthError] = useState<string | null>(null);
  const [tokens, setTokens] = useState<TokenInfo[]>([]);
  const [history, setHistory] = useState<ReceiptListItem[]>([]);
  const [watchlist, setWatchlist] = useState<WatchItem[]>([]);
  const [running, setRunning] = useState(false);
  const [feedMode, setFeedMode] = useState<FeedMode>(null);
  const [feedError, setFeedError] = useState<string | null>(null);

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
      .then((payload) => setTokens(payload.tokens ?? []))
      .catch(() => setTokens([]));
    loadHistory();
    loadWatchlist();
  }, [loadHistory, loadWatchlist]);

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
      } catch (error) {
        alert((error as Error).message);
      }
    },
    [loadWatchlist],
  );

  const removeWatch = useCallback(
    async (symbol: string) => {
      try {
        await api.removeWatch(symbol);
        await loadWatchlist();
      } catch {
        // Non-fatal; the list refreshes on the next action.
      }
    },
    [loadWatchlist],
  );

  const selectedCard = receipt ? findCard(receipt, selectedCardId) : undefined;

  return (
    <div className="app-shell">
      <Sidebar
        health={health}
        healthError={healthError}
        history={history}
        onOpenReceipt={openReceipt}
      />
      <main className="workbench">
        <ControlPanel
          tokens={tokens}
          watchlist={watchlist}
          running={running}
          onRun={runPulse}
          onWatch={addWatch}
          onRemoveWatch={removeWatch}
        />
        <Feed receipt={receipt} mode={feedMode} error={feedError} />
        <ReceiptDetail receipt={receipt} card={selectedCard} />
        <EvidenceMatrix receipt={receipt} />
      </main>
    </div>
  );
}
