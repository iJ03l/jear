//! IronClaw agent types.
//!
//! Pure types only — no SSH/HTTP calls yet. Covers personal instances,
//! channels, cost caps, and vault references (never raw secrets).

/// How the user reaches their IronClaw agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    /// Full WebChat at `https://<id>.agents.near.ai`.
    Web,
    /// Slack workspace integration.
    Slack,
    /// Telegram bot integration.
    Telegram,
    /// Local terminal via `ironclaw chat` / `run`.
    Terminal,
}

/// Where an IronClaw instance runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeployTarget {
    /// 1-click private instance on NEAR AI Cloud (TEE enclave).
    /// Holds the hub instance id, e.g. "abc123".
    Hub(String),
    /// Local single binary (`ironclaw serve` on 127.0.0.1:3000).
    Local,
}

/// Spend guard passed through to `ironclaw.yaml` + router.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CostCaps {
    /// Max cents per day (maps to `max_daily_cost_cents`).
    pub max_daily_cents: u64,
    /// Max cents per single task.
    pub max_task_cents: u64,
}

impl CostCaps {
    /// Build caps. Returns `None` when task cap exceeds daily cap.
    #[must_use]
    pub fn new(max_daily_cents: u64, max_task_cents: u64) -> Option<Self> {
        if max_task_cents > max_daily_cents {
            return None;
        }
        Some(Self {
            max_daily_cents,
            max_task_cents,
        })
    }

    /// True when an estimate fits inside the task cap.
    #[must_use]
    pub fn allows(&self, estimated_cents: u64) -> bool {
        estimated_cents <= self.max_task_cents
    }
}

/// Reference to a secret in the encrypted vault — never the value itself.
/// IronClaw injects values only at the host boundary for allowlisted endpoints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultRef(String);

impl VaultRef {
    /// Wrap a vault reference name, e.g. "GMAIL_TOKEN".
    #[must_use]
    pub fn new(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        if name.trim().is_empty() {
            return None;
        }
        Some(Self(name))
    }

    /// Borrow the reference name (safe to log).
    #[must_use]
    pub fn name(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps_reject_inverted_limits() {
        assert!(CostCaps::new(100, 500).is_none());
        let c = CostCaps::new(500, 100).expect("valid caps");
        assert!(c.allows(100));
        assert!(!c.allows(101));
    }

    #[test]
    fn vault_ref_rejects_empty() {
        assert!(VaultRef::new("").is_none());
        let v = VaultRef::new("GMAIL_TOKEN").expect("valid ref");
        assert_eq!(v.name(), "GMAIL_TOKEN");
    }

    #[test]
    fn hub_holds_instance_id() {
        let t = DeployTarget::Hub("abc123".to_string());
        assert!(matches!(t, DeployTarget::Hub(_)));
        assert!(matches!(DeployTarget::Local, DeployTarget::Local));
    }
}
