//! Monthly AI credit budget — fully client-controlled.
//!
//! The client sets any limit it wants; `jear` picks the best
//! *economical* model under that cap, not just the best model.
//! Pure types only — persistence comes later.

/// Monthly spend cap in cents, set by the client (any amount).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonthlyBudget {
    /// Max cents allowed per calendar month.
    pub limit_cents: u64,
    /// Cents already spent this month.
    pub spent_cents: u64,
}

impl MonthlyBudget {
    /// Build a fresh monthly budget with zero spend.
    /// Returns `None` for a zero limit.
    #[must_use]
    pub fn new(limit_cents: u64) -> Option<Self> {
        if limit_cents == 0 {
            return None;
        }
        Some(Self {
            limit_cents,
            spent_cents: 0,
        })
    }

    /// Cents left this month (saturates at zero).
    #[must_use]
    pub fn remaining(&self) -> u64 {
        self.limit_cents.saturating_sub(self.spent_cents)
    }

    /// Fraction spent 0.0..=1.0+ (over 1.0 means over budget).
    #[must_use]
    pub fn utilization(&self) -> f64 {
        if self.limit_cents == 0 {
            return 1.0;
        }
        self.spent_cents as f64 / self.limit_cents as f64
    }

    /// True when an estimate fits inside this month's remainder.
    #[must_use]
    pub fn can_afford(&self, estimated_cents: u64) -> bool {
        estimated_cents <= self.remaining()
    }

    /// Record spend (saturating — never wraps).
    pub fn record(&mut self, spent_cents: u64) {
        self.spent_cents = self.spent_cents.saturating_add(spent_cents);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_set_limit() {
        let b = MonthlyBudget::new(7_500).expect("valid budget");
        assert_eq!(b.remaining(), 7_500);
        assert!(b.can_afford(5));
        assert!(!b.can_afford(7_501));
    }

    #[test]
    fn spend_reduces_remaining() {
        let mut b = MonthlyBudget::new(7_500).expect("valid budget");
        b.record(7_490);
        assert_eq!(b.remaining(), 10);
        assert!(!b.can_afford(11));
        b.record(50);
        assert_eq!(b.remaining(), 0);
        assert!(b.utilization() >= 1.0);
    }

    #[test]
    fn rejects_zero_limit() {
        assert!(MonthlyBudget::new(0).is_none());
    }
}
