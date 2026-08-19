"use client";

import { ChevronDown, Eye, Filter, LoaderCircle, Play, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import type { TokensResponse } from "@/lib/api";
import type { PulseRequestBody, TokenInfo, WatchItem } from "@/lib/types";
import { shortDate } from "@/lib/format";

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
  onRun: (body: PulseRequestBody) => void;
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
  onRun,
  onWatch,
  onRemoveWatch,
}: ControlPanelProps) {
  const [symbol, setSymbol] = useState("BNB");
  const [thesis, setThesis] = useState("");
  const [timeframe, setTimeframe] = useState<string>("24h");
  const [regions, setRegions] = useState<string[]>(["Global", "Asia"]);
  const [sources, setSources] = useState<string[]>(["Tavily", "Crypto-native"]);
  const normalizedSymbol = symbol.trim().toUpperCase();
  const isWatched = useMemo(
    () => watchlist.some((item) => item.symbol.toUpperCase() === normalizedSymbol),
    [normalizedSymbol, watchlist],
  );
  const tokenStatus = useMemo(() => {
    if (tokensLoading) return "Loading market";
    if (tokenCatalogError) return "Unavailable";
    if (tokenCatalog?.data_mode === "live") {
      return `${tokenCatalog.cached ? "Cached live" : "Live"} - ${tokens.length} assets`;
    }
    return `Fallback - ${tokens.length} assets`;
  }, [tokenCatalog, tokenCatalogError, tokens.length, tokensLoading]);

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
    onRun({ symbol: normalizedSymbol, timeframe, regions, sources, thesis: thesis.trim() });
  };

  return (
    <section className="control-panel" aria-label="Pulse controls" aria-busy={running}>
      <form onSubmit={submit} className="command-form">
        <div className="command-primary">
          <div className="field-row symbol-field">
            <label className="field-label-row" htmlFor="symbolInput">
              <span>Token</span>
              <span
                className={tokenCatalog?.data_mode === "live" ? "field-meta live" : "field-meta"}
                title={tokenCatalog?.as_of ? `Catalog updated ${shortDate(tokenCatalog.as_of)}` : undefined}
              >
                {tokenStatus}
              </span>
            </label>
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
          </div>

          <div className="field-row thesis-field">
            <label htmlFor="thesisInput">Market thesis or event</label>
            <textarea
              id="thesisInput"
              name="thesis"
              rows={2}
              maxLength={1200}
              value={thesis}
              placeholder="What changed, and what decision should the evidence test?"
              onChange={(event) => setThesis(event.target.value)}
            />
          </div>

          <fieldset className="timeframe-field">
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

          <div className="command-actions">
            <button
              className="primary-button"
              type="submit"
              disabled={running || tokensLoading || !normalizedSymbol}
            >
              {running ? <LoaderCircle className="spin" aria-hidden="true" /> : <Play aria-hidden="true" />}
              <span>{running ? "Analyzing" : "Run pulse"}</span>
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
          </div>
        </div>

        <details className="advanced-controls">
          <summary>
            <span className="summary-title">
              <Filter aria-hidden="true" />
              Evidence filters
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
          </div>
        </details>
      </form>

      <details className="watch-panel">
        <summary>
          <span className="summary-title">
            <Eye aria-hidden="true" />
            Watch mode
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
