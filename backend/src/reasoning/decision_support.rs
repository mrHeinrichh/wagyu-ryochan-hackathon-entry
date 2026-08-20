//! Deterministic decision-chain, regional, and paper-plan helpers.

use std::collections::BTreeSet;

use crate::domain::{
    DecisionChainStep, PracticePlan, RankedSection, ReasoningLayerOutput, ReasoningVerdict,
    RegionSignal, RegionalConvergence, RyoToolEvidence, StoryCard,
};

pub(crate) fn build_decision_chain(
    sections: &[RankedSection],
    verdict: &ReasoningVerdict,
    layer: &ReasoningLayerOutput,
    ryo: &[RyoToolEvidence],
) -> Vec<DecisionChainStep> {
    let top_card = sorted_cards(sections).into_iter().next();
    let change = top_card
        .map(|card| card.headline.clone())
        .unwrap_or_else(|| "No material narrative change was verified in this run.".to_string());
    let change_source = top_card
        .map(|card| card.source.clone())
        .unwrap_or_else(|| "News coverage".to_string());
    let market = ryo
        .iter()
        .find(|item| item.tool == "deep_analysis")
        .or_else(|| ryo.iter().find(|item| item.tool == "analyze_token"));
    let market_detail = market
        .and_then(|item| item.summary.clone())
        .unwrap_or_else(|| "RYO market confirmation was unavailable.".to_string());
    let market_status = market
        .map(|item| source_status(&item.status, &item.data_mode))
        .unwrap_or("unavailable")
        .to_string();

    vec![
        DecisionChainStep {
            key: "change".to_string(),
            label: "What changed".to_string(),
            detail: change,
            status: if top_card.is_some() {
                "supported"
            } else {
                "unavailable"
            }
            .to_string(),
            source: change_source,
        },
        DecisionChainStep {
            key: "market".to_string(),
            label: "Market check".to_string(),
            detail: market_detail,
            status: market_status,
            source: "RYO deep analysis".to_string(),
        },
        DecisionChainStep {
            key: "decision".to_string(),
            label: "Decision".to_string(),
            detail: format!(
                "{} at {}% confidence. {}",
                verdict.decision, verdict.confidence, layer.reasoning
            ),
            status: verdict.decision.to_lowercase(),
            source: "Reasoning layer".to_string(),
        },
        DecisionChainStep {
            key: "action".to_string(),
            label: "Next move".to_string(),
            detail: verdict.recommended_next_action.clone(),
            status: if verdict.decision == "CONFIRMED" {
                "ready"
            } else {
                "wait"
            }
            .to_string(),
            source: "Risk policy".to_string(),
        },
    ]
}

pub(crate) fn build_invalidation(verdict: &ReasoningVerdict) -> String {
    match verdict.decision.as_str() {
        "CONFIRMED" => "Downgrade the setup if RYO market confirmation turns mixed or contradicted, or if the leading narrative is credibly refuted.".to_string(),
        "WATCHLIST" => "Upgrade only after live RYO confirmation and credible source convergence; reject if the market check weakens or the narrative is refuted.".to_string(),
        _ => "Reopen the thesis only when new credible evidence appears and live RYO market confirmation improves.".to_string(),
    }
}

pub(crate) fn build_practice_plan(
    timeframe: &str,
    risk_budget_pct: f32,
    verdict: &ReasoningVerdict,
    layer: &ReasoningLayerOutput,
    ryo: &[RyoToolEvidence],
    invalidation: &str,
) -> PracticePlan {
    let live_ryo = ryo
        .iter()
        .any(|item| item.status == "ok" && item.data_mode == "live");
    let simulated = ryo.iter().any(|item| item.data_mode == "simulated");
    let directional = match layer.sentiment.as_str() {
        "bullish" => "PRACTICE LONG",
        "bearish" => "PRACTICE SHORT",
        _ => "NO ENTRY",
    };
    let ready = verdict.decision == "CONFIRMED" && live_ryo && directional != "NO ENTRY";
    let data_mode = if simulated {
        "simulated"
    } else if live_ryo {
        "live-evidence"
    } else {
        "unavailable"
    };

    PracticePlan {
        label: "Simulation only - no order will be placed".to_string(),
        status: if ready { "ready" } else { "observe_only" }.to_string(),
        stance: if ready { directional } else { "NO ENTRY" }.to_string(),
        entry_condition: if ready {
            "Enter a paper position only after the next timeframe close preserves the RYO-confirmed direction.".to_string()
        } else {
            "Wait for live RYO confirmation and directional agreement before recording a paper entry.".to_string()
        },
        invalidation: invalidation.to_string(),
        stop_method: if ready {
            "Place the simulated stop one ATR beyond the invalidation level returned by RYO deep_analysis.".to_string()
        } else {
            "No stop is calculated until live ATR and an entry condition are available.".to_string()
        },
        target: if ready {
            "Take the simulated setup at 2R, or exit sooner when the thesis invalidates."
        } else {
            "No target while the setup is observation-only."
        }
        .to_string(),
        risk_budget_pct,
        sizing_rule: format!(
            "Paper size = {risk_budget_pct:.1}% portfolio risk divided by the ATR-based stop distance."
        ),
        timeframe: timeframe.to_string(),
        basis: "RYO deep_analysis ATR preview plus ranked global news evidence".to_string(),
        data_mode: data_mode.to_string(),
    }
}

