//! Source provenance and independent corroboration for news claims.

use std::collections::BTreeSet;

use crate::domain::{NewsConsensus, NewsStory, NewsVerification, StoryCard};
use crate::reasoning::classify::{narrative_cluster, sentiment};

pub(crate) fn verify_story(story: &NewsStory, all_stories: &[NewsStory]) -> NewsVerification {
    let (source_class, source_authority, primary_source) = classify_source(story);
    let mut corroborating_sources = BTreeSet::new();
    let mut conflicting_sources = BTreeSet::new();
    let story_text = format!("{} {}", story.headline, story.content).to_lowercase();
    let story_sentiment = sentiment(&story_text);

    for candidate in all_stories {
        if candidate.id == story.id || candidate.source.eq_ignore_ascii_case(&story.source) {
            continue;
        }
        if !claims_overlap(story, candidate) {
            continue;
        }
        let candidate_text = format!("{} {}", candidate.headline, candidate.content).to_lowercase();
        if claims_conflict(&story_text, &story_sentiment, &candidate_text) {
            conflicting_sources.insert(candidate.source.clone());
        } else {
            corroborating_sources.insert(candidate.source.clone());
        }
    }

    let corroboration_score = match corroborating_sources.len() {
        0 => 30,
        1 => 68,
        2 => 82,
        _ => 92,
    };
    let completeness = completeness_score(story);
    let mut credibility_score = ((source_authority as f32 * 0.5)
        + (corroboration_score as f32 * 0.35)
        + (completeness as f32 * 0.15))
        .round() as u8;
    if !conflicting_sources.is_empty() {
        credibility_score = credibility_score.saturating_sub(18);
    }
    if story.data_mode == "user-provided" {
        credibility_score = credibility_score.min(35);
    } else if story.data_mode == "simulated" {
        credibility_score = credibility_score.min(60);
    }

    let claim_status = if story.data_mode == "user-provided" {
        "user claim"
    } else if story.data_mode == "simulated" {
        "simulated"
    } else if !conflicting_sources.is_empty() {
        "conflicting"
    } else if !corroborating_sources.is_empty() {
        "corroborated"
    } else {
        "single source"
    };
    let credibility_label = credibility_label(credibility_score);
    let independent_source_count = 1 + corroborating_sources.len();
    let explanation = match claim_status {
        "corroborated" => format!(
            "{} independent domains report a materially similar claim; source type is {}.",
            independent_source_count, source_class
        ),
        "conflicting" => format!(
            "A materially similar claim has conflicting coverage from {}.",
            conflicting_sources
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ),
        "user claim" => {
            "This is user-supplied context and has no independent verification yet.".to_string()
        }
        "simulated" => {
            "This is a labelled demo fixture and is not evidence of a real event.".to_string()
        }
        _ => format!(
            "Only {} currently supports this claim; treat it as provisional.",
            story.source
        ),
    };

    NewsVerification {
        source_class,
        source_authority,
        credibility_score,
        credibility_label,
        claim_status: claim_status.to_string(),
        independent_source_count,
        corroborating_sources: corroborating_sources.into_iter().collect(),
        conflicting_sources: conflicting_sources.into_iter().collect(),
        primary_source,
        explanation,
    }
}

