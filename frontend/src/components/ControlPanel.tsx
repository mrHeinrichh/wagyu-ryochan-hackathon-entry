"use client";

import { ChevronDown, Clock3, Eye, Filter, FlaskConical, LoaderCircle, Play, SlidersHorizontal, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import type { TokensResponse } from "@/lib/api";
import type { PulseRequestBody, TokenInfo, WatchItem } from "@/lib/types";
import { shortDate } from "@/lib/format";
import TokenIcon from "./TokenIcon";

const TIMEFRAMES = ["6h", "24h", "7d", "30d"] as const;
const REGIONS = ["Global", "US", "Asia", "Europe"] as const;
const SOURCES = ["Tavily", "Official", "Crypto-native", "Mainstream"] as const;

interface ControlPanelProps {
  tokens: TokenInfo[];
  tokenCatalog: TokensResponse | null;
  tokenCatalogError: string | null;
  tokensLoading: boolean;
  watchlist: WatchItem[];
  running: boolean;
  cooldownSeconds: number;
  onRun: (body: PulseRequestBody) => void;
  onDemo: (body: PulseRequestBody) => void;
  onWatch: (symbol: string) => void;
  onRemoveWatch: (symbol: string) => void;
}

export default function ControlPanel({
  tokens,
  tokenCatalog,
  tokenCatalogError,
  tokensLoading,
  watchlist,
  running,
  cooldownSeconds,
  onRun,
  onDemo,
  onWatch,
  onRemoveWatch,
}: ControlPanelProps) {
  const [symbol, setSymbol] = useState("BNB");
  const [thesis, setThesis] = useState("");
  const [timeframe, setTimeframe] = useState<string>("24h");
  const [regions, setRegions] = useState<string[]>(["Global", "Asia"]);
  const [sources, setSources] = useState<string[]>(["Tavily", "Crypto-native"]);
  const [riskBudget, setRiskBudget] = useState(0.5);
  const normalizedSymbol = symbol.trim().toUpperCase();
  const selectedToken = useMemo(
    () => tokens.find((token) => token.symbol.toUpperCase() === normalizedSymbol),
    [normalizedSymbol, tokens],
  );
  const isWatched = useMemo(
    () => watchlist.some((item) => item.symbol.toUpperCase() === normalizedSymbol),
    [normalizedSymbol, watchlist],
  );
  const tokenStatus = useMemo(() => {
    if (tokensLoading) return "Loading market";
    if (tokenCatalogError) return "Unavailable";
    if (tokenCatalog?.data_mode === "live") {
      return tokenCatalog.cached ? "Live cache" : "Live market";
    }
    return "Fallback list";
  }, [tokenCatalog, tokenCatalogError, tokensLoading]);

  useEffect(() => {
    if (tokensLoading) return;
    if (tokens.length === 0) {
      setSymbol("");
      return;
    }
    if (!tokens.some((token) => token.symbol.toUpperCase() === normalizedSymbol)) {
      setSymbol(tokens[0].symbol.toUpperCase());
    }
  }, [normalizedSymbol, tokens, tokensLoading]);

  const toggle = (list: string[], value: string, set: (next: string[]) => void) => {
    set(list.includes(value) ? list.filter((item) => item !== value) : [...list, value]);
  };

  const submit = (event: React.FormEvent) => {
    event.preventDefault();
    if (!normalizedSymbol) return;
    onRun({ symbol: normalizedSymbol, timeframe, regions, sources, thesis: thesis.trim(), risk_budget_pct: riskBudget });
  };

  const runDemo = () => {
    if (!normalizedSymbol) return;
    onDemo({ symbol: normalizedSymbol, timeframe, regions, sources, thesis: thesis.trim(), risk_budget_pct: riskBudget, demo: true });
  };

  return (
    <section className="control-panel" aria-label="Pulse controls" aria-busy={running}>
      <div className="control-panel-heading">
        <div>
          <span className="control-heading-icon" aria-hidden="true"><SlidersHorizontal /></span>
          <div>
            <p className="eyebrow">Research setup</p>
            <h2>Build a market pulse</h2>
          </div>
        </div>
        <span className={tokenCatalog?.data_mode === "live" ? "catalog-status live" : "catalog-status"}>
          <i aria-hidden="true" />
          {tokenStatus}
        </span>
      </div>
      <form onSubmit={submit} className="command-form">
        <div className="command-primary">
          <div className="field-row symbol-field" data-tour="token">
            <label className="field-label-row" htmlFor="symbolInput">
              <span>Asset</span>
              <span
                className="field-meta"
                title={tokenCatalog?.as_of ? `Catalog updated ${shortDate(tokenCatalog.as_of)}` : undefined}
              >
                {selectedToken?.market_cap_rank ? `Market rank #${selectedToken.market_cap_rank}` : "Token"}
              </span>
            </label>
            <div className="select-shell token-select-shell">
              <TokenIcon token={selectedToken} />
              <select
                id="symbolInput"
                name="symbol"
                value={symbol}
                disabled={tokensLoading || tokens.length === 0}
                onChange={(event) => setSymbol(event.target.value.toUpperCase())}
              >
                {tokensLoading ? <option value="BNB">Loading latest market...</option> : null}
                {!tokensLoading && tokens.length === 0 ? (
                  <option value="">Token list unavailable</option>
                ) : null}
                {tokens.map((token) => (
                  <option key={token.symbol} value={token.symbol}>
                    {token.market_cap_rank ? `#${token.market_cap_rank} ` : ""}
                    {token.symbol} - {token.name}
                  </option>
                ))}
              </select>
              <ChevronDown className="select-chevron" aria-hidden="true" />
            </div>
          </div>

          <div className="field-row thesis-field" data-tour="context">
            <label htmlFor="thesisInput">Context <span className="optional-label">optional</span></label>
            <input
              id="thesisInput"
              name="thesis"
              type="text"
              maxLength={300}
              value={thesis}
              placeholder="Add a headline, event, or question"
              onChange={(event) => setThesis(event.target.value)}
            />
          </div>

          <fieldset className="timeframe-field" data-tour="timeframe">
            <legend>Timeframe</legend>
            <div className="segmented">
              {TIMEFRAMES.map((value) => (
                <label key={value}>
                  <input
                    type="radio"
                    name="timeframe"
                    value={value}
                    checked={timeframe === value}
                    onChange={() => setTimeframe(value)}
                  />
                  <span>{value}</span>
                </label>
              ))}
            </div>
          </fieldset>

          <div className="command-actions" data-tour="actions">
            <button
              className="primary-button"
              type="submit"
              disabled={running || cooldownSeconds > 0 || tokensLoading || !normalizedSymbol}
            >
              {running ? (
                <LoaderCircle className="spin" aria-hidden="true" />
              ) : cooldownSeconds > 0 ? (
                <Clock3 aria-hidden="true" />
              ) : (
                <Play aria-hidden="true" />
              )}
              <span>{running ? "Analyzing" : cooldownSeconds > 0 ? `Wait ${cooldownSeconds}s` : "Analyze"}</span>
            </button>
            <button
              className="secondary-button watch-button"
              type="button"
              disabled={tokensLoading || !normalizedSymbol || isWatched}
              onClick={() => onWatch(normalizedSymbol)}
            >
              <Eye aria-hidden="true" />
              <span>{isWatched ? "Watching" : "Watch"}</span>
            </button>
            <button
              className="secondary-button demo-button"
              type="button"
              disabled={running || cooldownSeconds > 0 || !normalizedSymbol}
              onClick={runDemo}
            >
              <FlaskConical aria-hidden="true" />
              <span>Try sample</span>
            </button>
          </div>
        </div>

        <details className="advanced-controls" data-tour="filters">
          <summary>
            <span className="summary-title">
              <Filter aria-hidden="true" />
              Filters &amp; risk
            </span>
            <span className="summary-meta">
              {regions.length} regions - {sources.length} sources
              <ChevronDown className="chevron" aria-hidden="true" />
            </span>
          </summary>
          <div className="filter-groups">
            <fieldset>
              <legend>Regions</legend>
              <div className="check-grid">
                {REGIONS.map((value) => (
                  <label key={value}>
                    <input
                      type="checkbox"
                      name="regions"
                      value={value}
                      checked={regions.includes(value)}
                      onChange={() => toggle(regions, value, setRegions)}
                    />
                    <span>{value}</span>
                  </label>
                ))}
              </div>
            </fieldset>

            <fieldset>
              <legend>Sources</legend>
              <div className="check-grid">
                {SOURCES.map((value) => (
                  <label key={value}>
                    <input
                      type="checkbox"
                      name="sources"
                      value={value}
                      checked={sources.includes(value)}
                      onChange={() => toggle(sources, value, setSources)}
                    />
                    <span>{value}</span>
                  </label>
                ))}
              </div>
            </fieldset>

            <fieldset className="risk-control">
              <div className="risk-label">
                <legend>Paper risk limit</legend>
                <output htmlFor="riskBudget">{riskBudget.toFixed(1)}%</output>
              </div>
              <input
                id="riskBudget"
                type="range"
                min="0.1"
                max="2"
                step="0.1"
                value={riskBudget}
                onChange={(event) => setRiskBudget(Number(event.target.value))}
              />
              <p>Maximum simulated portfolio risk per setup.</p>
            </fieldset>
          </div>
        </details>
      </form>

      <details className="watch-panel" data-tour="watchlist">
        <summary>
          <span className="summary-title">
            <Eye aria-hidden="true" />
            Watchlist
          </span>
          <span className="summary-meta">
            {watchlist.length}
            <ChevronDown className="chevron" aria-hidden="true" />
          </span>
        </summary>
        <div className="watchlist">
          {watchlist.length === 0 ? (
            <div className="empty-mini">No watched tokens</div>
          ) : (
            watchlist.map((item) => (
              <div key={item.symbol} className="watch-item">
                <div>
                  <strong>{item.symbol}</strong>
                  <span>
                    {item.interval_minutes} min - next {shortDate(item.next_check_at)}
                  </span>
                </div>
                <button
                  type="button"
                  className="icon-button danger"
                  title={`Remove ${item.symbol}`}
                  aria-label={`Remove ${item.symbol} from watch mode`}
                  onClick={() => onRemoveWatch(item.symbol)}
                >
                  <X aria-hidden="true" />
                </button>
              </div>
            ))
          )}
        </div>
      </details>
    </section>
  );
}
