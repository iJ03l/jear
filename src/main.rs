use jear::budget::MonthlyBudget;
use jear::cli::parse;
use jear::ironclaw::{CostCaps, DeployTarget};
use jear::jev::State;
use jear::jev_wire::{WireQuestion, WireRequest, JEV_DEFAULT_MODEL};
use jear::live::{
    complete, env_key, evaluate, list_models, near_base, pick_best, NEAR_ENV_KEY,
    TYPESAFE_BASE_ENV, TYPESAFE_ENV_KEY,
};
use jear::near::{Hosting, ModelInfo};
use jear::near_wire::ChatRequest;
use jear::route::{
    answers_from_wire, eligible_models, estimate_plan, route_with_monthly, within_caps, Answers,
};
use std::collections::BTreeMap;

fn main() {
    println!("{}", jear::hello());

    // CLI-driven demo: brief + client monthly cap, offline by default.
    let cfg = parse(&std::env::args().collect::<Vec<_>>());
    let state = State::new(cfg.brief.clone(), 50);
    let answers = live_answers(&cfg.brief).unwrap_or_else(|| Answers::new(0.3, 0.2, 0.1, 0.9));
    let monthly = MonthlyBudget::new(cfg.monthly_cents).expect("valid monthly cap");
    let decision = route_with_monthly(&state, &answers, 5, 0.6, &monthly);

    let models = vec![
        ModelInfo::new("zai-org/GLM-5.1-FP8", Hosting::Tee, true),
        ModelInfo::new("openai/gpt-5", Hosting::Proxied, true),
    ];
    let eligible = eligible_models(&models, &decision);
    let caps = CostCaps::new(500, 100).expect("valid caps");
    let target = DeployTarget::Hub("demo-instance".to_string());

    println!("brief: {}", cfg.brief);
    println!("monthly: {}c remaining", monthly.remaining());
    println!("route: {:?} (tee: {})", decision.route, decision.force_tee);
    println!("reason: {}", decision.reason);
    println!("eligible models: {}", eligible.len());
    println!("caps ok for 5c: {}", within_caps(&caps, 5));
    println!("target: {target:?}");

    // Live mode: complete via NEAR when --live and key present, else guidance.
    if !cfg.live {
        return;
    }
    let Some(key) = env_key(NEAR_ENV_KEY) else {
        println!("live requested but {NEAR_ENV_KEY} is missing — staying offline");
        return;
    };
    // Live catalog first: best economical model for the task-sized plan wins.
    // Falls back to the routed demo pair when the catalog is unreachable.
    let base = near_base();
    let (plan_in, plan_out) = estimate_plan(answers.complexity);
    let model = match list_models(&base, &key) {
        Ok(entries) => match pick_best(&entries, plan_in, plan_out) {
            Some(entry) => {
                println!("catalog: {} entries, best {}", entries.len(), entry.id);
                entry.id.clone()
            }
            None => {
                println!(
                    "catalog: {} entries, none priced — using routed model",
                    entries.len()
                );
                routed_model(&eligible)
            }
        },
        Err(e) => {
            println!("catalog error: {e} — using routed model");
            routed_model(&eligible)
        }
    };
    match complete(&base, &key, &ChatRequest::new(model, cfg.brief)) {
        Ok(resp) => {
            println!("answer: {}", resp.first_text().unwrap_or("(empty)"));
            if let Some(u) = resp.usage {
                println!("usage: {} total tokens", u.total_tokens);
            }
        }
        Err(e) => println!("live error: {e}"),
    }
}

/// First eligible demo model id, or the documented TEE default.
fn routed_model(eligible: &[&ModelInfo]) -> String {
    eligible
        .first()
        .map(|m| m.id.clone())
        .unwrap_or_else(|| "zai-org/GLM-5.1-FP8".to_string())
}

/// Ask live Jev for routing answers when `--live` keys exist.
/// Returns `None` offline (or on any Jev failure) so callers fall back.
fn live_answers(brief: &str) -> Option<Answers> {
    let base = env_key(TYPESAFE_BASE_ENV)?;
    let key = env_key(TYPESAFE_ENV_KEY)?;
    let mut questions = BTreeMap::new();
    let mut complexity = BTreeMap::new();
    complexity.insert("0".to_string(), "lookup".to_string());
    complexity.insert("1".to_string(), "multi-step".to_string());
    complexity.insert("2".to_string(), "high-stakes".to_string());
    questions.insert(
        "complexity".to_string(),
        WireQuestion::Score {
            instructions: "Task complexity".to_string(),
            criteria: complexity,
        },
    );
    let mut sensitivity = BTreeMap::new();
    sensitivity.insert("0".to_string(), "public".to_string());
    sensitivity.insert("2".to_string(), "secret".to_string());
    questions.insert(
        "sensitivity".to_string(),
        WireQuestion::Score {
            instructions: "Data sensitivity".to_string(),
            criteria: sensitivity,
        },
    );
    questions.insert(
        "needs_private".to_string(),
        WireQuestion::Noul {
            instructions: "Must run in TEE private inference?".to_string(),
        },
    );
    let req = WireRequest::new(JEV_DEFAULT_MODEL, brief, questions);
    match evaluate(&base, &key, &req) {
        Ok(resp) => {
            println!("jev: {} ({})", resp.model, resp.answers.len());
            Some(answers_from_wire(&resp))
        }
        Err(e) => {
            println!("jev error: {e} — using offline answers");
            None
        }
    }
}
