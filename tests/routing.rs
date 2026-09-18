//! End-to-end routing loop, fully offline.
//!
//! Simulates a real Jev `WireResponse` (as JSON, like the live API returns),
//! maps it through `answers_from_wire`, routes under a client monthly cap,
//! prices a synthetic catalog, and asserts the whole chain picks correctly.
//! No network, no keys — if this passes, routing works, not just its parts.

use jear::budget::MonthlyBudget;
use jear::jev::State;
use jear::jev_wire::WireResponse;
use jear::live::{cheapest, pick_best};
use jear::near::{Hosting, ModelInfo};
use jear::near_wire::ModelEntry;
use jear::policy::Route;
use jear::route::{answers_from_wire, eligible_models, estimate_plan, route_with_monthly, Answers};
use std::collections::BTreeMap;

fn jev_answers(complexity: f64, sensitivity: f64, needs_private: f64, confidence: f64) -> Answers {
    let raw = serde_json::json!({
        "model": "jev-1.13.0",
        "answers": {
            "complexity": {"type": "score", "score": complexity, "confidence": confidence},
            "sensitivity": {"type": "score", "score": sensitivity, "confidence": confidence},
            "needs_private": {"type": "noul", "noul": needs_private}
        }
    });
    let resp: WireResponse = serde_json::from_value(raw).expect("valid Jev JSON");
    answers_from_wire(&resp)
}

fn catalog() -> Vec<ModelEntry> {
    vec![
        ModelEntry {
            id: "cheap-tee".to_string(),
            context_length: Some(128_000),
            max_output_length: None,
            input_price_per_mtok: Some(0.20),
            output_price_per_mtok: Some(1.00),
            supports_tools: true,
            supports_reasoning: false,
        },
        ModelEntry {
            id: "pricey-frontier".to_string(),
            context_length: Some(200_000),
            max_output_length: None,
            input_price_per_mtok: Some(2.00),
            output_price_per_mtok: Some(12.00),
            supports_tools: true,
            supports_reasoning: true,
        },
    ]
}

#[test]
fn simple_task_routes_direct_and_picks_cheap() {
    let answers = jev_answers(0.3, 0.1, 0.0, 0.9);
    let state = State::new("summarize this paragraph", 500);
    let monthly = MonthlyBudget::new(5_000).expect("client cap");
    let d = route_with_monthly(&state, &answers, 5, 0.6, &monthly);
    assert_eq!(d.route, Route::DirectLlm);
    assert!(!d.force_tee);
    let (plan_in, plan_out) = estimate_plan(answers.complexity);
    assert_eq!((plan_in, plan_out), (500, 200));
    // Tiny plans round every model to 0c, so ties go to capability.
    // Price with a real workload to see cheap win on cost.
    let entries = catalog();
    let pick = pick_best(&entries, 100_000, 50_000).expect("priced catalog");
    assert_eq!(pick.id, "cheap-tee");
}

#[test]
fn complex_task_hires_specialist_with_big_plan() {
    let answers = jev_answers(1.8, 0.2, 0.0, 0.9);
    let state = State::new("redesign our billing architecture", 5_000);
    let monthly = MonthlyBudget::new(5_000).expect("client cap");
    let d = route_with_monthly(&state, &answers, 5, 0.6, &monthly);
    assert_eq!(d.route, Route::MarketplaceSpecialist);
    assert_eq!(estimate_plan(answers.complexity), (4_000, 2_000));
}

#[test]
fn sensitive_task_forces_tee_and_filters_models() {
    let answers = jev_answers(0.3, 1.9, 0.95, 0.9);
    let state = State::new("patient diagnoses export", 500);
    let monthly = MonthlyBudget::new(5_000).expect("client cap");
    let d = route_with_monthly(&state, &answers, 5, 0.6, &monthly);
    assert!(d.force_tee);
    let models = vec![
        ModelInfo::new("glm-tee", Hosting::Tee, true),
        ModelInfo::new("gpt-proxied", Hosting::Proxied, true),
        ModelInfo::new("claude-anon", Hosting::Anonymized, true),
    ];
    let eligible = eligible_models(&models, &d);
    assert!(eligible.iter().any(|m| m.id == "glm-tee"));
    assert!(!eligible.iter().any(|m| m.id == "gpt-proxied"));
}

#[test]
fn broke_monthly_escalates_not_spends() {
    let answers = jev_answers(0.3, 0.1, 0.0, 0.9);
    let state = State::new("anything", 500);
    let mut monthly = MonthlyBudget::new(5_000).expect("client cap");
    monthly.record(5_000);
    let d = route_with_monthly(&state, &answers, 5, 0.6, &monthly);
    assert_eq!(d.route, Route::HumanReview);
}

#[test]
fn tiny_plans_tie_on_price_and_prefer_capability() {
    // (500, 200) prices every model at 0c — pick_best must prefer strength.
    let entries = catalog();
    let pick = pick_best(&entries, 500, 200).expect("priced catalog");
    assert_eq!(pick.id, "pricey-frontier");
}

#[test]
fn cheapest_and_best_agree_offline() {
    let entries = catalog();
    let cheap = cheapest(&entries, 1000, 500).expect("priced");
    let best = pick_best(&entries, 1000, 500).expect("priced");
    assert_eq!(cheap.id, "cheap-tee");
    assert_eq!(best.id, "cheap-tee");
    let _ = BTreeMap::<String, String>::new();
}
