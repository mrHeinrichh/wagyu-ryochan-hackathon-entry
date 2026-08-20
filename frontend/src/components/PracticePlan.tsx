import { Crosshair, Shield, Target } from "lucide-react";
import type { PracticePlan as PracticePlanType } from "@/lib/types";

export default function PracticePlan({ plan }: { plan: PracticePlanType }) {
  if (!plan?.label) return null;
  const ready = plan.status === "ready";

  return (
    <section className={`practice-plan ${ready ? "ready" : "observe"}`} aria-labelledby="practice-plan-title">
      <div className="section-intro">
        <div>
          <p className="eyebrow">Paper position</p>
          <h3 id="practice-plan-title">{plan.stance}</h3>
        </div>
        <span className="simulation-label">{ready ? `${plan.risk_budget_pct}% risk` : "Simulation only"}</span>
      </div>
      <p className="practice-entry">{plan.entry_condition}</p>
      <div className="practice-facts">
        <div><Shield aria-hidden="true" /><span>Stop</span><strong>{plan.stop_method}</strong></div>
        <div><Target aria-hidden="true" /><span>Target</span><strong>{plan.target}</strong></div>
        <div><Crosshair aria-hidden="true" /><span>Size</span><strong>{plan.sizing_rule}</strong></div>
      </div>
    </section>
  );
}
