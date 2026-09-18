//! Orchestrator: Jev answers -> policy -> execution targets.
//!
//! Pure glue only — no network calls. Ties `jev`, `policy`, `near`,
//! `budget`, and `ironclaw` together so callers make one call to route.

use crate::budget::MonthlyBudget;
use crate::ironclaw::CostCaps;
use crate::jev::{clamp01, State};
use crate::jev_wire::{WireAnswer, WireResponse};
use crate::near::ModelInfo;
use crate::policy::{decide, Decision, PolicyInput, Route};

/// Distilled Jev outputs for one request (all 0.0..=1.0 except scores).
#[derive(Debug, Clone, PartialEq)]
pub struct Answers {
    /// Task complexity 0.0 (lookup) .. 2.0 (high-stakes).
    pub complexity: f64,
    /// Data sensitivity 0.0 (public) .. 2.0 (secret/regulated).
    pub sensitivity: f64,
    /// `needs_private` probability 0.0..=1.0.
    pub needs_private: f64,
    /// Routing confidence 0.0..=1.0.
    pub confidence: f64,
}

impl Answers {
    /// Build answers, clamping probabilities into range.
    #[must_use]
    pub fn new(complexity: f64, sensitivity: f64, needs_private: f64, confidence: f64) -> Self {
        Self {
            complexity,
            sensitivity,
            needs_private: clamp01(needs_private),
            confidence: clamp01(confidence),
        }
    }
}

/// Route one request: build policy input from state + answers, then decide.
#[must_use]
pub fn route(
    state: &State,
    answers: &Answers,
    estimated_cents: u64,
    confidence_floor: f64,
) -> Decision {
    let input = PolicyInput {
        complexity: answers.complexity,
        sensitivity: answers.sensitivity,
        needs_private: answers.needs_private,
        confidence: answers.confidence,
        budget_cents: state.budget_cents,
        estimated_cents,
    };
    decide(&input, confidence_floor)
}

/// Filter a model catalog to what a decision allows.
/// When `force_tee` is set, only TEE-hosted models remain.
#[must_use]
pub fn eligible_models<'a>(models: &'a [ModelInfo], decision: &Decision) -> Vec<&'a ModelInfo> {
    let minimum = if decision.force_tee { 2 } else { 0 };
    eligible_models_for(models, minimum)
}

/// Filter a model catalog by minimum privacy rank
/// (2 = TEE-only, 1 = TEE or anonymized gateway, 0 = any).
#[must_use]
pub fn eligible_models_for(models: &[ModelInfo], minimum: u8) -> Vec<&ModelInfo> {
    models.iter().filter(|m| m.meets_privacy(minimum)).collect()
}

/// Check IronClaw spend caps for an estimate.
#[must_use]
pub fn within_caps(caps: &CostCaps, estimated_cents: u64) -> bool {
    caps.allows(estimated_cents)
}

/// Build routing `Answers` from a live Jev `WireResponse`.
/// Looks up `complexity`/`sensitivity` scores and the `needs_private`
/// noul; confidence is the minimum of any Choice/Score confidences
/// (0.5 when none present). Missing values fall back to safe defaults.
#[must_use]
pub fn answers_from_wire(resp: &WireResponse) -> Answers {
    let complexity = match resp.answers.get("complexity") {
        Some(WireAnswer::Score { score, .. }) => *score,
        _ => 0.5,
    };
    let sensitivity = match resp.answers.get("sensitivity") {
        Some(WireAnswer::Score { score, .. }) => *score,
        _ => 0.0,
    };
    let needs_private = match resp.answers.get("needs_private") {
        Some(WireAnswer::Noul { noul }) => clamp01(*noul),
        _ => 0.0,
    };
    let mut confs: Vec<f64> = Vec::new();
    for answer in resp.answers.values() {
        match answer {
            WireAnswer::Choice { confidence, .. } => confs.push(*confidence),
            WireAnswer::Score { confidence, .. } => confs.push(*confidence),
            WireAnswer::Noul { .. } => {}
        }
    }
    let confidence = confs.into_iter().fold(f64::INFINITY, f64::min);
    let confidence = if confidence.is_finite() {
        clamp01(confidence)
    } else {
        0.5
    };
    Answers::new(complexity, sensitivity, needs_private, confidence)
}

/// Route with a monthly cap: when the estimate exceeds this month's
/// remainder, escalate for approval instead of picking an agent.
/// The cap is client-controlled; this picks the best *economical* route under it.
#[must_use]
pub fn route_with_monthly(
    state: &State,
    answers: &Answers,
    estimated_cents: u64,
    confidence_floor: f64,
    monthly: &MonthlyBudget,
) -> Decision {
    if !monthly.can_afford(estimated_cents) {
        let tee = answers.needs_private >= 0.8 || answers.sensitivity >= 1.2;
        return Decision {
            route: Route::HumanReview,
            force_tee: tee,
            reason: "over monthly budget — needs approval".to_string(),
        };
    }
    route(state, answers, estimated_cents, confidence_floor)
}

