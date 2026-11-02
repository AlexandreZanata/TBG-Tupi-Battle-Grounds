//! TW-01/GD-01 bounded defensive skirmishes and seat-swapped Colossus duels; no full gate.
use tbg_balance::duels::{self, Form};
use tbg_core::{MatchState, Phase, Ruleset, Seat, StrongholdKind, UnitKind};
fn main() {
    println!("section,kind,tier,branch,seat,opponent,ticks,hp,remaining,queued,score");
    if !std::env::args().any(|a| a == "--duels-only") {
        for kind in StrongholdKind::SELECTABLE {
            for (tier, branch) in [(0, 0), (1, 0), (2, 0), (1, 1), (2, 1)] {
                for seat in Seat::BOTH {
                    for enemy in [UnitKind::Archer, UnitKind::Longaxe] {
                        let mut kinds = [StrongholdKind::Classic; 2];
                        kinds[seat.index()] = kind;
                        let mut s = MatchState::new(91).with_strongholds(kinds);
                        s.players[seat.index()].fort_tier = tier;
                        s.players[seat.index()].fort_branch = branch;
                        s.players[seat.index()].stronghold_hp = s.stronghold(seat).max_hp;
                        s.phase = Phase::Running;
                        s.tick = tbg_core::content::RUNNING_START;
                        for i in 0..6 {
                            let x = 4000 + i * 150;
                            let x = if seat == Seat::A { x } else { 16000 - x };
                            s.lab_spawn_at(seat.other(), enemy, 0, 0, x);
                        }
                        // TW-01 measures canonical tower changes; PK-01 remains a private candidate.
                        s.physical_lanes = false;
                        let mut ticks = 0;
                        while ticks < 1000
                            && s.phase != Phase::Ended
                            && s.army_size(seat.other())
                                + s.deployments
                                    .iter()
                                    .filter(|u| u.seat == seat.other())
                                    .count()
                                > 0
                        {
                            s.step();
                            ticks += 1;
                        }
                        println!(
                            "defense,{kind:?},{tier},{branch},{seat:?},{enemy:?},{ticks},{},{},{},",
                            s.players[seat.index()].stronghold_hp,
                            s.army_size(seat.other()),
                            s.deployments.len()
                        );
                    }
                }
            }
        }
    }
    let rules = duels::arena(Ruleset::canon());
    for opponent in [
        UnitKind::SiegeWalker,
        UnitKind::Archer,
        UnitKind::Longaxe,
        UnitKind::Trident,
    ] {
        let a = Form {
            kind: UnitKind::Colossus,
            tier: 0,
            branch: 0,
        };
        let b = Form {
            kind: opponent,
            tier: 0,
            branch: 0,
        };
        for swapped in [false, true] {
            let score = if swapped {
                1.0 - duels::preview_duel(rules, b, a, 18).unwrap()
            } else {
                duels::preview_duel(rules, a, b, 18).unwrap()
            };
            println!(
                "duel,Colossus,0,0,{}, {opponent:?},,,,,{score}",
                if swapped { "B" } else { "A" }
            );
        }
    }
}
