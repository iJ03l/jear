//! Live API callers — need env keys + network.
//!
//! Keys come from the environment (`NEAR_API_KEY`, `TYPESAFE_API_KEY`,
//! plus optional base-URL overrides). Bases are caller-provided so no
//! endpoint is hardcoded except NEAR's documented default. Tests stay offline.

use crate::attest::{AttestationReport, ATTESTATION_PATH};
use crate::http::{get_json, join, post_json, HttpError};
use crate::jev_wire::{WireRequest, WireResponse, JEV_SYSTEMONE_PATH};
use crate::near::{CHAT_COMPLETIONS_PATH, CLOUD_BASE_URL, MODELS_PATH};
use crate::near_wire::{ChatRequest, ChatResponse, ModelEntry, ModelsResponse};

/// Env var holding the NEAR AI Cloud key (`sk-...`).
pub const NEAR_ENV_KEY: &str = "NEAR_API_KEY";

/// Env var holding the TypeSafe (Jev) key.
pub const TYPESAFE_ENV_KEY: &str = "TYPESAFE_API_KEY";

/// Optional env override for the NEAR base URL.
pub const NEAR_BASE_ENV: &str = "NEAR_BASE_URL";

/// Optional env override for the TypeSafe base URL (no default hardcoded).
pub const TYPESAFE_BASE_ENV: &str = "TYPESAFE_BASE_URL";

