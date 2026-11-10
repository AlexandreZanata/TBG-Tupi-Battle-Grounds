//! Deterministic champion action/status state, with 1v1 values resolved from the shared roster.
use crate::{geom::Pos, state::Credit};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PulseKind {
    Poison,
    Bleed,
    Venom,
    Regrowth,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Pulse {
    pub kind: PulseKind,
    pub amount: i32,
    pub left: u8,
    pub next: u32,
    pub gap: u16,
    pub credit: Credit,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Cast {
    pub slot: u8,
    pub target: u32,
    pub centre: Pos,
    pub left: u16,
    pub windup: u16,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Channel {
    pub target: u32,
    pub amount: i32,
    pub gap: u16,
    pub left: u8,
    pub next: u32,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Fx {
    pub attacks: u16,
    pub target: u32,
    pub walked: u16,
    pub ready: [u16; 2],
    pub channel: Option<Channel>,
    pub casts: [Option<Cast>; 2],
    pub shield: i32,
    pub shield_until: u32,
    pub reduction: u16,
    pub reduction_until: u32,
    pub slow: u16,
    pub slow_until: u32,
    pub root_until: u32,
    pub weaken: u16,
    pub weaken_until: u32,
    pub cut: u16,
    pub cut_until: u32,
    pub ward_until: u32,
    pub immune_until: u32,
    pub confused_until: u32,
    pub betray_bp: u16,
    pub betrayed: bool,
    pub betray_immune: u16,
    pub betray_credit: Option<Credit>,
    pub mark: u32,
    pub mark_until: u32,
    pub plates: u8,
    pub plate_hp: i32,
    pub essence: u16,
    pub essence_progress: u32,
    pub fish: Vec<(u16, u32)>,
    pub basic: Option<Cast>,
    pub summon_cd: u16,
    pub charged: bool,
    pub move_bp: u16,
    pub aura_dr: u16,
    pub ignore_slow: bool,
    pub bond: u32,
    pub carrier: u32,
    pub passenger: u32,
    pub board_until: u32,
    pub summon: u32,
    pub expires: u32,
    pub pulses: Vec<Pulse>,
    pub echo: i32,
    pub echo_until: u32,
    pub echo_credit: Option<Credit>,
    pub spark: i32,
    pub spark_until: u32,
    pub spark_credit: Option<Credit>,
    pub interrupt_until: u32,
}