pub(crate) fn build_regional_convergence(
    requested_regions: &[String],
    sections: &[RankedSection],
) -> RegionalConvergence {
    let cards = sorted_cards(sections);
    let mut regions = requested_regions.iter().cloned().collect::<BTreeSet<_>>();
    for card in &cards {
        if let Some(region) = &card.region {
            regions.insert(region.clone());
        }
    }
    if regions.is_empty() {
        regions.insert("Global".to_string());
    }

    let signals = regions
        .into_iter()
        .map(|region| region_signal(&region, cards.as_slice()))
        .collect::<Vec<_>>();
    let covered_all = signals
        .iter()
        .filter(|signal| signal.story_count > 0)
        .collect::<Vec<_>>();
    let covered_specific = covered_all
        .iter()
        .copied()
        .filter(|signal| signal.region != "Global")
        .collect::<Vec<_>>();
    let covered = if covered_specific.is_empty() {
        covered_all
    } else {
        covered_specific
    };
    let sentiments = covered
        .iter()
        .map(|signal| signal.sentiment.as_str())
        .collect::<BTreeSet<_>>();
    let regime = if covered.len() < 2 {
        "insufficient_coverage"
    } else if sentiments.len() > 1
        && sentiments.contains("Bullish")
        && sentiments.contains("Bearish")
    {
        "diverging"
    } else if sentiments.len() == 1 && sentiments.contains("Bullish") {
        "converging_bullish"
    } else if sentiments.len() == 1 && sentiments.contains("Bearish") {
        "converging_bearish"
    } else {
        "mixed"
    };
    let summary = match regime {
        "diverging" => {
            "Regional narratives disagree, so the global signal needs more confirmation."
        }
        "converging_bullish" => "Covered regions broadly agree on a bullish narrative.",
        "converging_bearish" => "Covered regions broadly agree on a bearish narrative.",
        "mixed" => "Regional coverage is available, but direction remains mixed.",
        _ => "There is not enough region-specific coverage to claim global convergence.",
    };
    let strongest_disagreement = if regime == "diverging" {
        let labels = covered
            .iter()
            .map(|signal| format!("{} is {}", signal.region, signal.sentiment.to_lowercase()))
            .collect::<Vec<_>>()
            .join(", while ");
        format!("{labels}.")
    } else if signals.iter().any(|signal| signal.story_count == 0) {
        "At least one requested region has no attributable coverage in this run.".to_string()
    } else {
        "No material regional disagreement was detected.".to_string()
    };

    RegionalConvergence {
        regime: regime.to_string(),
        summary: summary.to_string(),
        strongest_disagreement,
        signals,
    }
}

fn sorted_cards(sections: &[RankedSection]) -> Vec<&StoryCard> {
    let mut cards = sections
        .iter()
        .flat_map(|section| section.cards.iter())
        .collect::<Vec<_>>();
    cards.sort_by_key(|card| std::cmp::Reverse(card.score.impact));
    cards
}

fn region_signal(region: &str, cards: &[&StoryCard]) -> RegionSignal {
    let matches = cards
        .iter()
        .copied()
        .filter(|card| region == "Global" || card.region.as_deref() == Some(region))
        .collect::<Vec<_>>();
    let bullish = matches
        .iter()
        .filter(|card| card.sentiment == "bullish")
        .count();
    let bearish = matches
        .iter()
        .filter(|card| card.sentiment == "bearish")
        .count();
    let sentiment = if matches.is_empty() {
        "No coverage"
    } else if bullish > bearish {
        "Bullish"
    } else if bearish > bullish {
        "Bearish"
    } else {
        "Mixed"
    };
    let average_impact = (!matches.is_empty()).then(|| {
        (matches
            .iter()
            .map(|card| card.score.impact as usize)
            .sum::<usize>()
            / matches.len()) as u8
    });
    RegionSignal {
        region: region.to_string(),
        sentiment: sentiment.to_string(),
        story_count: matches.len(),
        average_impact,
        position_changing_count: matches
            .iter()
            .filter(|card| card.category == "position-changing")
            .count(),
        coverage: if matches.is_empty() {
            "missing"
        } else {
            "available"
        }
        .to_string(),
    }
}

fn source_status(status: &str, data_mode: &str) -> &'static str {
    if status == "unavailable" {
        "unavailable"
    } else if data_mode == "simulated" {
        "simulated"
    } else if data_mode == "live" {
        "live"
    } else {
        "partial"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{StoryCard, StoryScore};

    #[test]
    fn regional_summary_does_not_invent_missing_coverage() {
        let card = StoryCard {
            id: "1".to_string(),
            headline: "BTC momentum improves".to_string(),
            source: "example.com".to_string(),
            url: "https://example.com".to_string(),
            region: Some("Asia".to_string()),
            language: None,
            timestamp: None,
            related_token: "BTC".to_string(),
            narrative_cluster: "macro".to_string(),
            sentiment: "bullish".to_string(),
            score: StoryScore {
                relevance: Some(90),
                credibility: Some(70),
                novelty: Some(60),
                urgency: Some(60),
                market_confirmation: Some(70),
                uncertainty: Some(20),
                impact: 74,
                formula: "test".to_string(),
            },
            ryo_alignment: "aligned".to_string(),
            missing_data: vec![],
            recommendation: "watch".to_string(),
            confidence: "high".to_string(),
            reasoning: vec![],
            category: "watch".to_string(),
            data_mode: "live".to_string(),
            requires_attention: false,
            attention_reason: None,
        };
        let sections = vec![RankedSection {
            key: "watch".to_string(),
            label: "Watch".to_string(),
            cards: vec![card],
        }];
        let result =
            build_regional_convergence(&["Asia".to_string(), "Europe".to_string()], &sections);
        assert_eq!(result.regime, "insufficient_coverage");
        assert_eq!(
            result
                .signals
                .iter()
                .find(|item| item.region == "Europe")
                .unwrap()
                .coverage,
            "missing"
        );
    }
}
