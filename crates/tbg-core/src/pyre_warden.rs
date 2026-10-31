//! R20's basic flame replaces output with one exact forward segment, including allies.
use crate::{
    content::{Ability, Stats},
    hits, roster_slots,
    state::{Event, MatchState, Target, Unit},
    types::UnitKind,
};
pub(crate) fn resolve(
    s: &mut MatchState,
    a: &Unit,
    target: Target,
    st: &Stats,
    ev: &mut Vec<Event>,
) -> bool {
    if a.kind != UnitKind::PyreWarden || a.is_summon() {
        return false;
    }
    let Some((bp, ticks)) = s
        .rules()
        .def(a.kind)
        .ability(a.tier, a.branch)
        .into_iter()
        .find_map(|v| match v {
            Ability::ProfaneFire { cut_bp, cut_ticks } => Some((cut_bp, cut_ticks)),
            _ => None,
        })
    else {
        return false;
    };
    let amount = hits::scale(st.damage, -roster_slots::weaken_now(&a.fx, s.tick));
    let end = a.x.saturating_add(a.seat.dir() * st.range);
    ev.push(Event::ProfaneFlame {
        unit: a.id,
        start: a.x,
        end,
        tick: s.tick,
    });
    burn(s, a, st.range, amount, bp, ticks, ev);
    if let Target::Stronghold(seat) = target {
        hits::damage(s, Target::Stronghold(seat), amount, ev);
    }
    true
}
fn in_segment(a: &Unit, v: &Unit, range: i32) -> bool {
    let dx = (v.x - a.x) * a.seat.dir();
    v.id != a.id && v.hp > 0 && (0..=range).contains(&dx)
}
fn burn(
    s: &mut MatchState,
    a: &Unit,
    range: i32,
    amount: i32,
    bp: u16,
    ticks: u16,
    ev: &mut Vec<Event>,
) {
    for i in 0..s.units.len() {
        let v = s.units[i];
        if !in_segment(a, &v, range) {
            continue;
        }
        let _removed = hits::damage(s, Target::Unit(v.id), amount, ev);
        #[cfg(feature = "lab")]
        if v.seat != a.seat {
            crate::lab::dealt(s, a, _removed);
        }
        if s.units[i].hp > 0 && s.tick >= s.units[i].fx.ward_until {
            roster_slots::apply_ability_cut(&mut s.units[i].fx, bp, ticks, s.tick);
        }
    }
}
#[cfg(all(test, feature = "lab"))]
mod tests {
    use super::*;
    use crate::{combat, ArcanaPreset, Seat, Status};
    fn arena(seat: Seat) -> MatchState {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        s.lab_spawn_at(seat, UnitKind::PyreWarden, 0, 0, 8000);
        for (owner, dx) in [
            (seat, 0),
            (seat, 2800),
            (seat.other(), 1400),
            (seat.other(), 2801),
            (seat, -1),
        ] {
            s.lab_spawn_at(owner, UnitKind::BoneBrute, 0, 0, 8000 + seat.dir() * dx);
        }
        s
    }
    #[test]
    fn r20_exact_forward_segment_in_both_directions_hits_allies_equally_and_excludes_self() {
        for seat in Seat::BOTH {
            let mut s = arena(seat);
            let a = s.units[0];
            let st = s.stats_of(&a);
            let hp: Vec<_> = s.units.iter().map(|u| u.hp).collect();
            let mut ev = Vec::new();
            let target = s.units[3].id;
            hits::resolve_hit(&mut s, &a, Target::Unit(target), &st, &mut ev);
            for (i, u) in s.units.iter().enumerate() {
                assert_eq!(u.hp, hp[i] - if (1..=3).contains(&i) { 12 } else { 0 });
            }
            assert_eq!(
                ev.iter().filter(|e| matches!(e, Event::Hit { .. })).count(),
                3
            );
            assert_eq!(s.units[1].fx.ability_cut_bp, 3500);
            assert_eq!(s.units[2].fx.ability_cut_bp, 3500);
            assert!(ev.contains(&Event::ProfaneFlame {
                unit: a.id,
                start: 8000,
                end: 8000 + seat.dir() * 2800,
                tick: 0
            }));
        }
    }
    #[test]
    fn r20_ward_blocks_cut_not_flame_and_shields_do_not_block_surviving_victim_cut() {
        let mut s = arena(Seat::A);
        s.units[1].fx.ward_until = 50;
        roster_slots::apply_shield(&mut s.units[2].fx, 100, 50, 0);
        let a = s.units[0];
        let st = s.stats_of(&a);
        let target = s.units[3].id;
        resolve(&mut s, &a, Target::Unit(target), &st, &mut Vec::new());
        assert_eq!(s.units[1].fx.ability_cut_bp, 0);
        assert_eq!(s.units[2].fx.ability_cut_bp, 3500);
        assert_eq!(s.units[2].fx.shield, 88);
    }
    #[test]
    fn r20_evolutions_keep_friendly_fire_and_tower_takes_only_one_basic_hit() {
        let mut s = arena(Seat::A);
        s.units[0].tier = 2;
        s.units[0].branch = 1;
        let a = s.units[0];
        let st = s.stats_of(&a);
        let hp = s.players[1].stronghold_hp;
        let ally = s.units[1].hp;
        let mut ev = Vec::new();
        hits::resolve_hit(&mut s, &a, Target::Stronghold(Seat::B), &st, &mut ev);
        assert_eq!(s.units[1].hp, ally - 14);
        assert_eq!(s.units[1].fx.ability_cut_bp, 5000);
        assert_eq!(s.players[1].stronghold_hp, hp - 14);
        assert_eq!(
            ev.iter()
                .filter(|e| matches!(
                    e,
                    Event::Hit {
                        to: Target::Stronghold(_),
                        ..
                    }
                ))
                .count(),
            1
        );
        let st = s.rules().effective_stats(UnitKind::PyreWarden, 2, 0, 0);
        assert_eq!((st.max_hp, st.range), (1000, 3360));
    }
    #[test]
    fn r20_root_cancels_flame_and_friendly_death_has_no_caster_reward() {
        let mut s = arena(Seat::A);
        let id = s.units[3].id;
        s.units[0].windup = Some((Target::Unit(id), 1));
        s.units[0].status = Status::Rooted { ticks_left: 2 };
        let hp = s.units[1].hp;
        combat::run_units(&mut s, &mut Vec::new());
        assert_eq!(s.units[1].hp, hp);
        s.units[1].hp = 1;
        let a = s.units[0];
        let st = s.stats_of(&a);
        let before = s.players[0].energy;
        resolve(&mut s, &a, Target::Unit(id), &st, &mut Vec::new());
        combat::remove_dead(&mut s, &mut Vec::new());
        assert_eq!(s.players[0].energy, before);
    }
}
