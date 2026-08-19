"use client";

import { useState } from "react";
import type { PulseRequestBody, TokenInfo, WatchItem } from "@/lib/types";
import { shortDate } from "@/lib/format";

const TIMEFRAMES = ["6h", "24h", "7d", "30d"] as const;
const REGIONS = ["Global", "US", "Asia", "Europe"] as const;
const SOURCES = ["Tavily", "Official", "Crypto-native", "Mainstream"] as const;

interface ControlPanelProps {
  tokens: TokenInfo[];
  watchlist: WatchItem[];
  running: boolean;
  onRun: (body: PulseRequestBody) => void;
  onWatch: (symbol: string) => void;
  onRemoveWatch: (symbol: string) => void;
}

export default function ControlPanel({
  tokens,
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

  const toggle = (list: string[], value: string, set: (next: string[]) => void) => {
    set(list.includes(value) ? list.filter((v) => v !== value) : [...list, value]);
  };

  const submit = (event: React.FormEvent) => {
    event.preventDefault();
    onRun({ symbol, timeframe, regions, sources, thesis });
  };

  return (
    <section className="control-panel" aria-label="Pulse controls">
      <form onSubmit={submit}>
        <div className="field-row">
          <label htmlFor="symbolInput">Token</label>
          <input
            id="symbolInput"
            name="symbol"
            list="tokenList"
            value={symbol}
            autoComplete="off"
            maxLength={16}
            onChange={(e) => setSymbol(e.target.value)}
          />
          <datalist id="tokenList">
            {tokens.map((token) => (
              <option key={token.symbol} value={token.symbol}>
                {token.name}
              </option>
            ))}
          </datalist>
        </div>

        <div className="field-row">
          <label htmlFor="thesisInput">News / event / user thesis</label>
          <textarea
            id="thesisInput"
            name="thesis"
            rows={5}
            maxLength={1200}
            value={thesis}
            placeholder="Example: Binance regulatory news is moving globally, but I need to know whether BNB market evidence confirms it or this is just noise."
            onChange={(e) => setThesis(e.target.value)}
          />
        </div>

        <fieldset>
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
                {value}
              </label>
            ))}
          </div>
        </fieldset>

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
                {value}
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
                {value}
              </label>
            ))}
          </div>
        </fieldset>

        <div className="button-row">
          <button className="primary-button" type="submit" disabled={running}>
            {running ? "Running" : "Run Reasoning"}
          </button>
          <button className="secondary-button" type="button" onClick={() => onWatch(symbol)}>
            Watch
          </button>
        </div>
      </form>

      <div className="watch-panel">
        <div className="rail-title">Watch mode</div>
        <div className="watchlist">
          {watchlist.length === 0 ? (
            <div className="empty-mini">No watched tokens</div>
          ) : (
            watchlist.map((item) => (
              <div key={item.symbol} className="watch-item">
                <strong>{item.symbol}</strong>
                <span>
                  {item.interval_minutes} min - next {shortDate(item.next_check_at)}
                </span>
                <button
                  type="button"
                  className="secondary-button"
                  style={{ marginTop: "8px" }}
                  onClick={() => onRemoveWatch(item.symbol)}
                >
                  Remove
                </button>
              </div>
            ))
          )}
        </div>
      </div>
    </section>
  );
}
