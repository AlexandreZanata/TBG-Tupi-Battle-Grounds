//! P61 E1 internal ranked statistics (§6–§7, §9): decode stored summaries
//! with their own revision catalog, apply the exact eligibility rules and
//! aggregate descriptive, association-only metrics. Pure: the SQL read is
//! `store::stats::read_cohort`; nothing here writes or changes a rating.

pub mod aggregate;
pub mod render;

use serde_json::Value;

use crate::store::StoredMatch;

/// The revision catalog stored in `ranked_rulesets` (P61 §3).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Catalog {
    pub kinds: Vec<KindInfo>,
    pub augments: Vec<String>,
    pub arenas: Vec<String>,
    pub tick_hz: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct KindInfo {
    pub name: String,
    pub purchasable: bool,
    pub faction: usize,
    pub archetype: String,
}

impl Catalog {
    pub fn parse(json: &str) -> Option<Catalog> {
        let v: Value = serde_json::from_str(json).ok()?;
        let kinds = v["kinds"]
            .as_array()?
            .iter()
            .map(|k| {
                Some(KindInfo {
                    name: k.get(1)?.as_str()?.to_string(),
                    purchasable: k.get(2)?.as_bool()?,
                    faction: usize::try_from(k.get(4)?.as_u64()?).ok()?,
                    archetype: k.get(5)?.as_str()?.to_string(),
                })
            })
            .collect::<Option<Vec<_>>>()?;
        let augments = v["augments"]
            .as_array()?
            .iter()
            .map(|a| Some(a.get(1)?.as_str()?.to_string()))
            .collect::<Option<Vec<_>>>()?;
        let arenas = v["arenas"]
            .as_array()?
            .iter()
            .map(|a| a.as_str().map(str::to_string))
            .collect::<Option<Vec<_>>>()?;
        Some(Catalog {
            kinds,
            augments,
            arenas,
            tick_hz: u32::try_from(v["tick_hz"].as_u64()?).ok()?,
        })
    }
}

/// One stored unit entry of a side (P61 §4 JSON v1 order).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UnitRow {
    pub kind: usize,
    pub bought: u32,
    pub first_tick: Option<u32>,
    pub forms: u8,
    pub max_tier: u8,
    pub tier1: u32,
    pub tier2: u32,
}

/// One side's decoded summary.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SideData {
    pub units: Vec<UnitRow>,
    pub synergy_max: [u8; tbg_core::FACTION_COUNT],
    pub synergy_ticks: [[u32; 2]; tbg_core::FACTION_COUNT],
    pub offered: Vec<(usize, u32)>,
    pub chosen: Vec<(usize, u32)>,
    pub stronghold: usize,
    pub arcanas: Vec<u8>,
}

/// An eligible match: authoritative facts plus both decoded sides.
#[derive(Clone, Debug, PartialEq)]
pub struct MatchObs {
    pub accounts: [i64; 2],
    pub winner: Option<usize>,
    pub end_detail: String,
    pub combat_ticks: u32,
    pub wall_ms: u64,
    pub arena: usize,
    pub ended_unix: i64,
    pub before: [u16; 2],
    pub sides: [SideData; 2],
}

/// Internal coverage: every exclusion is counted, never hidden (§6).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Coverage {
    pub stored: u64,
    pub invalidated: u64,
    pub review: u64,
    pub no_change: u64,
    pub not_started: u64,
    pub missing_summary: u64,
    pub unreadable_summary: u64,
    pub eligible: u64,
}

/// §6 main cohort: Normal + Applied, reached Running, complete v1 summary.
pub fn eligible(matches: &[StoredMatch], catalog: &Catalog) -> (Vec<MatchObs>, Coverage) {
    let mut cov = Coverage::default();
    let mut out = Vec::new();
    for m in matches {
        cov.stored += 1;
        match (m.risk, m.disposition) {
            (2, _) => cov.invalidated += 1,
            (1, _) => cov.review += 1,
            (_, 1) => cov.no_change += 1,
            _ if !m.reached_running => cov.not_started += 1,
            _ => match m.payload.as_deref() {
                None => cov.missing_summary += 1,
                Some(p) => match decode(m, p, catalog) {
                    Some(obs) => {
                        cov.eligible += 1;
                        out.push(obs);
                    }
                    None => cov.unreadable_summary += 1,
                },
            },
        }
    }
    (out, cov)
}

