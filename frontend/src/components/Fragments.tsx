import type { StoryCard, DecisionReceipt } from "@/lib/types";

export function DetailCell({ label, value }: { label: string; value: string | number | null | undefined }) {
  return (
    <div className="detail-cell">
      <span>{label}</span>
      <strong>{value === null || value === undefined || value === "" ? "-" : value}</strong>
    </div>
  );
}

export function ListBlock({ label, items }: { label: string; items?: string[] | null }) {
  if (!items || items.length === 0) {
    return (
      <div>
        <p className="eyebrow">{label}</p>
        <p className="muted">None declared.</p>
      </div>
    );
  }
  return (
    <div>
      <p className="eyebrow">{label}</p>
      <ol className="detail-list">
        {items.map((item, index) => (
          <li key={index}>{item}</li>
        ))}
      </ol>
    </div>
  );
}

export function Warnings({ warnings }: { warnings?: string[] | null }) {
  if (!warnings || warnings.length === 0) return null;
  return (
    <div>
      <p className="eyebrow">Warnings</p>
      <ol className="detail-list">
        {warnings.map((item, index) => (
          <li key={index}>{item}</li>
        ))}
      </ol>
    </div>
  );
}

export function RawJson({ receipt }: { receipt: DecisionReceipt }) {
  return (
    <details className="raw-json">
      <summary>Raw JSON for judges</summary>
      <pre>{JSON.stringify(receipt, null, 2)}</pre>
    </details>
  );
}

export function SourceLink({ card }: { card: StoryCard }) {
  if (!card.url || card.url.startsWith("about:")) return null;
  return (
    <div>
      <p className="eyebrow">Source link</p>
      <p>
        <a href={card.url} target="_blank" rel="noreferrer">
          {card.url}
        </a>
      </p>
    </div>
  );
}
