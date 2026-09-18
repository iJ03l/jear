//! jear — Jev-routed client for NEAR AI Cloud and IronClaw.
//!
//! Routing by budget, quality, and sensitivity.
//! Live calls via `http` + wire modules; demo stays offline.

pub mod attest;
pub mod budget;
pub mod cli;
pub mod http;
pub mod ironclaw;
pub mod jev;
pub mod jev_wire;
pub mod live;
pub mod near;
pub mod near_wire;
pub mod policy;
pub mod route;

/// Crate version from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns a friendly greeting with the current version.
///
/// Used as a smoke-test for the foundation build.
#[must_use]
pub fn hello() -> String {
    format!("jear v{VERSION} — foundation ready")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_contains_version() {
        let msg = hello();
        assert!(msg.contains(VERSION), "unexpected greeting: {msg}");
        assert!(msg.contains("jear"), "unexpected greeting: {msg}");
    }
}
