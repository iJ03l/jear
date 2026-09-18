//! Minimal sync HTTP helper for JSON APIs.
//!
//! Bearer keys are passed per call and never logged. No retries yet —
//! callers decide policy. Live calls need network; tests stay offline.

use std::fmt;

/// HTTP failure without leaking the bearer key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpError {
    /// Transport failure (DNS, TLS, timeout, connection).
    Transport(String),
    /// Non-2xx status with body preview (key never included).
    Status(u16, String),
    /// Response body was not valid JSON.
    Json(String),
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(msg) => write!(f, "transport error: {msg}"),
            Self::Status(code, body) => write!(f, "http {code}: {body}"),
            Self::Json(msg) => write!(f, "invalid json: {msg}"),
        }
    }
}

impl std::error::Error for HttpError {}

/// Join a base URL and path without doubling slashes.
#[must_use]
pub fn join(base: &str, path: &str) -> String {
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

/// POST JSON with bearer auth, parse JSON response.
/// The `bearer` value never appears in errors or logs.
pub fn post_json(
    url: &str,
    bearer: &str,
    body: &serde_json::Value,
) -> Result<serde_json::Value, HttpError> {
    let response = ureq::post(url)
        .set("Content-Type", "application/json")
        .set("Authorization", &format!("Bearer {bearer}"))
        .send_json(body.clone())
        .map_err(map_ureq)?;
    response
        .into_json::<serde_json::Value>()
        .map_err(|e| HttpError::Json(e.to_string()))
}

/// GET with bearer auth, parse JSON response (e.g. model catalogs).
/// The `bearer` value never appears in errors or logs.
pub fn get_json(url: &str, bearer: &str) -> Result<serde_json::Value, HttpError> {
    let response = ureq::get(url)
        .set("Authorization", &format!("Bearer {bearer}"))
        .call()
        .map_err(map_ureq)?;
    response
        .into_json::<serde_json::Value>()
        .map_err(|e| HttpError::Json(e.to_string()))
}

/// Map a `ureq` failure without echoing auth material.
fn map_ureq(e: ureq::Error) -> HttpError {
    match e {
        ureq::Error::Status(code, resp) => {
            let preview = resp.into_string().unwrap_or_default();
            let short: String = preview.chars().take(300).collect();
            HttpError::Status(code, short)
        }
        ureq::Error::Transport(t) => HttpError::Transport(t.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_avoids_double_slash() {
        assert_eq!(
            join("https://cloud-api.near.ai/v1/", "/chat/completions"),
            "https://cloud-api.near.ai/v1/chat/completions"
        );
        assert_eq!(
            join("https://cloud-api.near.ai/v1", "chat/completions"),
            "https://cloud-api.near.ai/v1/chat/completions"
        );
    }

    #[test]
    fn errors_never_echo_bearer() {
        let e = HttpError::Status(401, "unauthorized".to_string());
        let msg = e.to_string();
        assert!(msg.contains("401"));
        assert!(!msg.contains("sk-"));
        assert!(!msg.contains("Bearer"));
    }

    #[test]
    fn transport_error_displays() {
        let e = HttpError::Transport("dns failed".to_string());
        assert!(e.to_string().contains("dns failed"));
    }
}
