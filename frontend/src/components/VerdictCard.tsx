import { Activity, ArrowRight } from "lucide-react";
import type { DecisionReceipt } from "@/lib/types";
import { percentNumber } from "@/lib/format";

export default function VerdictCard({ receipt }: { receipt: DecisionReceipt }) {
  const verdict = receipt.verdict;
  const layer = receipt.reasoning_layer;
  const signal = layer?.signal || receipt.signal || verdict?.decision || "PENDING";
  const confidence = Number(percentNumber(layer?.confidence ?? receipt.confidence ?? verdict?.confidence)) || 0;
  const conclusion = receipt.summary.conclusion || layer?.reasoning || receipt.reasoning;

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
        <div className="confidence-value" aria-label={`${confidence}% confidence`}>
          <strong>{confidence}%</strong>
          <span>confidence</span>
        </div>
      </div>

      <p className="verdict-conclusion">{conclusion || "No conclusion returned."}</p>

      <div className="next-action">
        <ArrowRight aria-hidden="true" />
        <div>
          <span>Next action</span>
          <strong>{layer?.next_action || verdict?.recommended_next_action || "No action returned."}</strong>
        </div>
      </div>

      <div className="verdict-context" aria-label="Decision context">
        <span>{layer?.market_confirmation || "Market pending"}</span>
        <span>{layer?.sentiment || "Neutral sentiment"}</span>
        <span>{receipt.data_mode}</span>
      </div>
    </article>
  );
}
