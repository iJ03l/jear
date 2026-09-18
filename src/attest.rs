//! TEE attestation report types + verify-before-display.
//!
//! Shapes for `GET /v1/attestation/report` (Dstack CPU quote). Checks
//! structure and nonce binding so replays fail fast. Full Intel DCAP
//! quote verification stays upstream/client-side — out of scope here.

use serde::Deserialize;

/// Report path (query params carry model, nonce, provider, ...).
pub const ATTESTATION_PATH: &str = "/v1/attestation/report";

/// Gateway CPU quote (required subset of `DstackCpuQuote`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct CpuQuote {
    /// Address that signs responses.
    #[serde(default)]
    pub signing_address: String,
    /// `ecdsa` or `ed25519`.
    #[serde(default)]
    pub signing_algo: String,
    /// Hex attestation quote (opaque to `jear`).
    #[serde(default)]
    pub intel_quote: String,
    /// Report data binding address + nonce.
    #[serde(default)]
    pub report_data: String,
    /// Echo of the request nonce (replay protection).
    #[serde(default)]
    pub request_nonce: String,
}

/// Attestation response (subset we verify on).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AttestationReport {
    /// Gateway quote — always present on success.
    pub gateway_attestation: CpuQuote,
}

/// Why a report fails verification (key material never included).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttestError {
    /// Required string field was empty.
    EmptyField(&'static str),
    /// `request_nonce` did not echo our nonce (possible replay).
    NonceMismatch,
}

impl std::fmt::Display for AttestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "attestation missing {field}"),
            Self::NonceMismatch => write!(f, "attestation nonce mismatch"),
        }
    }
}

impl std::error::Error for AttestError {}

/// Verify structure + nonce binding. Returns the signing address on success
/// so callers can pin it for per-response signature checks.
pub fn verify(report: &AttestationReport, expected_nonce: &str) -> Result<String, AttestError> {
    let q = &report.gateway_attestation;
    if q.signing_address.trim().is_empty() {
        return Err(AttestError::EmptyField("signing_address"));
    }
    if q.intel_quote.trim().is_empty() {
        return Err(AttestError::EmptyField("intel_quote"));
    }
    if q.report_data.trim().is_empty() {
        return Err(AttestError::EmptyField("report_data"));
    }
    if q.request_nonce != expected_nonce {
        return Err(AttestError::NonceMismatch);
    }
    Ok(q.signing_address.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quote(nonce: &str) -> CpuQuote {
        CpuQuote {
            signing_address: "0xabc".to_string(),
            signing_algo: "ecdsa".to_string(),
            intel_quote: "deadbeef".to_string(),
            report_data: "binds-address-and-nonce".to_string(),
            request_nonce: nonce.to_string(),
        }
    }

    fn report(nonce: &str) -> AttestationReport {
        AttestationReport {
            gateway_attestation: quote(nonce),
        }
    }

    #[test]
    fn valid_report_returns_signer() {
        assert_eq!(verify(&report("n-1"), "n-1"), Ok("0xabc".to_string()));
    }

    #[test]
    fn wrong_nonce_fails() {
        assert_eq!(
            verify(&report("n-1"), "n-2"),
            Err(AttestError::NonceMismatch)
        );
    }

    #[test]
    fn empty_quote_fails() {
        let mut r = report("n-1");
        r.gateway_attestation.intel_quote.clear();
        assert_eq!(
            verify(&r, "n-1"),
            Err(AttestError::EmptyField("intel_quote"))
        );
    }
}
