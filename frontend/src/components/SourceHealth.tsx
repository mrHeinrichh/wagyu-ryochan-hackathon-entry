import { CircleCheck, CircleDashed, CircleX, Database } from "lucide-react";
import type { DecisionReceipt } from "@/lib/types";
import { shortDate } from "@/lib/format";

export default function SourceHealth({ receipt }: { receipt: DecisionReceipt }) {
  return (
    <section className="source-health-panel" aria-labelledby="source-health-title">
      <div className="panel-heading compact">
        <div className="panel-heading-title">
          <Database aria-hidden="true" />
          <div><p className="eyebrow">Failure-aware</p><strong id="source-health-title">Source health</strong></div>
        </div>
        <span className={`mode-badge ${receipt.status}`}>{receipt.status}</span>
      </div>
      <div className="source-health-list">
        {receipt.availability.map((item) => {
          const Icon = item.status === "ok" ? CircleCheck : item.status === "unavailable" ? CircleX : CircleDashed;
          return (
            <div key={`${item.source}-${item.as_of}`} className={`source-health-row ${item.status} ${item.data_mode}`}>
              <Icon aria-hidden="true" />
              <div><strong>{item.source}</strong><span>{item.detail}</span></div>
              <small>{item.data_mode}<br />{shortDate(item.as_of)}</small>
            </div>
          );
        })}
      </div>
    </section>
  );
}