/// Read an env key; empty or missing yields `None` (never logs the value).
#[must_use]
pub fn env_key(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

/// NEAR base URL: env override wins, else the documented default.
#[must_use]
pub fn near_base() -> String {
    env_key(NEAR_BASE_ENV).unwrap_or_else(|| CLOUD_BASE_URL.to_string())
}

/// Full Jev evaluate URL from an explicit base.
#[must_use]
pub fn jev_url(base: &str) -> String {
    join(base, JEV_SYSTEMONE_PATH)
}

/// Full NEAR models URL from a base.
#[must_use]
pub fn models_url(base: &str) -> String {
    join(base, MODELS_PATH)
}

/// Full NEAR chat URL from a base.
#[must_use]
pub fn near_url(base: &str) -> String {
    join(base, CHAT_COMPLETIONS_PATH)
}

/// Evaluate Jev questions against state (live network).
pub fn evaluate(
    base: &str,
    api_key: &str,
    request: &WireRequest,
) -> Result<WireResponse, HttpError> {
    let body = serde_json::to_value(request).map_err(|e| HttpError::Json(e.to_string()))?;
    let raw = post_json(&jev_url(base), api_key, &body)?;
    serde_json::from_value(raw).map_err(|e| HttpError::Json(e.to_string()))
}

/// Run a NEAR chat completion (live network).
pub fn complete(
    base: &str,
    api_key: &str,
    request: &ChatRequest,
) -> Result<ChatResponse, HttpError> {
    let body = serde_json::to_value(request).map_err(|e| HttpError::Json(e.to_string()))?;
    let raw = post_json(&near_url(base), api_key, &body)?;
    serde_json::from_value(raw).map_err(|e| HttpError::Json(e.to_string()))
}

/// List the NEAR model catalog (live network).
pub fn list_models(base: &str, api_key: &str) -> Result<Vec<ModelEntry>, HttpError> {
    let raw = get_json(&models_url(base), api_key)?;
    let resp: ModelsResponse =
        serde_json::from_value(raw).map_err(|e| HttpError::Json(e.to_string()))?;
    Ok(resp.data)
}

/// Full attestation report URL from a base.
#[must_use]
pub fn attestation_url(base: &str) -> String {
    join(base, ATTESTATION_PATH)
}

/// Fetch the TEE attestation report for a nonce (live network, non-billable).
/// Pair with `attest::verify(&report, nonce)` before displaying answers.
pub fn attestation_report(
    base: &str,
    api_key: &str,
    nonce: &str,
) -> Result<AttestationReport, HttpError> {
    let url = format!("{}?nonce={nonce}", attestation_url(base));
    let raw = get_json(&url, api_key)?;
    serde_json::from_value(raw).map_err(|e| HttpError::Json(e.to_string()))
}

/// Cheapest catalog entry for a token plan, or `None` when none are priced.
/// This is the live "best economical model" pick.
#[must_use]
pub fn cheapest(models: &[ModelEntry], in_tokens: u64, out_tokens: u64) -> Option<&ModelEntry> {
    models
        .iter()
        .filter_map(|m| m.estimated_cents(in_tokens, out_tokens).map(|c| (c, m)))
        .min_by_key(|(c, _)| *c)
        .map(|(_, m)| m)
}

/// Capability score for quality-aware picks: reasoning counts most,
/// then tools, then a roomy context window.
#[must_use]
pub fn capability(entry: &ModelEntry) -> u64 {
    u64::from(entry.supports_reasoning) * 4
        + u64::from(entry.supports_tools) * 2
        + u64::from(entry.context_length.unwrap_or(0) > 64_000)
}

/// Best economical entry with teeth: cheapest price wins, ties break
/// toward stronger models (reasoning, tools, large context).
/// Returns `None` when nothing is priced.
#[must_use]
pub fn pick_best(models: &[ModelEntry], in_tokens: u64, out_tokens: u64) -> Option<&ModelEntry> {
    models
        .iter()
        .filter_map(|m| m.estimated_cents(in_tokens, out_tokens).map(|c| (c, m)))
        .min_by(|(ca, ma), (cb, mb)| {
            ca.cmp(cb)
                .then_with(|| capability(mb).cmp(&capability(ma)))
                .then_with(|| ma.id.cmp(&mb.id))
        })
        .map(|(_, m)| m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_join_paths() {
        assert_eq!(
            jev_url("https://example.test"),
            "https://example.test/v1/systemone"
        );
        assert_eq!(
            near_url("https://cloud-api.near.ai/v1/"),
            "https://cloud-api.near.ai/v1/chat/completions"
        );
    }

    #[test]
    fn missing_env_yields_none() {
        assert!(env_key("JEAR_DEFINITELY_UNSET_KEY_12345").is_none());
    }

    #[test]
    fn near_base_defaults_to_documented() {
        // Env override absent in CI; default must be the documented base.
        if env_key(NEAR_BASE_ENV).is_none() {
            assert_eq!(near_base(), CLOUD_BASE_URL);
        }
    }

    #[test]
    fn models_url_joins_path() {
        assert_eq!(
            models_url("https://cloud-api.near.ai/v1/"),
            "https://cloud-api.near.ai/v1/models"
        );
    }

    #[test]
    fn attestation_url_carries_path() {
        assert_eq!(
            attestation_url("https://cloud-api.near.ai/v1/"),
            "https://cloud-api.near.ai/v1/attestation/report"
        );
    }

    #[test]
    fn cheapest_picks_lowest_estimate() {
        use crate::near_wire::ModelEntry;
        let cheap = ModelEntry {
            id: "cheap".to_string(),
            context_length: None,
            max_output_length: None,
            input_price_per_mtok: Some(0.20),
            output_price_per_mtok: Some(1.00),
            supports_tools: true,
            supports_reasoning: false,
        };
        let pricey = ModelEntry {
            id: "pricey".to_string(),
            context_length: None,
            max_output_length: None,
            input_price_per_mtok: Some(2.00),
            output_price_per_mtok: Some(12.00),
            supports_tools: true,
            supports_reasoning: true,
        };
        let models = vec![pricey, cheap];
        let pick = cheapest(&models, 1000, 1000).expect("priced models");
        assert_eq!(pick.id, "cheap");
        assert!(cheapest(&[], 1000, 1000).is_none());
    }

    #[test]
    fn pick_best_breaks_price_ties_by_capability() {
        use crate::near_wire::ModelEntry;
        let plain = ModelEntry {
            id: "plain".to_string(),
            context_length: Some(8_000),
            max_output_length: None,
            input_price_per_mtok: Some(0.20),
            output_price_per_mtok: Some(1.00),
            supports_tools: false,
            supports_reasoning: false,
        };
        let strong = ModelEntry {
            id: "strong".to_string(),
            context_length: Some(128_000),
            max_output_length: None,
            input_price_per_mtok: Some(0.20),
            output_price_per_mtok: Some(1.00),
            supports_tools: true,
            supports_reasoning: true,
        };
        let models = vec![plain, strong];
        let pick = pick_best(&models, 1000, 1000).expect("priced models");
        assert_eq!(pick.id, "strong");
        assert!(pick_best(&[], 1000, 1000).is_none());
    }
}
