//! Receipt-grounded research assistant with an honest deterministic fallback.

use crate::domain::{ChatRequest, ChatResponse, DecisionReceipt};
use crate::services::openai::openai_chat;
use crate::state::AppState;

/// Answer with OpenAI when configured, preserving a useful local answer when
/// the model is unavailable or the project is running in demo mode.
pub(crate) async fn answer_chat(
    state: &AppState,
    request: &ChatRequest,
    receipt: Option<&DecisionReceipt>,
) -> ChatResponse {
    let mut fallback = deterministic_answer(&request.question, receipt);
    let Some(openai_key) = state.config.openai_api_key.as_deref() else {
        return fallback;
    };

    match openai_chat(
        state,
        openai_key,
        &request.question,
        request.history.as_slice(),
        receipt,
    )
    .await
    {
        Ok((answer, suggestions)) => ChatResponse {
            answer,
            suggestions,
            generated_by: format!("openai:{}", state.config.openai_model),
            receipt_id: receipt.map(|item| item.id.clone()),
            warnings: Vec::new(),
        },
        Err(error) => {
            fallback.warnings.push(format!(
                "OpenAI assistant unavailable: {}. Receipt-based fallback used.",
                error.message
            ));
            fallback
        }
    }
}

fn deterministic_answer(question: &str, receipt: Option<&DecisionReceipt>) -> ChatResponse {
    let Some(receipt) = receipt else {
        return ChatResponse {
            answer: "I can explain the workspace now. Run an analysis or load the sample, and I will summarize that receipt using its news, RYO evidence, risks, and next action.".to_string(),
            suggestions: vec![
                "How does the analysis work?".to_string(),
                "What is a decision receipt?".to_string(),
                "Which data can be unavailable?".to_string(),
            ],
            generated_by: "deterministic:product-guide".to_string(),
            receipt_id: None,
            warnings: Vec::new(),
        };
    };

    let lower = question.to_lowercase();
    let highest_card = receipt
        .sections
        .iter()
        .flat_map(|section| section.cards.iter())
        .max_by_key(|card| card.score.impact);
    let answer = if lower.contains("risk") || lower.contains("invalid") {
        format!(
            "For {}, the main invalidation is: {} The paper-plan stance is {} with a {}% risk budget. This remains research, not an executable trade.",
            receipt.symbol,
            receipt.invalidation,
            receipt.practice_plan.stance,
            receipt.practice_plan.risk_budget_pct
        )
    } else if lower.contains("news") || lower.contains("headline") || lower.contains("matter") {
        highest_card.map_or_else(
            || format!("No ranked headline is available in this {} receipt.", receipt.symbol),
            |card| {
                format!(
                    "The highest-impact headline is '{}' at {}/100 impact. It maps to {}, and the receipt recommends: {}.",
                    card.headline,
                    card.score.impact,
                    card.narrative_cluster,
                    card.recommendation
                )
            },
        )
    } else if lower.contains("next") || lower.contains("monitor") || lower.contains("suggest") {
        format!(
            "The receipt's next action is: {} A useful follow-up is to watch whether '{}' changes and whether missing market evidence becomes available.",
            receipt.next_action, receipt.invalidation
        )
    } else {
        let top_line = highest_card
            .map(|card| {
                format!(
                    " The leading story is '{}' at {}/100 impact.",
                    card.headline, card.score.impact
                )
            })
            .unwrap_or_default();
        format!(
            "{} is {} at {}% confidence. {}{} Next: {}",
            receipt.symbol,
            receipt.verdict.decision,
            receipt.verdict.confidence,
            receipt.summary.conclusion,
            top_line,
            receipt.next_action
        )
    };

    ChatResponse {
        answer,
        suggestions: vec![
            "What could invalidate this view?".to_string(),
            "Which headline matters most?".to_string(),
            "What should I monitor next?".to_string(),
        ],
        generated_by: "deterministic:receipt-guide".to_string(),
        receipt_id: Some(receipt.id.clone()),
        warnings: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::deterministic_answer;

    #[test]
    fn gives_product_help_without_a_receipt() {
        let response = deterministic_answer("How does this work?", None);

        assert!(response.answer.contains("Run an analysis"));
        assert_eq!(response.suggestions.len(), 3);
        assert!(response.receipt_id.is_none());
    }
}
