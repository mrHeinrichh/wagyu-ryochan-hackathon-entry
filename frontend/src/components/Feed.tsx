"use client";

import type { DecisionReceipt, StoryCard } from "@/lib/types";
import { percentText } from "@/lib/format";
import { useAppStore } from "@/store/app";

type FeedMode = { status: string; mode: string } | null;

interface FeedProps {
  receipt: DecisionReceipt | null;
  mode: FeedMode;
  error: string | null;
}

function metricValues(receipt: DecisionReceipt): (string | number)[] {
  return [
    receipt.signal || receipt.verdict?.decision || "-",
    percentText(receipt.confidence),
    receipt.unavailable_data?.length ?? receipt.verdict?.missing_data?.length ?? 0,
    receipt.ryo_tools_used?.length ?? receipt.ryo?.length ?? 0,
  ];
}

const METRIC_LABELS = ["Signal", "Confidence", "Missing", "RYO tools"];

export default function Feed({ receipt, mode, error }: FeedProps) {
  const selectedCardId = useAppStore((s) => s.selectedCardId);
  const selectCard = useAppStore((s) => s.selectCard);

  const badge = mode ? `${mode.status} / ${mode.mode}` : "idle";
  const badgeClass = mode ? `mode-badge ${mode.status} ${mode.mode}` : "mode-badge";
  const metrics = receipt ? metricValues(receipt) : ["-", "-", 0, 0];
  const totalCards = receipt ? receipt.sections.reduce((sum, s) => sum + s.cards.length, 0) : 0;

  return (
    <section className="feed-panel" aria-label="Ranked feed">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Inspectable reasoning output</p>
          <h2>{receipt ? receipt.summary.headline : "Turn market reads into CONFIRMED, WATCHLIST, or REJECTED"}</h2>
        </div>
        <div className={badgeClass}>{badge}</div>
      </div>

      <div className="metric-grid">
        {METRIC_LABELS.map((label, index) => (
          <div key={label} className="metric">
            <span>{label}</span>
            <strong>{metrics[index]}</strong>
          </div>
        ))}
      </div>

      <div className="feed-sections">
        {error ? (
          <div className="empty-state">{error}</div>
        ) : !receipt ? (
          <div className="empty-state">
            Add a token and thesis, then run the reasoning layer. It ranks evidence only when sources are available.
          </div>
        ) : totalCards === 0 ? (
          <div className="empty-state">
            No cards were produced. Check the source availability and run again with live keys.
          </div>
        ) : (
          receipt.sections.map((section) => (
            <section key={section.key} className="feed-section">
              <div className="section-header">
                <h3>{section.label}</h3>
                <span>{section.cards.length} cards</span>
              </div>
              <div className="story-list">
                {section.cards.length === 0 ? (
                  <div className="empty-mini">No cards in this bucket</div>
                ) : (
                  section.cards.map((card) => (
                    <StoryRow
                      key={card.id}
                      card={card}
                      active={card.id === selectedCardId}
                      onSelect={() => selectCard(card.id)}
                    />
                  ))
                )}
              </div>
            </section>
          ))
        )}
      </div>
    </section>
  );
}

function StoryRow({
  card,
  active,
  onSelect,
}: {
  card: StoryCard;
  active: boolean;
  onSelect: () => void;
}) {
  return (
    <article className={active ? "story-row active" : "story-row"}>
      <button className="story-hitbox" type="button" onClick={onSelect}>
        <div className="story-main">
          <div className="story-topline">
            <span className="impact-pill">impact {card.score.impact}</span>
            <span className="cluster-pill">{card.narrative_cluster}</span>
            <span className={`data-pill ${card.data_mode}`}>{card.data_mode}</span>
          </div>
          <h4>{card.headline}</h4>
          <p>{card.reasoning[0] || card.ryo_alignment}</p>
        </div>
        <div className="story-side">
          <strong>{card.score.impact}</strong>
          <span>{card.recommendation}</span>
        </div>
      </button>
    </article>
  );
}