/// Estimate a token plan from complexity so output price shapes the pick.
/// Bands mirror `policy::decide`: lookup, multi-step, high-stakes.
#[must_use]
pub fn estimate_plan(complexity: f64) -> (u64, u64) {
    if complexity < 0.7 {
        (500, 200)
    } else if complexity < 1.4 {
        (2_000, 800)
    } else {
        (4_000, 2_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ironclaw::CostCaps;
    use crate::near::{Hosting, ModelInfo};

    fn state() -> State {
        State::new("summarize emails", 50)
    }

    fn answers() -> Answers {
        Answers::new(0.3, 0.2, 0.1, 0.9)
    }

    #[test]
    fn simple_routes_direct() {
        let d = route(&state(), &answers(), 5, 0.6);
        assert_eq!(d.route, crate::policy::Route::DirectLlm);
    }

    #[test]
    fn sensitive_filters_to_private_only() {
        let a = Answers::new(0.3, 1.8, 0.9, 0.9);
        let d = route(&state(), &a, 5, 0.6);
        assert!(d.force_tee);
        let models = vec![
            ModelInfo::new("zai-org/GLM-5.1-FP8", Hosting::Tee, true),
            ModelInfo::new("openai/gpt-5", Hosting::Proxied, true),
        ];
        let eligible = eligible_models(&models, &d);
        assert_eq!(eligible.len(), 1);
        assert_eq!(eligible[0].id, "zai-org/GLM-5.1-FP8");
    }

    #[test]
    fn medium_sensitivity_keeps_anonymized() {
        use crate::near::{Hosting, ModelInfo};
        let models = vec![
            ModelInfo::new("glm-tee", Hosting::Tee, true),
            ModelInfo::new("claude-anon", Hosting::Anonymized, true),
            ModelInfo::new("gpt-proxied", Hosting::Proxied, true),
        ];
        let tier1: Vec<&str> = eligible_models_for(&models, 1)
            .iter()
            .map(|m| m.id.as_str())
            .collect();
        assert_eq!(tier1, vec!["glm-tee", "claude-anon"]);
        assert_eq!(eligible_models_for(&models, 2).len(), 1);
        assert_eq!(eligible_models_for(&models, 0).len(), 3);
    }

    #[test]
    fn caps_gate_expensive_tasks() {
        let caps = CostCaps::new(500, 100).expect("valid caps");
        assert!(within_caps(&caps, 100));
        assert!(!within_caps(&caps, 101));
    }

    #[test]
    fn monthly_over_spend_escalates() {
        use crate::budget::MonthlyBudget;
        let mut monthly = MonthlyBudget::new(7_500).expect("valid budget");
        monthly.record(7_499);
        let d = route_with_monthly(&state(), &answers(), 5, 0.6, &monthly);
        assert_eq!(d.route, crate::policy::Route::HumanReview);
        let fresh = MonthlyBudget::new(7_500).expect("valid budget");
        let ok = route_with_monthly(&state(), &answers(), 5, 0.6, &fresh);
        assert_eq!(ok.route, crate::policy::Route::DirectLlm);
    }

    #[test]
    fn wire_response_maps_to_answers() {
        use crate::jev_wire::{WireAnswer, WireResponse};
        use std::collections::BTreeMap;
        let mut answers = BTreeMap::new();
        answers.insert(
            "complexity".to_string(),
            WireAnswer::Score {
                score: 1.8,
                confidence: 0.9,
            },
        );
        answers.insert(
            "sensitivity".to_string(),
            WireAnswer::Score {
                score: 0.2,
                confidence: 0.8,
            },
        );
        answers.insert("needs_private".to_string(), WireAnswer::Noul { noul: 0.1 });
        let resp = WireResponse {
            model: "jev-1.13.0".to_string(),
            answers,
        };
        let a = answers_from_wire(&resp);
        assert!((a.complexity - 1.8).abs() < 1e-9);
        assert!((a.sensitivity - 0.2).abs() < 1e-9);
        assert!((a.needs_private - 0.1).abs() < 1e-9);
        assert!((a.confidence - 0.8).abs() < 1e-9);
    }

    #[test]
    fn wire_response_defaults_when_empty() {
        use crate::jev_wire::WireResponse;
        use std::collections::BTreeMap;
        let resp = WireResponse {
            model: "jev-1.13.0".to_string(),
            answers: BTreeMap::new(),
        };
        let a = answers_from_wire(&resp);
        assert!((a.confidence - 0.5).abs() < 1e-9);
    }

    #[test]
    fn plans_grow_with_complexity() {
        assert_eq!(estimate_plan(0.3), (500, 200));
        assert_eq!(estimate_plan(1.0), (2_000, 800));
        assert_eq!(estimate_plan(1.8), (4_000, 2_000));
    }
}