/// An unknown schema or an incomplete side is a coverage gap, never zeros.
fn decode(m: &StoredMatch, payload: &str, catalog: &Catalog) -> Option<MatchObs> {
    let v: Value = serde_json::from_str(payload).ok()?;
    if !matches!(v["schema"].as_u64()?, 1 | 2) {
        return None;
    }
    let side = |i: usize| -> Option<SideData> {
        let s = v["sides"].get(i)?;
        if !s["complete"].as_bool()? {
            return None;
        }
        let n = |x: &Value| x.as_u64().and_then(|n| u32::try_from(n).ok());
        let units = s["units"]
            .as_array()?
            .iter()
            .map(|u| {
                let kind = usize::try_from(u.get(0)?.as_u64()?).ok()?;
                (kind < catalog.kinds.len()).then_some(())?;
                Some(UnitRow {
                    kind,
                    bought: n(u.get(1)?)?,
                    first_tick: n(u.get(2)?),
                    forms: u8::try_from(u.get(3)?.as_u64()?).ok()?,
                    max_tier: u8::try_from(u.get(4)?.as_u64()?).ok()?,
                    tier1: n(u.get(5)?)?,
                    tier2: n(u.get(6)?)?,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        let pairs = |key: &str| -> Option<Vec<(usize, u32)>> {
            s[key]
                .as_array()?
                .iter()
                .map(|p| Some((usize::try_from(p.get(0)?.as_u64()?).ok()?, n(p.get(1)?)?)))
                .collect()
        };
        let mut synergy_max = [0u8; tbg_core::FACTION_COUNT];
        let mut synergy_ticks = [[0u32; 2]; tbg_core::FACTION_COUNT];
        let factions = s["synergy_max"].as_array()?.len();
        if factions != 4 && factions != tbg_core::FACTION_COUNT {
            return None;
        }
        if s["synergy_ticks"].as_array()?.len() != factions {
            return None;
        }
        for f in 0..factions {
            synergy_max[f] = u8::try_from(s["synergy_max"].get(f)?.as_u64()?).ok()?;
            for (lvl, slot) in synergy_ticks[f].iter_mut().enumerate() {
                *slot = n(s["synergy_ticks"].get(f)?.get(lvl)?)?;
            }
        }
        Some(SideData {
            units,
            synergy_max,
            synergy_ticks,
            offered: pairs("offered")?,
            chosen: pairs("chosen")?,
            stronghold: usize::try_from(s["stronghold"].get(0)?.as_u64()?).ok()?,
            arcanas: s["arcanas"]
                .as_array()?
                .iter()
                .map(|r| r.as_u64().and_then(|r| u8::try_from(r).ok()))
                .collect::<Option<Vec<_>>>()?,
        })
    };
    Some(MatchObs {
        accounts: m.accounts,
        winner: m.winner,
        end_detail: m.end_detail.clone(),
        combat_ticks: m.combat_ticks,
        wall_ms: v["wall_ms"].as_u64()?,
        arena: usize::try_from(v["arena"].as_u64()?).ok()?,
        ended_unix: m.ended_unix,
        before: m.before,
        sides: [side(0)?, side(1)?],
    })
}

/// Wilson 95 % interval for `w` successes in `n` (§7); `None` for `n = 0`.
/// Descriptive under clustering; never applied to score rates.
pub fn wilson(w: u64, n: u64) -> Option<(f64, f64)> {
    if n == 0 {
        return None;
    }
    let (z, n, p) = (1.96f64, n as f64, w as f64 / n as f64);
    let z2 = z * z;
    let denom = 1.0 + z2 / n;
    let center = (p + z2 / (2.0 * n)) / denom;
    let half = z * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt() / denom;
    Some(((center - half).max(0.0), (center + half).min(1.0)))
}
