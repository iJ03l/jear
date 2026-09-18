//! Jev System One router types.
//!
//! Pure types only — no network calls yet. Matches TypeSafe AI semantics:
//! `Choice` picks one option, `Score` rates on a rubric, `Noul` is a yes/no
//! probability. Every answer carries calibrated `confidence` for gating.

use std::collections::BTreeMap;

/// Input context evaluated by Jev. Keep it small and explicit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    /// User brief or prompt text (redacted, no secrets).
    pub text: String,
    /// Remaining budget in cents for this user/task.
    pub budget_cents: u64,
}

impl State {
    /// Build a new routing state.
    #[must_use]
    pub fn new(text: impl Into<String>, budget_cents: u64) -> Self {
        Self {
            text: text.into(),
            budget_cents,
        }
    }
}

/// A `Choice` question: pick one handler from a fixed set (max 255 for Jev).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceQuestion {
    /// What to decide, e.g. "Which handler can complete this cheapest?".
    pub instructions: String,
    /// Option name -> description, e.g. "direct_llm" -> "lookup, summary".
    pub criteria: BTreeMap<String, String>,
}

/// Answer to a `Choice` question.
#[derive(Debug, Clone, PartialEq)]
pub struct ChoiceAnswer {
    /// Winning option name.
    pub choice: String,
    /// Probability per option (sums to ~1.0).
    pub probabilities: BTreeMap<String, f64>,
    /// Calibrated confidence 0.0..=1.0 — whether to act.
    pub confidence: f64,
}

/// A `Score` question: rate state on an ordered rubric.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreQuestion {
    /// What to rate, e.g. "Data sensitivity".
    pub instructions: String,
    /// Level index -> description, e.g. 0 -> "public", 2 -> "secret".
    pub levels: BTreeMap<u8, String>,
}

/// Answer to a `Score` question.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoreAnswer {
    /// Continuous score, e.g. 1.4 on a 0..=2 rubric.
    pub score: f64,
    /// Calibrated confidence 0.0..=1.0.
    pub confidence: f64,
}

/// A `Noul` question: probability the statement is true.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoulQuestion {
    /// Statement to test, e.g. "Must run in TEE private inference?".
    pub instructions: String,
}

/// Answer to a `Noul` question: probability yes (0.0..=1.0).
#[derive(Debug, Clone, PartialEq)]
pub struct NoulAnswer {
    /// Probability the statement is true.
    pub noul: f64,
}

/// Confidence-gated routing: escalate when confidence is below threshold.
///
/// The answer tells you *what*; confidence tells you *whether to act*.
#[must_use]
pub fn should_escalate(confidence: f64, threshold: f64) -> bool {
    confidence < threshold
}

/// Clamp a probability/score helper into 0.0..=1.0 for safe comparisons.
#[must_use]
pub fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_holds_text_and_budget() {
        let s = State::new("summarize emails", 50);
        assert_eq!(s.text, "summarize emails");
        assert_eq!(s.budget_cents, 50);
    }

    #[test]
    fn escalate_when_low_confidence() {
        assert!(should_escalate(0.4, 0.6));
        assert!(!should_escalate(0.82, 0.6));
    }

    #[test]
    fn clamp_keeps_unit_range() {
        assert_eq!(clamp01(-0.5), 0.0);
        assert_eq!(clamp01(1.5), 1.0);
        assert_eq!(clamp01(0.85), 0.85);
    }
}
