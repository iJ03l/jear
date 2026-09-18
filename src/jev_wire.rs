//! Jev SystemOne JSON wire types.
//!
//! `serde` shapes for `POST /v1/systemone` — no HTTP calls yet.
//! Matches TypeSafe docs: `state` + parallel typed `questions` in,
//! typed answers with probabilities + confidence out.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Default model alias (server resolves to latest `jev-1.x`).
pub const JEV_DEFAULT_MODEL: &str = "jev-latest";

/// SystemOne evaluate path.
pub const JEV_SYSTEMONE_PATH: &str = "/v1/systemone";

/// One typed question in a SystemOne request.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WireQuestion {
    /// Pick one option from `criteria` (option -> description).
    Choice {
        /// What to decide.
        instructions: String,
        /// Option name -> description (max 255 for Jev).
        criteria: BTreeMap<String, String>,
    },
    /// Rate on an ordered rubric (level -> description).
    Score {
        /// What to rate.
        instructions: String,
        /// Level name -> description.
        criteria: BTreeMap<String, String>,
    },
    /// Yes/no probability.
    Noul {
        /// Statement to test.
        instructions: String,
    },
}

/// SystemOne request body.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WireRequest {
    /// Model alias, e.g. `jev-latest`.
    pub model: String,
    /// Context — text or structured JSON (kept as `Value`).
    pub state: serde_json::Value,
    /// Question name -> typed question (all evaluated in parallel).
    pub questions: BTreeMap<String, WireQuestion>,
}

impl WireRequest {
    /// Build a request with text state.
    #[must_use]
    pub fn new(
        model: impl Into<String>,
        state_text: impl Into<String>,
        questions: BTreeMap<String, WireQuestion>,
    ) -> Self {
        Self {
            model: model.into(),
            state: serde_json::Value::String(state_text.into()),
            questions,
        }
    }
}

/// Typed answer in a SystemOne response.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WireAnswer {
    /// Choice result with per-option probabilities + confidence.
    Choice {
        /// Winning option.
        choice: String,
        /// Option -> probability.
        probabilities: BTreeMap<String, f64>,
        /// Calibrated confidence.
        confidence: f64,
    },
    /// Score result.
    Score {
        /// Continuous score.
        score: f64,
        /// Calibrated confidence.
        confidence: f64,
    },
    /// Yes/no result.
    Noul {
        /// Probability yes.
        noul: f64,
    },
}

/// SystemOne response body (subset we route on).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct WireResponse {
    /// Resolved versioned model, e.g. `jev-1.13.0`.
    pub model: String,
    /// Question name -> typed answer.
    pub answers: BTreeMap<String, WireAnswer>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn choice_request() -> WireRequest {
        let mut criteria = BTreeMap::new();
        criteria.insert("billing".to_string(), "Payments, refunds".to_string());
        criteria.insert("technical".to_string(), "Bugs, outages".to_string());
        let mut questions = BTreeMap::new();
        questions.insert(
            "department".to_string(),
            WireQuestion::Choice {
                instructions: "Which team should handle this?".to_string(),
                criteria,
            },
        );
        WireRequest::new("jev-latest", "Payouts failing for 3 days", questions)
    }

    #[test]
    fn request_serializes_with_type_tags() {
        let v = serde_json::to_value(choice_request()).expect("serialize");
        assert_eq!(v["model"], "jev-latest");
        assert_eq!(v["questions"]["department"]["type"], "choice");
        assert!(v["questions"]["department"]["criteria"]["technical"].is_string());
    }

    #[test]
    fn response_deserializes_choice() {
        let raw = serde_json::json!({
            "model": "jev-1.13.0",
            "answers": {
                "department": {
                    "type": "choice",
                    "choice": "technical",
                    "probabilities": {"billing": 0.08, "technical": 0.85, "sales": 0.07},
                    "confidence": 0.82
                }
            }
        });
        let r: WireResponse = serde_json::from_value(raw).expect("deserialize");
        assert_eq!(r.model, "jev-1.13.0");
        match r.answers.get("department").expect("answer") {
            WireAnswer::Choice {
                choice,
                probabilities,
                confidence,
            } => {
                assert_eq!(choice, "technical");
                assert!((probabilities["technical"] - 0.85).abs() < 1e-9);
                assert!((*confidence - 0.82).abs() < 1e-9);
            }
            other => panic!("unexpected answer: {other:?}"),
        }
    }

    #[test]
    fn constants_match_docs() {
        assert_eq!(JEV_DEFAULT_MODEL, "jev-latest");
        assert_eq!(JEV_SYSTEMONE_PATH, "/v1/systemone");
    }
}
