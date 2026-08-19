"use client";

import { ArrowRight, ChevronDown, Newspaper } from "lucide-react";
import type { DecisionReceipt, RunMode, StoryCard } from "@/lib/types";
import { percentText } from "@/lib/format";
import { useAppStore } from "@/store/app";

interface FeedProps {
  receipt: DecisionReceipt | null;
  mode: RunMode | null;
  error: string | null;
}

function metricValues(receipt: DecisionReceipt): (string | number)[] {
  return [
    receipt.signal || receipt.verdict?.decision || "-",
    percentText(receipt.confidence),
    receipt.summary.highest_impact ?? "-",
    receipt.ryo_tools_used?.length ?? receipt.ryo?.length ?? 0,
  ];
}

const METRIC_LABELS = ["Signal", "Confidence", "Top impact", "RYO tools"];

export default function Feed({ receipt, mode, error }: FeedProps) {
  const selectedCardId = useAppStore((state) => state.selectedCardId);
  const selectCard = useAppStore((state) => state.selectCard);

  const badge = mode ? `${mode.status} / ${mode.mode}` : "idle";
  const badgeClass = mode ? `mode-badge ${mode.status} ${mode.mode}` : "mode-badge";
  const metrics = receipt ? metricValues(receipt) : ["-", "-", "-", 0];
  const totalCards = receipt ? receipt.sections.reduce((sum, section) => sum + section.cards.length, 0) : 0;

  return (
    <section className="feed-panel" aria-label="Ranked evidence feed">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Ranked intelligence</p>
          <h2>{receipt ? receipt.summary.headline : "Decision feed"}</h2>
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
          <div className="empty-state error-state" role="alert">
            {error}
          </div>
        ) : !receipt ? (
          <div className="empty-state">
            <Newspaper aria-hidden="true" />
            <strong>No decision receipt yet</strong>
            <span>Evidence and ranked narratives will appear here.</span>
          </div>
        ) : totalCards === 0 ? (
          <div className="empty-state">
            <Newspaper aria-hidden="true" />
            <strong>No ranked evidence</strong>
            <span>This receipt did not produce any narrative cards.</span>
          </div>
        ) : (
          receipt.sections.map((section, index) => (
            <details key={section.key} className="feed-section" open={index < 2 && section.cards.length > 0}>
              <summary className="section-header">
                <span className="section-title">
                  <ChevronDown className="chevron" aria-hidden="true" />
                  {section.label}
                </span>
                <span className="count-badge">{section.cards.length}</span>
              </summary>
              <div className="story-list">
                {section.cards.length === 0 ? (
                  <div className="empty-mini">No cards in this group</div>
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
            </details>
          ))
        )}
      </div>
    </section>
  );
}

function StoryRow({ card, active, onSelect }: { card: StoryCard; active: boolean; onSelect: () => void }) {
  return (
    <article className={active ? "story-row active" : "story-row"}>
      <button className="story-hitbox" type="button" onClick={onSelect} aria-pressed={active}>
        <div className="story-main">
          <div className="story-topline">
            <span className="impact-pill">Impact {card.score.impact}</span>
            <span className="cluster-pill">{card.narrative_cluster}</span>
            <span className={`data-pill ${card.data_mode}`}>{card.data_mode}</span>
          </div>
          <h4>{card.headline}</h4>
          <p>{card.reasoning[0] || card.ryo_alignment}</p>
        </div>
        <div className="story-side">
          <span>{card.recommendation}</span>
          <ArrowRight aria-hidden="true" />
        </div>
      </button>
    </article>
  );
}
