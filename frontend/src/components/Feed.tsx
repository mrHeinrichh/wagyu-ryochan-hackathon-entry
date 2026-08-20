"use client";

import { AlertTriangle, ChevronDown, ExternalLink, Newspaper, PanelRightClose } from "lucide-react";
import { shortDate } from "@/lib/format";
import type { DecisionReceipt, NewsStory, RunMode, StoryCard } from "@/lib/types";
import { useAppStore } from "@/store/app";
import RegionalPulse from "./RegionalPulse";

interface FeedProps {
  receipt: DecisionReceipt | null;
  mode: RunMode | null;
  error: string | null;
  onCollapse: () => void;
}

export default function Feed({ receipt, mode, error, onCollapse }: FeedProps) {
  const selectedCardId = useAppStore((state) => state.selectedCardId);
  const selectCard = useAppStore((state) => state.selectCard);

  const badge = receipt
    ? receipt.data_mode === "live"
      ? "live"
      : receipt.data_mode === "simulated"
        ? "demo"
        : "mixed data"
    : error
      ? "unavailable"
      : "idle";
  const badgeClass = mode ? `mode-badge ${mode.status} ${mode.mode}` : "mode-badge";
  const cards = receipt
    ? receipt.sections
        .flatMap((section) => section.cards)
        .sort((left, right) => right.score.impact - left.score.impact)
    : [];
  const primaryCards = cards.slice(0, 3);
  const remainingCards = cards.slice(3);

  const renderStory = (card: StoryCard) => (
    <StoryRow
      key={card.id}
      card={card}
      story={receipt?.stories.find((item) => item.id === card.id)}
      active={card.id === selectedCardId}
      onSelect={() => selectCard(card.id)}
    />
  );

  return (
    <section className="feed-panel" aria-label="Ranked evidence feed">
      <div className="panel-heading" data-tour="news">
        <div>
          <p className="eyebrow">Top evidence</p>
          <h2>What matters now</h2>
        </div>
        <div className="panel-heading-actions">
          <div className={badgeClass}>{badge}</div>
          <button
            className="icon-button panel-collapse"
            type="button"
            aria-label="Hide news panel"
            title="Hide news panel"
            onClick={onCollapse}
          >
            <PanelRightClose aria-hidden="true" />
          </button>
        </div>
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
        ) : cards.length === 0 ? (
          <div className="empty-state">
            <Newspaper aria-hidden="true" />
            <strong>No ranked evidence</strong>
            <span>This receipt did not produce any narrative cards.</span>
          </div>
        ) : (
          <>
            <RegionalPulse convergence={receipt.regional_convergence} />
            <div className="story-list primary-stories">{primaryCards.map(renderStory)}</div>
            {remainingCards.length > 0 ? (
              <details className="feed-section more-stories">
                <summary className="section-header">
                  <span className="section-title">
                    <ChevronDown className="chevron" aria-hidden="true" />
                    More evidence
                  </span>
                  <span className="count-badge">{remainingCards.length}</span>
                </summary>
                <div className="story-list">{remainingCards.map(renderStory)}</div>
              </details>
            ) : null}
          </>
        )}
      </div>
    </section>
  );
}

function StoryRow({
  card,
  story,
  active,
  onSelect,
}: {
  card: StoryCard;
  story: NewsStory | undefined;
  active: boolean;
  onSelect: () => void;
}) {
  const whyItMatters = card.attention_reason || card.reasoning[0] || card.ryo_alignment;
  const canOpenSource = Boolean(
    card.url && !card.url.startsWith("about:") && !card.data_mode.includes("simulated"),
  );

  return (
    <details
      className={`news-story${active ? " active" : ""}${card.requires_attention ? " attention" : ""}`}
      onToggle={(event) => {
        if (event.currentTarget.open) onSelect();
      }}
    >
      <summary className="news-summary">
        <span className="news-icon" aria-hidden="true">
          <Newspaper />
        </span>
        <span className="news-summary-copy">
          <span className="news-meta-line">
            {card.requires_attention ? (
              <span className="attention-flag">
                <AlertTriangle aria-hidden="true" />
                Needs attention
              </span>
            ) : null}
            <span className="impact-pill">Impact {card.score.impact}</span>
            <span className="cluster-pill">{card.narrative_cluster}</span>
          </span>
          <span className="news-headline">{card.headline}</span>
          <span className="news-standfirst">{whyItMatters}</span>
        </span>
        <span className="news-expand" aria-hidden="true">
          <span>Read</span>
          <ChevronDown className="chevron" />
        </span>
      </summary>

      <article className="newspaper-article">
        {card.requires_attention ? (
          <div className="attention-banner" role="note">
            <AlertTriangle aria-hidden="true" />
            <div>
              <span>Why this needs attention</span>
              <strong>{card.attention_reason}</strong>
            </div>
          </div>
        ) : null}

        <header className="newspaper-header">
          <p className="newspaper-kicker">{card.narrative_cluster}</p>
          <h3>{card.headline}</h3>
          <p className="newspaper-dek">{card.reasoning[0] || card.ryo_alignment}</p>
          <div className="newspaper-byline">
            <strong>{card.source}</strong>
            <span>{card.region || "Global"}</span>
            <span>{shortDate(card.timestamp)}</span>
          </div>
        </header>

        <div className="newspaper-columns">
          <section className="newspaper-lead">
            <p className="article-label">Why it matters most</p>
            <p>{whyItMatters}</p>
          </section>
          {story?.content ? (
            <section>
              <p className="article-label">Source context</p>
              <p>{story.content}</p>
            </section>
          ) : null}
        </div>

        <div className="editorial-facts">
          <div>
            <span>Market check</span>
            <strong>{card.ryo_alignment}</strong>
          </div>
          <div>
            <span>What to do</span>
            <strong>{card.recommendation}</strong>
          </div>
        </div>

        {card.reasoning.length > 1 ? (
          <div className="supporting-checks">
            <p className="article-label">Supporting checks</p>
            <ul>
              {card.reasoning.slice(1).map((reason) => (
                <li key={reason}>{reason}</li>
              ))}
            </ul>
          </div>
        ) : null}

        {canOpenSource ? (
          <a className="newspaper-source-link" href={card.url} target="_blank" rel="noreferrer">
            Read original source
            <ExternalLink aria-hidden="true" />
          </a>
        ) : (
          <span className="simulated-source-note">Simulated source context</span>
        )}
      </article>
    </details>
  );
}
