//! Grouping and summarizing scored cards, plus run-level status/data-mode.

use std::collections::{BTreeMap, BTreeSet};

use crate::domain::{NewsStory, RankedSection, ReceiptSummary, RyoToolEvidence, StoryCard};

/// Bucket cards by category and sort each bucket by descending impact, in the
/// fixed display order the UI expects.
pub(crate) fn rank_sections(cards: Vec<StoryCard>) -> Vec<RankedSection> {
    let mut buckets: BTreeMap<String, Vec<StoryCard>> = BTreeMap::new();
    for card in cards {
        buckets.entry(card.category.clone()).or_default().push(card);
    }
    for cards in buckets.values_mut() {
        cards.sort_by_key(|card| std::cmp::Reverse(card.score.impact));
    }
    let order = [
        ("position-changing", "Position-changing"),
        ("watch", "Watch closely"),
        ("unverified", "Unverified or conflicting"),
        ("noise", "Noise"),
    ];
    order
        .iter()
        .map(|(key, label)| RankedSection {
            key: (*key).to_string(),
            label: (*label).to_string(),
            cards: buckets.remove(*key).unwrap_or_default(),
        })
        .collect()
}

/// Build the roll-up summary (counts, highest impact, headline, conclusion).
pub(crate) fn summarize_receipt(
    symbol: &str,
    sections: &[RankedSection],
    story_count: usize,
    ryo: &[RyoToolEvidence],
) -> ReceiptSummary {
    let position_changing_count = section_count(sections, "position-changing");
    let watch_count = section_count(sections, "watch");
    let noise_count = section_count(sections, "noise");
    let unverified_count = section_count(sections, "unverified");
    let highest_impact = sections
        .iter()
        .flat_map(|section| section.cards.iter().map(|card| card.score.impact))
        .max();
    let ryo_available = ryo.iter().any(|item| item.status != "unavailable");

    let conclusion = if story_count == 0 {
        format!("No live news evidence was available for {symbol}; do not infer a changed view.")
    } else if position_changing_count > 0 {
        format!(
            "{symbol} has at least one story that may change a position view; inspect the receipt before acting."
        )
    } else if watch_count > 0 {
        format!(
            "{symbol} has watch-level narratives, but the evidence does not yet force a position change."
        )
    } else {
        format!("{symbol} news looks low-impact or unverified in this run.")
    };

    ReceiptSummary {
        headline: if ryo_available {
            format!("{symbol} global news ranked against RYO market evidence")
        } else {
            format!("{symbol} global news ranked; RYO market evidence unavailable")
        },
        conclusion,
        highest_impact,
        position_changing_count,
        watch_count,
        noise_count,
        unverified_count,
    }
}

/// Count the cards in a named section.
pub(crate) fn section_count(sections: &[RankedSection], key: &str) -> usize {
    sections
        .iter()
        .find(|section| section.key == key)
        .map(|section| section.cards.len())
        .unwrap_or(0)
}

/// Collapse the per-source data modes into one run-level label.
pub(crate) fn resolve_data_mode(stories: &[NewsStory], ryo: &[RyoToolEvidence]) -> String {
    let mut modes = BTreeSet::new();
    for story in stories {
        modes.insert(story.data_mode.as_str());
    }
    for tool in ryo {
        modes.insert(tool.data_mode.as_str());
    }
    let has_live = modes.contains("live");
    let has_user_provided = modes.contains("user-provided");
    if has_live && has_user_provided {
        "mixed".to_string()
    } else if has_live {
        "live".to_string()
    } else if has_user_provided {
        "user-provided".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Overall run status: `ok`, `partial`, or `unavailable`.
pub(crate) fn resolve_status(stories: &[NewsStory], ryo: &[RyoToolEvidence]) -> String {
    if stories.is_empty() && ryo.iter().all(|tool| tool.status == "unavailable") {
        "unavailable".to_string()
    } else if stories.is_empty() || ryo.iter().any(|tool| tool.status == "unavailable") {
        "partial".to_string()
    } else {
        "ok".to_string()
    }
}
