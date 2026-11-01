//! Freeze the same authoritative personal summary for result, reconnect and durable detail.

use super::Match;
use crate::store::player_history::{Envelope, HistorySide};
use tbg_core::{
    protocol::{
        AppliedAugment, ChampionPurchases, MatchMode, MatchOrigin, PersonalMatchSummary,
        TerminalStronghold,
    },
    Seat,
};

impl Match {
    pub(super) fn personal_summaries(
        &self,
        origin: MatchOrigin,
        wall_ms: u64,
    ) -> [PersonalMatchSummary; 2] {
        let ended = crate::store::release::now_unix();
        let collected = self
            .summary
            .clone()
            .finish(&self.state, wall_ms, crate::ranked::source_hash())
            .ok();
        Seat::BOTH.map(|seat| {
            let index = seat.index();
            let stats = &self.stats[index];
            let player = self.state.player(seat);
            let champions = collected
                .as_ref()
                .and_then(|v| v["sides"][index]["units"].as_array())
                .and_then(|units| {
                    units
                        .iter()
                        .map(|u| {
                            let kind = *tbg_core::types::ALL_KINDS
                                .get(usize::try_from(u[0].as_u64()?).ok()?)?;
                            Some(ChampionPurchases {
                                champion: format!("{kind:?}"),
                                purchases: u32::try_from(u[1].as_u64()?).ok()?,
                                evolution_tiers: [
                                    u32::try_from(u[5].as_u64()?).ok()?,
                                    u32::try_from(u[6].as_u64()?).ok()?,
                                ],
                            })
                        })
                        .collect::<Option<Vec<_>>>()
                });
            let purchases = champions
                .as_ref()
                .and_then(|c| c.iter().try_fold(0u32, |a, c| a.checked_add(c.purchases)));
            let outcome = self.state.outcome;
            PersonalMatchSummary {
                version: 1,
                quality: if champions.is_some() {
                    "complete"
                } else {
                    "partial"
                }
                .into(),
                mode: self.mode,
                origin,
                ended_unix: ended,
                arena: Some(self.state.arena.slug().into()),
                ruleset_version: Some(tbg_core::ruleset_version::RULESET_VERSION),
                source_hash: Some(format!("{:016x}", crate::ranked::source_hash())),
                combat_ticks: Some(
                    self.state
                        .tick
                        .saturating_sub(tbg_core::content::RUNNING_START),
                ),
                tick_hz: tbg_core::content::TICK_HZ as u16,
                wall_ms: Some(wall_ms),
                outcome: match outcome.and_then(|o| o.winner) {
                    None => "draw",
                    Some(w) if w == seat => "win",
                    _ => "loss",
                }
                .into(),
                reason: self
                    .end_detail
                    .unwrap_or(crate::ranked::EndDetail::Stronghold)
                    .as_str()
                    .into(),
                purchases,
                champions,
                evolutions: Some(stats.evolutions),
                fortifications: Some(stats.fortifications),
                augments: Some(
                    player
                        .augments
                        .iter()
                        .enumerate()
                        .filter(|(_, n)| **n > 0)
                        .map(|(id, n)| AppliedAugment {
                            id: id as u16,
                            copies: *n,
                        })
                        .collect(),
                ),
                stronghold: Some(TerminalStronghold {
                    kind: crate::live::kind_slug(player.stronghold).into(),
                    hp: player.stronghold_hp,
                    max_hp: self.state.stronghold(seat).max_hp,
                }),
            }
        })
    }

    pub(super) fn history_envelope(
        &self,
        key: [u8; 16],
        summaries: [PersonalMatchSummary; 2],
    ) -> Option<Envelope> {
        // A pre-combat cancellation never becomes an invented permanent loss.
        if !self.reached_running || self.players.iter().all(|p| p.account.is_none()) {
            return None;
        }
        let ranked = self.mode == MatchMode::Ranked;
        Some(Envelope {
            format: 1,
            key: if ranked {
                format!("ranked:{}", crate::ranked::hex(&key))
            } else {
                format!("result:{}", crate::ranked::hex(&self.result_id))
            },
            result_id: Some(self.result_id),
            ranked_key: ranked.then_some(key),
            ranked_row: None,
            winner: self
                .state
                .outcome
                .and_then(|o| o.winner)
                .map(|s| s.index() as u8),
            sides: summaries
                .into_iter()
                .enumerate()
                .map(|(i, summary)| HistorySide {
                    entry_id: crate::names::random_bytes::<16>(),
                    account: self.players[i].account.map(|(id, _)| id),
                    name: self.players[i].name.clone(),
                    summary,
                })
                .collect::<Vec<_>>()
                .try_into()
                .ok()?,
        })
    }
}
