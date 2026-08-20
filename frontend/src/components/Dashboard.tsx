"use client";

import { useCallback, useEffect, useState } from "react";
import { ChevronDown, Layers3, LoaderCircle, Search } from "lucide-react";
import { ApiRequestError, api, type TokensResponse } from "@/lib/api";
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
import SourceHealth from "./SourceHealth";
import ProductTour from "./ProductTour";
import ResearchAssistant, { type AssistantTab } from "./ResearchAssistant";
import ResultViewControls, { type ResultView } from "./ResultViewControls";

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
  const [tourOpen, setTourOpen] = useState(false);
  const [assistantOpen, setAssistantOpen] = useState(false);
  const [assistantTab, setAssistantTab] = useState<AssistantTab>("chat");
  const [resultView, setResultView] = useState<ResultView>("split");
  const [analysisCooldownUntil, setAnalysisCooldownUntil] = useState(0);
  const [analysisCooldownSeconds, setAnalysisCooldownSeconds] = useState(0);

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

  useEffect(() => {
    const savedView = window.localStorage.getItem("pulse-result-view");
    if (savedView === "split" || savedView === "decision" || savedView === "news") {
      setResultView(savedView);
    }
  }, []);

  useEffect(() => {
    if (!analysisCooldownUntil) {
      setAnalysisCooldownSeconds(0);
      return;
    }
    const update = () => {
      const remaining = Math.max(0, Math.ceil((analysisCooldownUntil - Date.now()) / 1000));
      setAnalysisCooldownSeconds(remaining);
      if (remaining === 0) setAnalysisCooldownUntil(0);
    };
    update();
    const interval = window.setInterval(update, 250);
    return () => window.clearInterval(interval);
  }, [analysisCooldownUntil]);

  const changeResultView = useCallback((view: ResultView) => {
    setResultView(view);
    window.localStorage.setItem("pulse-result-view", view);
  }, []);

  const runPulse = useCallback(
    async (body: PulseRequestBody) => {
      const defaultCooldown = health?.guardrails?.analysis_cooldown_seconds ?? 12;
      let cooldown = defaultCooldown;
      setRunning(true);
      setFeedError(null);
      setFeedMode({ status: "partial", mode: "running" });
      try {
        const result = await api.reason(body);
        setReceipt(result);
        setFeedMode({ status: result.status, mode: result.data_mode });
        await loadHistory();
      } catch (error) {
        if (error instanceof ApiRequestError && error.retryAfterSeconds) {
          cooldown = error.retryAfterSeconds;
        }
        setFeedMode({ status: "unavailable", mode: "failed" });
        setFeedError((error as Error).message);
      } finally {
        setAnalysisCooldownSeconds(cooldown);
        setAnalysisCooldownUntil(Date.now() + cooldown * 1000);
        setRunning(false);
      }
    },
    [setReceipt, loadHistory, health?.guardrails?.analysis_cooldown_seconds],
  );

  const runDemo = useCallback(
    async (body: PulseRequestBody) => {
      let cooldown = 2;
      setRunning(true);
      setFeedError(null);
      setFeedMode({ status: "partial", mode: "simulated" });
      try {
        const result = await api.demo(body);
        setReceipt(result);
        setFeedMode({ status: result.status, mode: result.data_mode });
        setNotice("Sample receipt loaded. Every input is labelled simulated.");
        await loadHistory();
      } catch (error) {
        if (error instanceof ApiRequestError && error.retryAfterSeconds) {
          cooldown = error.retryAfterSeconds;
        }
        setFeedMode({ status: "unavailable", mode: "failed" });
        setFeedError((error as Error).message);
      } finally {
        setAnalysisCooldownSeconds(cooldown);
        setAnalysisCooldownUntil(Date.now() + cooldown * 1000);
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

  useEffect(() => {
    const receiptId = new URLSearchParams(window.location.search).get("receipt");
    if (receiptId) openReceipt(receiptId);
  }, [openReceipt]);

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
    <div
      className={`${sidebarCollapsed ? "app-shell sidebar-collapsed" : "app-shell"}${assistantOpen ? " assistant-open" : ""}`}
    >
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
        <WorkspaceHeader
          health={health}
          running={running}
          mode={feedMode}
          onStartTour={() => {
            setAssistantOpen(false);
            setTourOpen(true);
          }}
          onOpenFaq={() => {
            setTourOpen(false);
            setAssistantTab("faq");
            setAssistantOpen(true);
          }}
        />
        <ControlPanel
          tokens={tokens}
          tokenCatalog={tokenCatalog}
          tokenCatalogError={tokenCatalogError}
          tokensLoading={tokensLoading}
          watchlist={watchlist}
          running={running}
          cooldownSeconds={analysisCooldownSeconds}
          onRun={runPulse}
          onDemo={runDemo}
          onWatch={addWatch}
          onRemoveWatch={removeWatch}
        />
        {!receipt && !feedError ? (
          <section className="start-state" aria-live="polite" data-tour="empty-results">
            <span className="start-state-icon">
              {running ? <LoaderCircle className="spin" aria-hidden="true" /> : <Search aria-hidden="true" />}
            </span>
            <div>
              <h2>{running ? "Building your pulse" : "Ready for a market pulse"}</h2>
              <p>
                {running
                  ? "Checking current news and market evidence."
                  : "Choose a token and analyze the latest evidence."}
              </p>
            </div>
          </section>
        ) : (
          <>
            <ResultViewControls value={resultView} onChange={changeResultView} />
            <div className={`result-grid view-${resultView}`} key={receipt?.id || "error-result"}>
              {resultView !== "news" ? (
                <ReceiptDetail
                  receipt={receipt}
                  card={selectedCard}
                  onNotice={setNotice}
                  onCollapse={() => changeResultView("news")}
                />
              ) : null}
              {resultView !== "decision" ? (
                <Feed
                  receipt={receipt}
                  mode={feedMode}
                  error={feedError}
                  onCollapse={() => changeResultView("decision")}
                />
              ) : null}
            </div>
            {receipt ? (
              <details className="research-details">
                <summary>
                  <span className="summary-title">
                    <Layers3 aria-hidden="true" />
                    Research details
                  </span>
                  <span className="summary-meta">
                    Charts, sources and receipt
                    <ChevronDown className="chevron" aria-hidden="true" />
                  </span>
                </summary>
                <div className="research-details-body">
                  <SourceHealth receipt={receipt} />
                  <DecisionCharts receipt={receipt} card={selectedCard} />
                  <EvidenceMatrix receipt={receipt} />
                </div>
              </details>
            ) : null}
          </>
        )}
      </main>
      <div className={notice ? "toast visible" : "toast"} role="status" aria-live="polite">
        {notice}
      </div>
      <ResearchAssistant
        receipt={receipt}
        open={assistantOpen}
        tab={assistantTab}
        onOpenChange={setAssistantOpen}
        onTabChange={setAssistantTab}
      />
      <ProductTour open={tourOpen} onClose={() => setTourOpen(false)} />
    </div>
  );
}
