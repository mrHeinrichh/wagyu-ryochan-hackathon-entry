import type { DecisionReceipt } from "@/lib/types";
import { percentText } from "@/lib/format";

interface EvidenceCardData {
  title: string;
  status: string;
  text: string;
}

function buildCards(receipt: DecisionReceipt): EvidenceCardData[] {
  const verdict = receipt.verdict;
  const layer = receipt.reasoning_layer;
  const availabilityCards: EvidenceCardData[] = receipt.availability.map((item) => ({
    title: item.source,
    status: item.status,
    text: `${item.data_mode} - ${item.detail}`,
  }));
  const ryoCards: EvidenceCardData[] = receipt.ryo.map((item) => ({
    title: item.tool,
    status: item.status,
    text: item.summary || item.warnings.join(", ") || "No summary returned.",
  }));
  return [
    {
      title: "Reasoning layer signal",
      status: String(verdict?.generated_by || "").startsWith("openai") ? "ok" : "partial",
      text: `${layer?.signal || verdict?.decision || "PENDING"} - ${percentText(
        layer?.confidence ?? receipt.confidence,
      )} confidence - ${layer?.next_action || verdict?.recommended_next_action || "No action yet."}`,
    },
    ...availabilityCards,
    ...ryoCards,
  ];
}

export default function EvidenceMatrix({ receipt }: { receipt: DecisionReceipt | null }) {
  const cards = receipt ? buildCards(receipt) : [];
  return (
    <section className="evidence-panel" aria-label="Evidence and availability">
      <div className="panel-heading compact">
        <div>
          <p className="eyebrow">Evidence matrix</p>
          <h2>Sources and RYO tools</h2>
        </div>
      </div>
      <div className="evidence-grid">
        {cards.length === 0 ? (
          <div className="empty-state">Evidence appears after a pulse run.</div>
        ) : (
          cards.map((card, index) => (
            <article key={index} className="evidence-card">
              <h3>
                <span>{card.title}</span>
                <span className={`mini-badge ${card.status}`}>{card.status}</span>
              </h3>
              <p>{card.text}</p>
            </article>
          ))
        )}
      </div>
    </section>
  );
}
