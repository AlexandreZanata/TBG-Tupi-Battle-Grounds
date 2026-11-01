//! P60 §4 integrity policy, read once at start-up and snapshotted into each
//! settlement's facts. Only `shadow` exists: budgets and signals are
//! computed and recorded as `would_*`, and the existing Elo always applies.
//! `enforce` is rejected until the receipt UI and owner decision O1 land.

use tbg_core::ranked_integrity::BudgetPolicy;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RankedPolicy {
    /// **Proposed** (O1): 120 positive points per pair per window.
    pub pair_cap: u32,
    /// **Proposed** (O1): 120 positive points from immature opponents.
    pub new_source_cap: u32,
    /// Rolling window in seconds (604800 = 7 days).
    pub window_secs: i64,
}

impl Default for RankedPolicy {
    fn default() -> RankedPolicy {
        RankedPolicy {
            pair_cap: 120,
            new_source_cap: 120,
            window_secs: 604_800,
        }
    }
}

impl RankedPolicy {
    pub fn budget(&self) -> BudgetPolicy {
        BudgetPolicy {
            pair_limit: self.pair_cap,
            immature_limit: self.new_source_cap,
        }
    }

    /// `TBG_RANKED_INTEGRITY`, `TBG_RANKED_PAIR_CAP`,
    /// `TBG_RANKED_NEW_SOURCE_CAP`, `TBG_RANKED_WINDOW_SECS`. Any invalid or
    /// out-of-bounds value is a start-up error, never a silent default.
    pub fn from_env() -> Result<RankedPolicy, String> {
        RankedPolicy::from_lookup(|k| std::env::var(k).ok())
    }

    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<RankedPolicy, String> {
        match get("TBG_RANKED_INTEGRITY").as_deref() {
            None | Some("shadow") => {}
            Some("enforce") => {
                return Err(
                    "TBG_RANKED_INTEGRITY=enforce is not available (P60 stage 2 \
                            needs the receipt UI and an owner decision)"
                        .into(),
                )
            }
            Some(other) => return Err(format!("TBG_RANKED_INTEGRITY={other}: use shadow")),
        }
        let d = RankedPolicy::default();
        let bounded = |key: &str, default: u64, max: u64| -> Result<u64, String> {
            match get(key) {
                None => Ok(default),
                Some(v) => match v.parse::<u64>() {
                    Ok(n) if (1..=max).contains(&n) => Ok(n),
                    _ => Err(format!("{key}={v}: expected an integer in 1..={max}")),
                },
            }
        };
        Ok(RankedPolicy {
            pair_cap: bounded("TBG_RANKED_PAIR_CAP", u64::from(d.pair_cap), 10_000)? as u32,
            new_source_cap: bounded(
                "TBG_RANKED_NEW_SOURCE_CAP",
                u64::from(d.new_source_cap),
                10_000,
            )? as u32,
            window_secs: bounded("TBG_RANKED_WINDOW_SECS", d.window_secs as u64, 2_592_000)? as i64,
        })
    }
}

/// P61 §10: `TBG_RANKED_STATS`. `collect` (default) stores the optional
/// per-match summary after settlement; `off` disables only that optional
/// collection, never P60 settlement. `public` (the E2 route) is not
/// available yet and is rejected at start-up.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StatsMode {
    Off,
    #[default]
    Collect,
}

impl StatsMode {
    pub fn from_env() -> Result<StatsMode, String> {
        StatsMode::parse(std::env::var("TBG_RANKED_STATS").ok().as_deref())
    }

    pub fn parse(value: Option<&str>) -> Result<StatsMode, String> {
        match value {
            None | Some("collect") => Ok(StatsMode::Collect),
            Some("off") => Ok(StatsMode::Off),
            Some("public") => Err("TBG_RANKED_STATS=public is not available yet (P61 E2)".into()),
            Some(other) => Err(format!("TBG_RANKED_STATS={other}: use collect or off")),
        }
    }
}
