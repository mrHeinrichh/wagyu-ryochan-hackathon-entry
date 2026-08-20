//! Opposing evidence cases and an abstaining judge built from one receipt.

use crate::domain::{
    BullBearDebate, DebateArgument, DebateJudge, NewsConsensus, RankedSection, RyoToolEvidence,
    ScenarioOdds, StoryCard,
};
use crate::util::count_hits;

pub(crate) fn build_debate(
    symbol: &str,
    sections: &[RankedSection],
    consensus: &NewsConsensus,
    ryo: &[RyoToolEvidence],
    missing_data: &[String],
) -> (ScenarioOdds, BullBearDebate) {
    let cards = sections
        .iter()
        .flat_map(|section| section.cards.iter())
        .collect::<Vec<_>>();
    let mut bull_weight = 20.0_f32;
    let mut bear_weight = 20.0_f32;
    let mut unclear_weight = 25.0_f32;

    for card in &cards {
        let credibility = f32::from(card.verification.credibility_score.max(20)) / 100.0;
        let weight = f32::from(card.score.impact) * credibility;
        if matches!(card.category.as_str(), "unverified" | "noise") {
            unclear_weight += weight * 0.7;
        } else {
            match card.sentiment.as_str() {
                "bullish" => bull_weight += weight,
                "bearish" => bear_weight += weight,
                _ => unclear_weight += weight * 0.8,
            }
        }
    }

    let ryo_text = ryo
        .iter()
        .filter_map(|item| item.summary.as_deref())
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    let ryo_bull = count_hits(
        &ryo_text,
        &["bull", "positive", "upside", "momentum", "breakout"],
    );
    let ryo_bear = count_hits(
        &ryo_text,
        &["bear", "negative", "downside", "risk-off", "weak"],
    );
    if ryo_bull > ryo_bear {
        bull_weight += 30.0;
    } else if ryo_bear > ryo_bull {
        bear_weight += 30.0;
    } else {
        unclear_weight += 20.0;
    }
    unclear_weight += match consensus.state.as_str() {
        "strong" => 0.0,
        "emerging" => 15.0,
        "disputed" => 45.0,
        _ => 30.0,
    };

    let (bullish, bearish, unclear) = normalize_odds(bull_weight, bear_weight, unclear_weight);
    let direction_gap = bullish.abs_diff(bearish);
    let confidence = ((f32::from(consensus.credibility_score) * 0.55)
        + (f32::from(direction_gap) * 0.45))
        .round()
        .clamp(5.0, 92.0) as u8;
    let judge_decision = if unclear >= bullish.max(bearish) || direction_gap < 10 {
        "NO CONSENSUS"
    } else if bullish > bearish {
        "BULL CASE LEADS"
    } else {
        "BEAR CASE LEADS"
    };
    let decisive_evidence = decisive_card(&cards, judge_decision)
        .map(|card| {
            format!(
                "{} ({}, credibility {}/100)",
                card.headline, card.source, card.verification.credibility_score
            )
        })
        .unwrap_or_else(|| "No independently credible directional story dominates.".to_string());

    let bull = argument_for(symbol, "Bull Agent", "bullish", &cards);
    let bear = argument_for(symbol, "Bear Agent", "bearish", &cards);
    let rationale = match judge_decision {
        "BULL CASE LEADS" => format!(
            "Bullish evidence leads by {direction_gap} points, but the {unclear}% unclear scenario remains explicit."
        ),
        "BEAR CASE LEADS" => format!(
            "Bearish evidence leads by {direction_gap} points, but the {unclear}% unclear scenario remains explicit."
        ),
        _ => format!(
            "The directional gap is only {direction_gap} points or uncertainty dominates, so the judge abstains."
        ),
    };
    let basis = format!(
        "Evidence-weighted judgement from {} independent news domains, credibility consensus, and available RYO market context; not a price forecast.",
        consensus.independent_sources
    );
    let scenario_odds = ScenarioOdds {
        bullish,
        bearish,
        unclear,
        confidence,
        basis,
        generated_by: "deterministic-rust".to_string(),
    };
    let debate = BullBearDebate {
        bull,
        bear,
        judge: DebateJudge {
            decision: judge_decision.to_string(),
            confidence,
            rationale,
            decisive_evidence,
            missing_evidence: missing_data.iter().take(6).cloned().collect(),
        },
        generated_by: "deterministic-rust".to_string(),
    };
    (scenario_odds, debate)
}

fn argument_for(symbol: &str, agent: &str, stance: &str, cards: &[&StoryCard]) -> DebateArgument {
    let mut matching = cards
        .iter()
        .filter(|card| card.sentiment == stance && card.category != "unverified")
        .copied()
        .collect::<Vec<_>>();
    matching.sort_by_key(|card| std::cmp::Reverse(card.score.impact));
    let evidence = matching
        .iter()
        .take(3)
        .map(|card| {
            format!(
                "{} - {} (impact {}, credibility {})",
                card.source, card.headline, card.score.impact, card.verification.credibility_score
            )
        })
        .collect::<Vec<_>>();
    let opposing = if stance == "bullish" {
        "bearish"
    } else {
        "bullish"
    };
    let weaknesses = cards
        .iter()
        .filter(|card| card.sentiment == opposing || card.category == "unverified")
        .take(2)
        .map(|card| format!("{}: {}", card.source, card.headline))
        .collect::<Vec<_>>();
    DebateArgument {
        agent: agent.to_string(),
        thesis: if evidence.is_empty() {
            format!("No sufficiently supported {stance} case for {symbol} is present.")
        } else {
            format!(
                "The {stance} case for {symbol} is supported by {} ranked evidence item(s).",
                evidence.len()
            )
        },
        evidence,
        weaknesses,
    }
}

fn decisive_card<'a>(cards: &'a [&StoryCard], decision: &str) -> Option<&'a StoryCard> {
    let stance = if decision.starts_with("BULL") {
        "bullish"
    } else if decision.starts_with("BEAR") {
        "bearish"
    } else {
        return None;
    };
    cards
        .iter()
        .filter(|card| card.sentiment == stance && card.category != "unverified")
        .max_by_key(|card| {
            u16::from(card.score.impact) + u16::from(card.verification.credibility_score)
        })
        .copied()
}

fn normalize_odds(bull: f32, bear: f32, unclear: f32) -> (u8, u8, u8) {
    let total = (bull + bear + unclear).max(1.0);
    let bullish = ((bull / total) * 100.0).round().clamp(1.0, 98.0) as u8;
    let bearish = ((bear / total) * 100.0).round().clamp(1.0, 98.0) as u8;
    let unclear = 100_u8.saturating_sub(bullish).saturating_sub(bearish);
    (bullish, bearish, unclear)
}

#[cfg(test)]
mod tests {
    use super::normalize_odds;

    #[test]
    fn scenario_odds_always_sum_to_one_hundred() {
        let (bull, bear, unclear) = normalize_odds(91.0, 32.0, 44.0);
        assert_eq!(u16::from(bull) + u16::from(bear) + u16::from(unclear), 100);
    }
}
