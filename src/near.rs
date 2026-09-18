//! NEAR AI Cloud types.
//!
//! Pure types only — no HTTP calls yet. Covers the OpenAI-compatible
//! endpoint, TEE vs proxied models, and safe key handling.

/// Base URL for all NEAR AI Cloud REST calls.
pub const CLOUD_BASE_URL: &str = "https://cloud-api.near.ai/v1";

/// Chat completions path (primary inference endpoint).
pub const CHAT_COMPLETIONS_PATH: &str = "/chat/completions";

/// Models list path (catalog with pricing + features).
pub const MODELS_PATH: &str = "/models";

/// Where a model executes — determines privacy guarantees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hosting {
    /// Runs on NEAR GPU fleet in TEE (Intel TDX + NVIDIA TEE).
    /// Supports attestation, signatures, verification. Nobody —
    /// not even NEAR — can see prompts or outputs.
    Tee,
    /// Proxied to upstream provider (OpenAI, Anthropic, Gemini).
    /// Same unified API/billing, but TEE guarantees do not extend.
    Proxied,
}

/// A model entry from the catalog (subset we route on).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelInfo {
    /// Model id, e.g. "zai-org/GLM-5.1-FP8".
    pub id: String,
    /// Where it runs.
    pub hosting: Hosting,
    /// Supports tool calling.
    pub supports_tools: bool,
}

impl ModelInfo {
    /// Build a catalog entry.
    #[must_use]
    pub fn new(id: impl Into<String>, hosting: Hosting, supports_tools: bool) -> Self {
        Self {
            id: id.into(),
            hosting,
            supports_tools,
        }
    }

    /// True when safe for sensitive state (TEE-only).
    #[must_use]
    pub fn is_private(&self) -> bool {
        self.hosting == Hosting::Tee
    }
}

/// API key wrapper that redacts on `Debug` to avoid log leaks.
#[derive(Clone, PartialEq, Eq)]
pub struct ApiKey(String);

impl ApiKey {
    /// Wrap a raw `sk-...` key. Empty keys are rejected.
    #[must_use]
    pub fn new(key: impl Into<String>) -> Option<Self> {
        let key = key.into();
        if key.trim().is_empty() {
            return None;
        }
        Some(Self(key))
    }
}

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ApiKey(redacted)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tee_model_is_private() {
        let m = ModelInfo::new("zai-org/GLM-5.1-FP8", Hosting::Tee, true);
        assert!(m.is_private());
    }

    #[test]
    fn proxied_model_is_not_private() {
        let m = ModelInfo::new("openai/gpt-5", Hosting::Proxied, true);
        assert!(!m.is_private());
    }

    #[test]
    fn api_key_rejects_empty_and_redacts() {
        assert!(ApiKey::new("").is_none());
        assert!(ApiKey::new("   ").is_none());
        let k = ApiKey::new("sk-test").expect("valid key");
        assert_eq!(format!("{k:?}"), "ApiKey(redacted)");
    }

    #[test]
    fn endpoints_are_stable() {
        assert_eq!(CLOUD_BASE_URL, "https://cloud-api.near.ai/v1");
        assert_eq!(CHAT_COMPLETIONS_PATH, "/chat/completions");
        assert_eq!(MODELS_PATH, "/models");
    }
}
