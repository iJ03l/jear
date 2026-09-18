//! Policy engine: budget x quality x sensitivity.
//!
//! Pure, deterministic routing on top of Jev answers. No network calls.
//! The answer tells you *what*; confidence tells you *whether to act*.

/// Where a request should execute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// Direct NEAR AI Cloud inference (lookup, summary, extraction).
    DirectLlm,
    /// Personal IronClaw instance (needs tools, email, calendar, code).
    IronclawPersonal,
    /// Hired marketplace specialist (finance, legal, research pro).
    MarketplaceSpecialist,
    /// Stop and ask a human (low confidence, over budget, unsafe).
    HumanReview,
}

/// Inputs to the policy, built from Jev `Choice`/`Score`/`Noul` answers.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyInput {
    /// Task complexity 0.0 (lookup) .. 2.0 (high-stakes).
    pub complexity: f64,
    /// Data sensitivity 0.0 (public) .. 2.0 (secret/regulated).
    pub sensitivity: f64,
    /// Jev `needs_private` probability 0.0..=1.0.
    pub needs_private: f64,
    /// Jev routing confidence 0.0..=1.0.
    pub confidence: f64,
    /// Remaining budget in cents.
    pub budget_cents: u64,
    /// Estimated cost in cents for the preferred route.
    pub estimated_cents: u64,
}

/// Policy outcome with a human-readable reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    /// Chosen route.
    pub route: Route,
    /// Must stay on TEE-hosted private models.
    pub force_tee: bool,
    /// Why this route was chosen (for UI + logs).
    pub reason: String,
}

/// Decide a route. Thresholds are explicit so behavior is auditable.
#[must_use]
pub fn decide(input: &PolicyInput, confidence_floor: f64) -> Decision {
    let tee = input.needs_private >= 0.8 || input.sensitivity >= 1.2;

    if input.confidence < confidence_floor {
        return Decision {
            route: Route::HumanReview,
            force_tee: tee,
            reason: "low Jev confidence — escalate".to_string(),
        };
    }

    if input.estimated_cents > input.budget_cents {
        return Decision {
            route: Route::HumanReview,
            force_tee: tee,
            reason: "over budget — needs approval".to_string(),
        };
    }

    if input.complexity < 0.7 {
        return Decision {
            route: Route::DirectLlm,
            force_tee: tee,
            reason: "simple lookup — direct inference".to_string(),
        };
    }

    if input.complexity < 1.4 {
        return Decision {
            route: Route::IronclawPersonal,
            force_tee: tee,
            reason: "multi-step with tools — personal agent".to_string(),
        };
    }

    Decision {
        route: Route::MarketplaceSpecialist,
        force_tee: tee,
        reason: "high-stakes — hire specialist".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> PolicyInput {
        PolicyInput {
            complexity: 0.3,
            sensitivity: 0.2,
            needs_private: 0.1,
            confidence: 0.9,
            budget_cents: 50,
            estimated_cents: 5,
        }
    }

    #[test]
    fn simple_goes_direct() {
        let d = decide(&base(), 0.6);
        assert_eq!(d.route, Route::DirectLlm);
        assert!(!d.force_tee);
    }

    #[test]
    fn low_confidence_escalates() {
        let mut i = base();
        i.confidence = 0.4;
        let d = decide(&i, 0.6);
        assert_eq!(d.route, Route::HumanReview);
    }

    #[test]
    fn over_budget_needs_approval() {
        let mut i = base();
        i.estimated_cents = 500;
        let d = decide(&i, 0.6);
        assert_eq!(d.route, Route::HumanReview);
    }

    #[test]
    fn sensitive_forces_tee() {
        let mut i = base();
        i.sensitivity = 1.8;
        i.needs_private = 0.9;
        let d = decide(&i, 0.6);
        assert!(d.force_tee);
    }

    #[test]
    fn complex_hires_specialist() {
        let mut i = base();
        i.complexity = 1.8;
        let d = decide(&i, 0.6);
        assert_eq!(d.route, Route::MarketplaceSpecialist);
    }
}
