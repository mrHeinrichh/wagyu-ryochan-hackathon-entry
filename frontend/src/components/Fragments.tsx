import { ChevronDown, ExternalLink } from "lucide-react";
import type { DecisionReceipt, StoryCard } from "@/lib/types";

export function DetailCell({ label, value }: { label: string; value: string | number | null | undefined }) {
  return (
    <div className="detail-cell">
      <span>{label}</span>
      <strong>{value === null || value === undefined || value === "" ? "-" : value}</strong>
    </div>
  );
}

export function Disclosure({
  title,
  meta,
  children,
  open = false,
}: {
  title: string;
  meta?: string | number;
  children: React.ReactNode;
  open?: boolean;
}) {
  return (
    <details className="disclosure" open={open}>
      <summary>
        <span>{title}</span>
        <span className="summary-meta">
          {meta}
          <ChevronDown className="chevron" aria-hidden="true" />
        </span>
      </summary>
      <div className="disclosure-body">{children}</div>
    </details>
  );
}

export function ListBlock({ label, items }: { label: string; items?: string[] | null }) {
  return (
    <div className="list-block">
      <p className="eyebrow">{label}</p>
      {!items || items.length === 0 ? (
        <p className="muted">None declared.</p>
      ) : (
        <ol className="detail-list">
          {items.map((item, index) => (
            <li key={index}>{item}</li>
          ))}
        </ol>
      )}
    </div>
  );
}

export function Warnings({ warnings }: { warnings?: string[] | null }) {
  if (!warnings || warnings.length === 0) return null;
  return (
    <div className="warning-block" role="note">
      <strong>Warnings</strong>
      <ul>
        {warnings.map((item, index) => (
          <li key={index}>{item}</li>
        ))}
      </ul>
    </div>
  );
}

export function RawJson({ receipt }: { receipt: DecisionReceipt }) {
  return (
    <details className="raw-json">
      <summary>
        <span>Raw JSON</span>
        <ChevronDown className="chevron" aria-hidden="true" />
      </summary>
      <pre>{JSON.stringify(receipt, null, 2)}</pre>
    </details>
  );
}

export function SourceLink({ card }: { card: StoryCard }) {
  if (!card.url || card.url.startsWith("about:")) return null;
  return (
    <a className="source-link" href={card.url} target="_blank" rel="noreferrer">
      Open source
      <ExternalLink aria-hidden="true" />
    </a>
  );
}
