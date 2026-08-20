import { ChevronDown, Scale, ShieldAlert, ShieldCheck } from "lucide-react";
import type { DecisionReceipt, DebateArgument } from "@/lib/types";

export default function ConsensusPanel({ receipt }: { receipt: DecisionReceipt }) {
  const consensus = receipt.news_consensus;
  const odds = receipt.verdict.scenario_odds;
  const debate = receipt.verdict.debate;
  if (!consensus || !odds || !debate) return null;

  const isAiJudged = odds.generated_by.startsWith("openai:");
  const consensusIcon = consensus.state === "disputed" ? ShieldAlert : ShieldCheck;
  const ConsensusIcon = consensusIcon;

  return (
    <section className="consensus-panel" aria-label="Credibility consensus and bull bear debate">
      <div className="consensus-heading">
        <div>
          <span className={`consensus-icon ${consensus.state}`} aria-hidden="true">
            <ConsensusIcon />
          </span>
          <div>
            <p className="eyebrow">Credibility consensus</p>
            <h3>
              {consensus.state === "simulated"
                ? "Simulated source check"
                : `${consensus.credibility_label} confidence`}
            </h3>
          </div>
        </div>
        <strong>{consensus.credibility_score}/100</strong>
      </div>
      <p className="consensus-summary">{consensus.summary}</p>

      <div className="consensus-stats" aria-label="News verification counts">
        <span><strong>{consensus.independent_sources}</strong> independent sources</span>
        <span>
          <strong>{consensus.corroborated_claims}</strong>
          {receipt.data_mode === "simulated" ? "cross-matched fixtures" : "corroborated"}
        </span>
        <span><strong>{consensus.conflicting_claims}</strong> conflicting</span>
      </div>

      <div className="scenario-read">
        <div className="scenario-title">
          <div>
            <Scale aria-hidden="true" />
            <strong>Bull / bear scenario read</strong>
          </div>
          <span>{isAiJudged ? "AI Judge" : "Rules fallback"}</span>
        </div>
        <div
          className="scenario-bar"
          role="img"
          aria-label={`Bullish ${odds.bullish}%, bearish ${odds.bearish}%, unclear ${odds.unclear}%`}
        >
          <span className="bull" style={{ width: `${odds.bullish}%` }} />
          <span className="bear" style={{ width: `${odds.bearish}%` }} />
          <span className="unclear" style={{ width: `${odds.unclear}%` }} />
        </div>
        <div className="scenario-labels">
          <span className="bull"><i />Bullish <strong>{odds.bullish}%</strong></span>
          <span className="bear"><i />Bearish <strong>{odds.bearish}%</strong></span>
          <span className="unclear"><i />Unclear <strong>{odds.unclear}%</strong></span>
        </div>
        <p>{odds.basis}</p>
      </div>

      <details className="debate-disclosure">
        <summary>
          <span>
            <Scale aria-hidden="true" />
            Open agent debate
          </span>
          <span className="debate-decision">
            {debate.judge.decision}
            <ChevronDown className="chevron" aria-hidden="true" />
          </span>
        </summary>
        <div className="debate-body">
          <div className="debate-cases">
            <Argument side="bull" argument={debate.bull} />
            <Argument side="bear" argument={debate.bear} />
          </div>
          <div className="judge-result">
            <span>Judge · {debate.judge.confidence}% confidence</span>
            <strong>{debate.judge.decision}</strong>
            <p>{debate.judge.rationale}</p>
            <small>Decisive evidence: {debate.judge.decisive_evidence}</small>
          </div>
        </div>
      </details>
    </section>
  );
}

function Argument({ side, argument }: { side: "bull" | "bear"; argument: DebateArgument }) {
  return (
    <section className={`debate-case ${side}`}>
      <span>{argument.agent}</span>
      <strong>{argument.thesis}</strong>
      {argument.evidence.length ? (
        <ul>
          {argument.evidence.map((item) => <li key={item}>{item}</li>)}
        </ul>
      ) : <p>No qualified evidence for this side.</p>}
      {argument.weaknesses.length ? <small>Weakness: {argument.weaknesses[0]}</small> : null}
    </section>
  );
}
