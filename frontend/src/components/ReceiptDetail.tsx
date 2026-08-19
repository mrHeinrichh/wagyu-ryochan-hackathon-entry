import type { DecisionReceipt, StoryCard } from "@/lib/types";
import { percentText, nullableScore } from "@/lib/format";
import VerdictCard from "./VerdictCard";
import { DetailCell, RawJson, SourceLink, Warnings } from "./Fragments";

interface ReceiptDetailProps {
  receipt: DecisionReceipt | null;
  card: StoryCard | undefined;
}

export default function ReceiptDetail({ receipt, card }: ReceiptDetailProps) {
  const title = receipt
    ? `${receipt.symbol} ${receipt.signal || receipt.verdict?.decision || "receipt"}`
    : "No receipt selected";

  return (
    <section className="receipt-panel" aria-label="Decision receipt">
      <div className="panel-heading compact">
        <div>
          <p className="eyebrow">Replayable decision receipt</p>
          <h2>{title}</h2>
        </div>
      </div>

      <div className="receipt-detail">
        {!receipt ? (
          <div className="empty-state">
            Every run creates an auditable receipt with signal, confidence, RYO calls, missing data and raw JSON.
          </div>
        ) : (
          <div className="receipt-summary">
            <VerdictCard receipt={receipt} />
            {card ? <CardBreakdown receipt={receipt} card={card} /> : <h3>{receipt.summary.conclusion}</h3>}
            <Warnings warnings={receipt.warnings} />
            <RawJson receipt={receipt} />
          </div>
        )}
      </div>
    </section>
  );
}

function CardBreakdown({ receipt, card }: { receipt: DecisionReceipt; card: StoryCard }) {
  return (
    <>
      <h3>{card.headline}</h3>
      <p>{receipt.summary.conclusion}</p>
      <div className="detail-grid">
        <DetailCell label="Signal" value={receipt.signal} />
        <DetailCell label="Confidence" value={percentText(receipt.confidence)} />
        <DetailCell label="Reasoning" value={receipt.reasoning} />
        <DetailCell label="Next action" value={receipt.next_action} />
        <DetailCell label="Recommendation" value={card.recommendation} />
        <DetailCell label="Card confidence" value={card.confidence} />
        <DetailCell label="Sentiment" value={card.sentiment} />
        <DetailCell label="Cluster" value={card.narrative_cluster} />
        <DetailCell label="Source" value={card.source} />
        <DetailCell label="Run ID" value={receipt.run_id} />
      </div>
      <div>
        <p className="eyebrow">Reasoning</p>
        <ol className="detail-list">
          {card.reasoning.map((item, index) => (
            <li key={index}>{item}</li>
          ))}
        </ol>
      </div>
      <div>
        <p className="eyebrow">Missing data</p>
        <p>{card.missing_data.length ? card.missing_data.join(", ") : "None declared for this card."}</p>
      </div>
      <div>
        <p className="eyebrow">RYO alignment</p>
        <p>{card.ryo_alignment}</p>
      </div>
      <div>
        <p className="eyebrow">Scoring</p>
        <div className="detail-grid">
          <DetailCell label="Impact" value={String(card.score.impact)} />
          <DetailCell label="Relevance" value={nullableScore(card.score.relevance)} />
          <DetailCell label="Credibility" value={nullableScore(card.score.credibility)} />
          <DetailCell label="Urgency" value={nullableScore(card.score.urgency)} />
          <DetailCell label="Market confirm" value={nullableScore(card.score.market_confirmation)} />
          <DetailCell label="Uncertainty" value={nullableScore(card.score.uncertainty)} />
        </div>
      </div>
      <SourceLink card={card} />
    </>
  );
}
