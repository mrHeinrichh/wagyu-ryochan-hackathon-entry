import { ChevronDown, Database } from "lucide-react";
import type { DecisionReceipt } from "@/lib/types";
import { percentText } from "@/lib/format";

interface EvidenceCardData {
  title: string;
  status: string;
  mode: string;
  text: string;
}

function buildCards(receipt: DecisionReceipt): EvidenceCardData[] {
  const verdict = receipt.verdict;
  const layer = receipt.reasoning_layer;
  const availabilityCards: EvidenceCardData[] = receipt.availability.map((item) => ({
    title: item.source,
    status: item.status,
    mode: item.data_mode,
    text: item.detail,
  }));
  const ryoCards: EvidenceCardData[] = receipt.ryo.map((item) => ({
    title: item.tool,
    status: item.status,
    mode: item.data_mode,
    text: item.summary || item.warnings.join(", ") || "No summary returned.",
  }));
  return [
    {
      title: "Reasoning layer",
      status: String(verdict?.generated_by || "").startsWith("openai") ? "ok" : "partial",
      mode: receipt.data_mode,
      text: `${layer?.signal || verdict?.decision || "PENDING"} - ${percentText(
        layer?.confidence ?? receipt.confidence,
      )} confidence - ${layer?.next_action || verdict?.recommended_next_action || "No action returned."}`,
    },
    ...availabilityCards,
    ...ryoCards,
  ];
}

export default function EvidenceMatrix({ receipt }: { receipt: DecisionReceipt | null }) {
  const cards = receipt ? buildCards(receipt) : [];
  return (
    <details className="evidence-panel">
      <summary className="panel-heading compact">
        <span className="panel-heading-title">
          <Database aria-hidden="true" />
          <span>
            <span className="eyebrow">Source coverage</span>
            <strong>Evidence matrix</strong>
          </span>
        </span>
        <span className="summary-meta">
          {cards.length} sources
          <ChevronDown className="chevron" aria-hidden="true" />
        </span>
      </summary>
      <div className="evidence-grid">
        {cards.length === 0 ? (
          <div className="empty-state compact-empty">No evidence matrix available.</div>
        ) : (
          cards.map((card, index) => (
            <article key={`${card.title}-${index}`} className="evidence-card">
              <div className="evidence-card-heading">
                <h3>{card.title}</h3>
                <span className={`mini-badge ${card.status}`}>{card.status}</span>
              </div>
              <span className={`data-mode-label ${card.mode}`}>{card.mode}</span>
              <p>{card.text}</p>
            </article>
          ))
        )}
      </div>
    </details>
  );
}
