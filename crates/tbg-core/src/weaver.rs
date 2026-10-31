//! R19's locked single-primary web and one post-Root Slow with transition-time defenses.
use crate::{
    content::Ability,
    hits, roster_slots,
    state::{Event, MatchState, Unit},
    types::{OnHit, UnitId},
};
fn parameters(s: &MatchState, u: &Unit) -> Option<(u16, u16, i32, u16, u16)> {
    s.rules()
        .def(u.kind)
        .ability(u.tier, u.branch)
        .into_iter()
        .find_map(|a| match a {
            Ability::StickyWeb {
                damage,
                root,
                range_add,
                slow_bp,
                slow_ticks,
                ..
            } => Some((damage, root, range_add, slow_bp, slow_ticks)),
            _ => None,
        })
}
pub(crate) fn extra_range(s: &MatchState, u: &Unit) -> i32 {
    parameters(s, u).map_or(0, |p| p.2)
}
pub(crate) fn resolve(s: &mut MatchState, u: &Unit, id: UnitId, ev: &mut Vec<Event>) {
    let Some((damage, root, _, bp, ticks)) = parameters(s, u) else {
        return;
    };
    let Some(i) = s.unit_index(id) else {
        return;
    };
    let amount = hits::auxiliary_amount(&s.stats_of(u), damage);
    hits::auxiliary_hit(s, u, id, amount, ev);
    if s.units[i].hp <= 0 {
        return;
    }
    let applied = hits::apply_on_hit(s, id, OnHit::Root { ticks: root }, u.seat, ev);
    #[cfg(feature = "lab")]
    crate::lab::status(s, u, applied);
    if applied > 0 && bp > 0 {
        let duration = hits::status_duration(s, id, u.seat, ticks);
        roster_slots::queue_slow(&mut s.units[i].fx, bp, duration);
    }
}
#[cfg(all(test, feature = "lab"))]
mod tests {
    use super::*;
    use crate::{combat, specials, ArcanaPreset, Seat, Status, UnitKind};
    fn arena() -> MatchState {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        s.lab_spawn_at(Seat::A, UnitKind::Weaver, 2, 1, 4000);
        s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 7300);
        s
    }
    #[test]
    fn r19_special_reaches_beyond_basic_and_guard_wins_only_within_extended_range() {
        let mut s = arena();
        s.units[0].action_cd = 0;
        assert_eq!(s.stats_of(&s.units[0]).range, 3000);
        let guard = s.lab_spawn_at(Seat::B, UnitKind::Rampart, 0, 0, 7500);
        specials::advance(&mut s, 0, &mut Vec::new());
        assert_eq!(s.units[0].fx.special.unwrap().target, Some(guard));
        assert_eq!(s.units[0].action_cd, 160);
        assert_eq!(
            specials::period(s.rules(), UnitKind::Weaver, 2, 0),
            Some(130)
        );
    }
    #[test]
    fn r19_root_extension_delays_one_slow_and_ward_on_transition_discards_it() {
        let mut s = arena();
        let a = s.units[0];
        let id = s.units[1].id;
        let hp = s.units[1].hp;
        resolve(&mut s, &a, id, &mut Vec::new());
        assert_eq!(s.units[1].hp, hp - 25);
        assert_eq!(s.units[1].status, Status::Rooted { ticks_left: 30 });
        hits::apply_on_hit(
            &mut s,
            id,
            OnHit::Root { ticks: 50 },
            Seat::A,
            &mut Vec::new(),
        );
        for _ in 0..49 {
            combat::tick_timers(&mut s);
        }
        assert!(s.units[1].fx.queued_slow.is_some());
        s.units[1].fx.ward_until = 100;
        combat::tick_timers(&mut s);
        assert_eq!(s.units[1].status, Status::None);
        assert!(s.units[1].fx.queued_slow.is_none());
    }
    #[test]
    fn r19_queue_follows_true_expiry_but_cleanse_or_new_immunity_discards_it() {
        let mut s = arena();
        let a = s.units[0];
        let id = s.units[1].id;
        resolve(&mut s, &a, id, &mut Vec::new());
        for _ in 0..30 {
            combat::tick_timers(&mut s);
        }
        assert_eq!(
            s.units[1].status,
            Status::Slowed {
                bp: 1500,
                ticks_left: 40
            }
        );
        resolve(&mut s, &a, id, &mut Vec::new());
        let brewer = s.lab_spawn_at(Seat::B, UnitKind::AntidoteBrewer, 0, 0, 7500);
        let b = s.units[s.unit_index(brewer).unwrap()];
        crate::afflictions::brew_cast(&mut s, &b, 3000, id, 1, 0, false, &mut Vec::new());
        assert!(s.units[1].fx.queued_slow.is_none());
        resolve(&mut s, &a, id, &mut Vec::new());
        s.units[1].kind = UnitKind::Oilpress;
        s.units[1].tier = 2;
        for _ in 0..30 {
            combat::tick_timers(&mut s);
        }
        assert_eq!(s.units[1].status, Status::None);
    }
    #[test]
    fn r19_dead_primary_cancels_web_and_ward_blocks_only_control() {
        let mut s = arena();
        let a = s.units[0];
        let id = s.units[1].id;
        let hp = s.units[1].hp;
        s.units[1].fx.ward_until = 50;
        resolve(&mut s, &a, id, &mut Vec::new());
        assert_eq!(s.units[1].hp, hp - 25);
        assert!(s.units[1].fx.queued_slow.is_none());
        s.units[0].action_cd = 0;
        specials::advance(&mut s, 0, &mut Vec::new());
        s.units[1].hp = 0;
        let mut ev = Vec::new();
        specials::advance(&mut s, 0, &mut ev);
        assert!(s.units[0].fx.special.is_none());
        assert!(ev.is_empty());
    }
}
