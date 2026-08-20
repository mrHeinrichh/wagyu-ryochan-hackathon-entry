import { AlertTriangle, Check, Database, Newspaper, ShieldCheck } from "lucide-react";
import type { DecisionReceipt } from "@/lib/types";

const ICONS = [Newspaper, Database, Check, ShieldCheck];

export default function DecisionPath({ receipt }: { receipt: DecisionReceipt }) {
  const steps = receipt.decision_chain || [];
  if (steps.length === 0) return null;

  return (
    <section className="decision-path" aria-labelledby="decision-path-title">
      <div className="section-intro">
        <div>
          <p className="eyebrow">Evidence to action</p>
          <h3 id="decision-path-title">Why this result</h3>
        </div>
        <span>{steps.length} checks</span>
      </div>
      <ol>
        {steps.map((step, index) => {
          const Icon = ICONS[index] || Check;
          return (
            <li key={step.key} className={`decision-step ${step.status}`}>
              <span className="step-icon"><Icon aria-hidden="true" /></span>
              <div>
                <span>{step.label}</span>
                <strong>{step.detail}</strong>
                <small>{step.source}</small>
              </div>
            </li>
          );
        })}
      </ol>
      <div className="invalidation-note">
        <AlertTriangle aria-hidden="true" />
        <div><span>What changes this</span><strong>{receipt.invalidation}</strong></div>
      </div>
    </section>
  );
}
