import { Activity, ArrowRight, Database, Gauge } from "lucide-react";
import type { CSSProperties } from "react";
import type { DecisionReceipt } from "@/lib/types";
import { percentNumber } from "@/lib/format";
import { DetailCell } from "./Fragments";

export default function VerdictCard({ receipt }: { receipt: DecisionReceipt }) {
  const verdict = receipt.verdict;
  const layer = receipt.reasoning_layer;
  const signal = layer?.signal || receipt.signal || verdict?.decision || "PENDING";
  const confidence = Number(percentNumber(layer?.confidence ?? receipt.confidence ?? verdict?.confidence)) || 0;
  const gaugeStyle = { "--confidence": `${confidence}%` } as CSSProperties;

  return (
    <article className="verdict-card">
      <div className="verdict-top">
        <div className="verdict-label">
          <span className={`signal-icon ${signal.toLowerCase()}`}>
            <Activity aria-hidden="true" />
          </span>
          <div>
            <p className="eyebrow">Decision</p>
            <h3>{signal}</h3>
          </div>
        </div>
        <div
          className="confidence-gauge"
          style={gaugeStyle}
          role="img"
          aria-label={`${confidence}% confidence`}
        >
          <div>
            <strong>{confidence}%</strong>
            <span>confidence</span>
          </div>
        </div>
      </div>

      <div className="next-action">
        <ArrowRight aria-hidden="true" />
        <div>
          <span>Next action</span>
          <strong>{layer?.next_action || verdict?.recommended_next_action || "No action returned."}</strong>
        </div>
      </div>

      <div className="verdict-facts">
        <DetailCell label="Token" value={layer?.symbol || verdict?.token_symbol || receipt.symbol} />
        <DetailCell label="Market confirmation" value={layer?.market_confirmation || "-"} />
        <DetailCell label="Sentiment" value={layer?.sentiment || "-"} />
        <DetailCell label="Data mode" value={receipt.data_mode} />
      </div>

      <div className="reasoning-summary">
        <Gauge aria-hidden="true" />
        <p>{layer?.reasoning || receipt.reasoning || "No reasoning returned."}</p>
      </div>

      <div className="tool-summary">
        <Database aria-hidden="true" />
        <span>{(layer?.ryo_tools_used || receipt.ryo_tools_used || []).join(", ") || "No RYO tools recorded"}</span>
      </div>
    </article>
  );
}
