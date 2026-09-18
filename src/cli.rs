//! CLI args — offline parsing, no network.
//!
//! `jear "brief..." [--monthly-cents N] [--live]` keeps the demo
//! default when no brief is given. Limits are client-controlled.

/// Parsed command-line config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// User brief in plain English.
    pub brief: String,
    /// Client-set monthly cap in cents.
    pub monthly_cents: u64,
    /// Run live calls when keys are present (else offline demo).
    pub live: bool,
}

impl Config {
    /// Default offline demo config.
    #[must_use]
    pub fn demo() -> Self {
        Self {
            brief: "Summarize my last 3 payout-failure emails".to_string(),
            monthly_cents: 5_000,
            live: false,
        }
    }
}

/// Parse `std::env::args()`-style args (program name first).
/// Unknown flags are ignored so future options stay compatible.
#[must_use]
pub fn parse(args: &[String]) -> Config {
    let mut cfg = Config::demo();
    let mut brief_parts: Vec<String> = Vec::new();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--monthly-cents" => {
                if let Some(v) = args.get(i + 1).and_then(|s| s.parse::<u64>().ok()) {
                    if v > 0 {
                        cfg.monthly_cents = v;
                    }
                }
                i += 1;
            }
            "--live" => cfg.live = true,
            flag if flag.starts_with("--") => {}
            word => brief_parts.push(word.to_string()),
        }
        i += 1;
    }
    if !brief_parts.is_empty() {
        cfg.brief = brief_parts.join(" ");
    }
    cfg
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn defaults_to_demo() {
        let c = parse(&args(&["jear"]));
        assert_eq!(c, Config::demo());
        assert!(!c.live);
    }

    #[test]
    fn brief_joins_words() {
        let c = parse(&args(&["jear", "hello", "world"]));
        assert_eq!(c.brief, "hello world");
    }

    #[test]
    fn client_sets_monthly_and_live() {
        let c = parse(&args(&["jear", "--monthly-cents", "12000", "--live"]));
        assert_eq!(c.monthly_cents, 12_000);
        assert!(c.live);
    }

    #[test]
    fn bad_monthly_keeps_default() {
        let c = parse(&args(&["jear", "--monthly-cents", "abc"]));
        assert_eq!(c.monthly_cents, Config::demo().monthly_cents);
    }
}
