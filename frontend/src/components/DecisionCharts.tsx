import { BarChart3 } from "lucide-react";
import type { DecisionReceipt, StoryCard } from "@/lib/types";

interface DecisionChartsProps {
  receipt: DecisionReceipt | null;
  card: StoryCard | undefined;
}

interface BarDatum {
  label: string;
  value: number | null;
  tone?: string;
}

function clampScore(value: number | null): number {
  return value === null ? 0 : Math.max(0, Math.min(100, value));
}

function ScoreBars({ data }: { data: BarDatum[] }) {
  return (
    <div className="score-bars" role="img" aria-label={data.map((item) => `${item.label} ${item.value ?? "not available"}`).join(", ")}>
      {data.map((item) => (
        <div key={item.label} className="score-row">
          <span>{item.label}</span>
          <div className="score-track" aria-hidden="true">
            <span className={item.tone} style={{ width: `${clampScore(item.value)}%` }} />
          </div>
          <strong>{item.value ?? "n/a"}</strong>
        </div>
      ))}
    </div>
  );
}

export default function DecisionCharts({ receipt, card }: DecisionChartsProps) {
  if (!receipt) return null;

  const distribution: BarDatum[] = [
    { label: "Position changing", value: receipt.summary.position_changing_count, tone: "critical" },
    { label: "Watch", value: receipt.summary.watch_count, tone: "watch" },
    { label: "Noise", value: receipt.summary.noise_count, tone: "noise" },
    { label: "Unverified", value: receipt.summary.unverified_count, tone: "unverified" },
  ];
  const total = distribution.reduce((sum, item) => sum + (item.value ?? 0), 0);
  const factors: BarDatum[] = card
    ? [
        { label: "Impact", value: card.score.impact, tone: "impact" },
        { label: "Credibility", value: card.score.credibility, tone: "credibility" },
        { label: "Urgency", value: card.score.urgency, tone: "urgency" },
        { label: "Market confirmation", value: card.score.market_confirmation, tone: "confirmation" },
        { label: "Uncertainty", value: card.score.uncertainty, tone: "uncertainty" },
      ]
    : [];

  return (
    <section className="charts-panel" aria-label="Decision charts">
      <div className="panel-heading compact">
        <div>
          <p className="eyebrow">Decision shape</p>
          <h2>Evidence at a glance</h2>
        </div>
        <BarChart3 aria-hidden="true" />
      </div>
      <div className="charts-grid">
        <figure className="chart-block">
          <figcaption>
            <strong>Narrative distribution</strong>
            <span>{total} ranked stories</span>
          </figcaption>
          <div className="distribution-bar" role="img" aria-label={distribution.map((item) => `${item.label} ${item.value}`).join(", ")}>
            {total > 0
              ? distribution.map((item) => (
                  <span
                    key={item.label}
                    className={item.tone}
                    style={{ width: `${((item.value ?? 0) / total) * 100}%` }}
                    title={`${item.label}: ${item.value}`}
                  />
                ))
              : null}
          </div>
          <div className="chart-legend">
            {distribution.map((item) => (
              <span key={item.label}>
                <i className={item.tone} aria-hidden="true" />
                {item.label} <strong>{item.value}</strong>
              </span>
            ))}
          </div>
        </figure>

        <figure className="chart-block">
          <figcaption>
            <strong>{card ? "Selected story factors" : "Story factors"}</strong>
            <span>{card?.source ?? "No story selected"}</span>
          </figcaption>
          {card ? <ScoreBars data={factors} /> : <div className="empty-mini">No story selected</div>}
        </figure>
      </div>
    </section>
  );
}
