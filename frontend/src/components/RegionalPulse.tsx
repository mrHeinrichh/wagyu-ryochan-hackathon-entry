import { Globe2 } from "lucide-react";
import type { RegionalConvergence } from "@/lib/types";

export default function RegionalPulse({ convergence }: { convergence: RegionalConvergence }) {
  if (!convergence?.signals?.length) return null;

  return (
    <section className="regional-pulse" aria-labelledby="regional-pulse-title">
      <div className="regional-heading">
        <div><Globe2 aria-hidden="true" /><strong id="regional-pulse-title">Regional read</strong></div>
        <span>{convergence.regime.replaceAll("_", " ")}</span>
      </div>
      <div className="region-strip">
        {convergence.signals.slice(0, 4).map((signal) => (
          <div key={signal.region} className={`region-signal ${signal.sentiment.toLowerCase().replace(" ", "-")}`}>
            <span>{signal.region}</span>
            <strong>{signal.sentiment}</strong>
            <small>{signal.story_count ? `${signal.story_count} sources · impact ${signal.average_impact}` : "No coverage"}</small>
          </div>
        ))}
      </div>
      <p>{convergence.summary} {convergence.strongest_disagreement}</p>
    </section>
  );
}