pub(crate) fn summarize_consensus(cards: &[StoryCard]) -> NewsConsensus {
    if cards.is_empty() {
        return NewsConsensus {
            state: "insufficient".to_string(),
            credibility_label: "Unavailable".to_string(),
            summary: "No live news evidence was available for a credibility consensus.".to_string(),
            ..NewsConsensus::default()
        };
    }

    let simulated = cards.iter().all(|card| card.data_mode == "simulated");

    let independent_sources = cards
        .iter()
        .map(|card| card.source.to_lowercase())
        .collect::<BTreeSet<_>>()
        .len();
    let primary_sources = cards
        .iter()
        .filter(|card| card.verification.primary_source)
        .map(|card| card.source.to_lowercase())
        .collect::<BTreeSet<_>>()
        .len();
    let corroborated_claims = cards
        .iter()
        .filter(|card| {
            card.verification.claim_status == "corroborated"
                || (simulated && card.verification.independent_source_count > 1)
        })
        .count();
    let single_source_claims = cards
        .iter()
        .filter(|card| {
            matches!(
                card.verification.claim_status.as_str(),
                "single source" | "user claim" | "simulated"
            ) && card.verification.independent_source_count == 1
        })
        .count();
    let conflicting_claims = cards
        .iter()
        .filter(|card| card.verification.claim_status == "conflicting")
        .count();
    let credibility_score = (cards
        .iter()
        .map(|card| u32::from(card.verification.credibility_score))
        .sum::<u32>()
        / cards.len() as u32) as u8;
    let state = if simulated {
        "simulated"
    } else if conflicting_claims > 0 {
        "disputed"
    } else if corroborated_claims >= 2 && credibility_score >= 70 {
        "strong"
    } else if corroborated_claims > 0 {
        "emerging"
    } else {
        "fragmented"
    };
    let summary = match state {
        "simulated" => format!(
            "Demo only: {corroborated_claims} fixture claims cross-match across {independent_sources} simulated domains; this is not live corroboration."
        ),
        "strong" => format!(
            "Strong consensus: {corroborated_claims} claims have independent support across {independent_sources} domains."
        ),
        "emerging" => format!(
            "Emerging consensus: some independent support exists, but {single_source_claims} claims remain single-source."
        ),
        "disputed" => format!(
            "Disputed evidence: {conflicting_claims} claims have materially conflicting coverage."
        ),
        _ => format!(
            "Fragmented evidence: {single_source_claims} claims currently depend on one source."
        ),
    };

    NewsConsensus {
        credibility_score,
        credibility_label: if simulated {
            "Demo only".to_string()
        } else {
            credibility_label(credibility_score)
        },
        independent_sources,
        primary_sources,
        corroborated_claims,
        single_source_claims,
        conflicting_claims,
        state: state.to_string(),
        summary,
    }
}

fn classify_source(story: &NewsStory) -> (String, u8, bool) {
    let source = story.source.to_lowercase();
    if story.data_mode == "user-provided" {
        return ("User-supplied claim".to_string(), 20, false);
    }
    if story.data_mode == "simulated" {
        return ("Simulated fixture".to_string(), 50, false);
    }
    if source.ends_with(".gov")
        || ["sec.gov", "cftc.gov", "federalreserve.gov"]
            .iter()
            .any(|domain| matches_domain(&source, domain))
    {
        return (
            "Government or regulator primary source".to_string(),
            96,
            true,
        );
    }
    if [
        "ethereum.org",
        "solana.com",
        "bnbchain.org",
        "bitcoin.org",
        "coinbase.com",
        "binance.com",
        "kraken.com",
    ]
    .iter()
    .any(|domain| matches_domain(&source, domain))
    {
        return ("Protocol or company primary source".to_string(), 84, true);
    }
    if ["reuters.com", "apnews.com", "bloomberg.com"]
        .iter()
        .any(|domain| matches_domain(&source, domain))
    {
        return ("Major news wire".to_string(), 90, false);
    }
    if ["ft.com", "wsj.com", "cnbc.com"]
        .iter()
        .any(|domain| matches_domain(&source, domain))
    {
        return ("Established financial press".to_string(), 84, false);
    }
    if [
        "coindesk.com",
        "theblock.co",
        "decrypt.co",
        "blockworks.co",
        "cointelegraph.com",
    ]
    .iter()
    .any(|domain| matches_domain(&source, domain))
    {
        return ("Established crypto press".to_string(), 76, false);
    }
    if source.contains("medium.com") || source.contains("substack.com") || source.contains("blog") {
        return ("Self-published commentary".to_string(), 48, false);
    }
    if source.is_empty() {
        return ("Unknown source".to_string(), 35, false);
    }
    ("Other publication".to_string(), 58, false)
}

