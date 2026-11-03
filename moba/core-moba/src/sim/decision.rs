//! Persistent mission/engagement state machine; combat and navigation execute its orders.
use super::targeting::{self, Choice, Vulnerability};
use crate::{
    champions,
    geom::Pos,
    state::{Activity, Brain, MobaMatch, Unit},
    types::Side,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    #[default]
    Advance,
    Pursue,
    Windup,
    Recover,
    Support,
    Disabled,
    Repath,
    Return,
    Wait,
    Formation,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reason {
    #[default]
    Mission,
    Acquired,
    Retained,
    TargetGone,
    Invalid,
    PursuitLimit,
    Unreachable,
    Controlled,
    Channel,
    Obstructed,
    Formation,
    Arrived,
}

/// Internal replay state, not a new client order or wire schema.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Decision {
    pub phase: Phase,
    pub reason: Reason,
    pub changed_because: Reason,
    pub target: u32,
    pub anchor: Pos,
    pub since: u32,
}

#[derive(Clone, Copy)]
pub(super) enum Order {
    Hold,
    Move,
    Windup,
    Strike(Choice),
}

pub fn phase(u: &mut Unit, next: Phase) {
    u.decision.phase = next;
    u.activity = match next {
        Phase::Advance | Phase::Return => Activity::Advance,
        Phase::Pursue => Activity::Pursue,
        Phase::Windup | Phase::Recover | Phase::Support => Activity::Attack,
        _ => Activity::Hold,
    };
}

fn commit(m: &mut MobaMatch, i: usize, target: u32, reason: Reason) {
    let u = &mut m.units[i];
    if u.decision.target != target {
        u.decision.anchor = u.pos;
        u.decision.since = m.tick;
        u.decision.changed_because = if reason == Reason::Retained {
            Reason::Acquired
        } else {
            reason
        };
        u.decision.target = target;
        u.navigation.reset_route();
        if u.fx.target != target && champions::same_target_counter(u) {
            u.fx.attacks = 0;
            u.fx.target = target;
        }
    }
    u.target = target;
    u.decision.reason = reason;
}

pub(super) fn order(m: &mut MobaMatch, vul: &Vulnerability, i: usize) -> Order {
    if let Some(order) = locked(&mut m.units[i], m.tick) {
        return order;
    }
    let old = m.units[i].target;
    let choice = targeting::choose(m, vul, i);
    let id = choice.map_or(0, |c| m.units[c.slot].id);
    let reason = if old != 0 && old != id {
        targeting::release_reason(m, vul, i)
    } else if id == 0 {
        Reason::Mission
    } else if old == id {
        Reason::Retained
    } else {
        Reason::Acquired
    };
    commit(m, i, id, reason);
    let Some(choice) = choice else {
        if targeting::support_holds(m, i) {
            phase(&mut m.units[i], Phase::Wait);
            return Order::Hold;
        }
        let returning = matches!(m.units[i].brain, Brain::Neutral { .. });
        phase(
            &mut m.units[i],
            if returning {
                Phase::Return
            } else {
                Phase::Advance
            },
        );
        return Order::Move;
    };
    engage(m, i, choice)
}

fn locked(u: &mut Unit, now: u32) -> Option<Order> {
    if !u.alive() || champions::cannot_basic(u, now) {
        u.fx.basic = None;
        u.fx.betrayed = false;
        u.decision.reason = if u.fx.channel.is_some() {
            Reason::Channel
        } else {
            Reason::Controlled
        };
        phase(
            u,
            if u.fx.channel.is_some() {
                Phase::Support
            } else {
                Phase::Disabled
            },
        );
        return Some(Order::Hold);
    }
    if u.fx.basic.is_some() {
        phase(u, Phase::Windup);
        return Some(Order::Windup);
    }
    None
}

fn engage(m: &mut MobaMatch, i: usize, choice: Choice) -> Order {
    let (a, b) = (&m.units[i], &m.units[choice.slot]);
    let ready = in_position(a, b);
    // Hysteresis only holds an existing recovery pose. It never extends the
    // legal strike range or lets a newly acquired target stop the approach.
    let recovering = a.cooldown > 0
        && matches!(
            a.decision.phase,
            Phase::Windup | Phase::Recover | Phase::Support
        )
        && a.pos
            .within(b.pos, reach(a, b) + tbg_core::content::occupancy::GAP)
        && leash_position(a, b, a.pos);
    if !ready && !recovering {
        phase(&mut m.units[i], Phase::Pursue);
        return Order::Move;
    }
    let cooldown = a.cooldown;
    phase(
        &mut m.units[i],
        if choice.support {
            Phase::Support
        } else {
            Phase::Recover
        },
    );
    if cooldown > 0 || !ready {
        Order::Hold
    } else {
        Order::Strike(choice)
    }
}

/// The windup/channel owns the visible aim even if its effect differs from
/// the normal combat target (betrayal or a healing channel). Timed abilities
/// keep their separate presentation cue and do not replace the basic aim.
pub fn presentation_target(u: &Unit) -> u32 {
    u.fx.basic
        .map(|c| c.target)
        .or_else(|| u.fx.channel.map(|c| c.target))
        .unwrap_or(u.target)
}

fn leash_position(a: &Unit, b: &Unit, at: Pos) -> bool {
    !(matches!(a.brain, Brain::Jungle { .. })
        && b.side == Side::Neutral
        && matches!(b.brain,Brain::Neutral{camp} if !at.within(crate::map::camp(usize::from(camp)).0,crate::content::CAMP_LEASH)))
}
pub fn reach(a: &Unit, b: &Unit) -> i32 {
    if champions::abilities(a).iter().any(|ability| {
        matches!(
            ability,
            tbg_core::content::Ability::Brew { .. } | tbg_core::content::Ability::Puppet { .. }
        )
    }) {
        return a.stats.range + b.stats.radius;
    }
    if a.side == b.side {
        if a.stats.heal > 0 {
            a.stats.range
        } else {
            a.stats.range + b.stats.radius
        }
    } else {
        champions::reach(a, b)
    }
}
pub fn in_position_at(a: &Unit, b: &Unit, at: Pos) -> bool {
    at.within(b.pos, reach(a, b)) && leash_position(a, b, at)
}
pub fn in_position(a: &Unit, b: &Unit) -> bool {
    in_position_at(a, b, a.pos)
}