fn matches_domain(source: &str, domain: &str) -> bool {
    source == domain
        || source
            .strip_suffix(domain)
            .is_some_and(|prefix| prefix.ends_with('.'))
}

fn completeness_score(story: &NewsStory) -> u8 {
    let mut score = 35;
    if story.published_at.is_some() {
        score += 25;
    }
    if story.content.chars().count() >= 80 {
        score += 25;
    }
    if story.url.starts_with("https://") {
        score += 15;
    }
    score
}

fn claims_overlap(left: &NewsStory, right: &NewsStory) -> bool {
    let left_text = format!("{} {}", left.headline, left.content);
    let right_text = format!("{} {}", right.headline, right.content);
    let left_cluster = narrative_cluster(&left_text.to_lowercase());
    let right_cluster = narrative_cluster(&right_text.to_lowercase());
    if left_cluster != right_cluster {
        return false;
    }
    let left_terms = claim_terms(&left.headline);
    let right_terms = claim_terms(&right.headline);
    let overlap = left_terms.intersection(&right_terms).count();
    let union = left_terms.union(&right_terms).count().max(1);
    overlap >= 2 && overlap as f32 / union as f32 >= 0.2
}

fn claim_terms(value: &str) -> BTreeSet<String> {
    const STOPWORDS: &[&str] = &[
        "a", "an", "and", "as", "at", "after", "before", "by", "for", "from", "in", "is", "it",
        "its", "market", "new", "of", "on", "the", "to", "with", "crypto", "token", "today",
    ];
    value
        .to_lowercase()
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|word| word.len() >= 3 && !STOPWORDS.contains(word))
        .map(str::to_string)
        .collect()
}

fn claims_conflict(left: &str, left_sentiment: &str, right: &str) -> bool {
    let denial = [
        "denies",
        "denied",
        "false",
        "debunk",
        "no evidence",
        "not true",
    ];
    let one_denies = denial.iter().any(|term| left.contains(term))
        ^ denial.iter().any(|term| right.contains(term));
    let right_sentiment = sentiment(right);
    one_denies
        || matches!(
            (left_sentiment, right_sentiment.as_str()),
            ("bullish", "bearish") | ("bearish", "bullish")
        )
}

fn credibility_label(score: u8) -> String {
    match score {
        80..=100 => "High".to_string(),
        65..=79 => "Medium".to_string(),
        _ => "Low".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn story(id: &str, source: &str, headline: &str) -> NewsStory {
        NewsStory {
            id: id.to_string(),
            headline: headline.to_string(),
            source: source.to_string(),
            url: format!("https://{source}/{id}"),
            content: format!("Detailed reporting about {headline} with additional context."),
            published_at: Some("2026-08-20T00:00:00Z".to_string()),
            region: Some("Global".to_string()),
            language: Some("en".to_string()),
            data_mode: "live".to_string(),
            search_relevance: Some(90),
        }
    }

    #[test]
    fn independent_domains_corroborate_a_similar_claim() {
        let stories = vec![
            story("1", "reuters.com", "Bitcoin ETF approval lifts demand"),
            story("2", "sec.gov", "Bitcoin ETF approval receives final order"),
        ];
        let result = verify_story(&stories[0], &stories);
        assert_eq!(result.claim_status, "corroborated");
        assert_eq!(result.independent_source_count, 2);
        assert!(result.credibility_score >= 75);
    }

    #[test]
    fn user_claim_is_never_presented_as_verified() {
        let mut item = story("1", "user-input", "SOL partnership announced");
        item.data_mode = "user-provided".to_string();
        let result = verify_story(&item, std::slice::from_ref(&item));
        assert_eq!(result.claim_status, "user claim");
        assert!(result.credibility_score <= 35);
    }

    #[test]
    fn lookalike_domains_do_not_inherit_publisher_authority() {
        let item = story("1", "fake-reuters.com", "Bitcoin ETF approval reported");
        let result = verify_story(&item, std::slice::from_ref(&item));
        assert_eq!(result.source_class, "Other publication");
    }
}
