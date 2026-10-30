//! Reproducible artificial army populations; setup/top-up work stays
//! outside the timed tick. No production rules or army caps are changed.

use tbg_core::{ArcanaPreset, MatchState, Phase, PlayerIntention, Seat, StrongholdKind, UnitKind};

pub struct Fixture {
    pub state: MatchState,
    pub scene: String,
    pub target: usize,
    next: [usize; 2],
}

impl Fixture {
    pub fn new(scene: &str, target: usize, seed: u64) -> Self {
        let mut state = MatchState::with_arcanas(seed, [ArcanaPreset::DEFAULT; 2])
            .with_strongholds([StrongholdKind::Artillery, StrongholdKind::Command]);
        while state.phase != Phase::Running {
            state.step();
        }
        if matches!(scene, "mixed" | "cargo") {
            for p in &mut state.players {
                // Synthetic late-game ownership; values are existing copies,
                // not new balance values or a claim about normal matches.
                for id in [0, 3, 7, 14, 20, 23, 30] {
                    p.augments[id] = 1;
                }
            }
        }
        let mut fixture = Self {
            state,
            scene: scene.to_owned(),
            target,
            next: [seed as usize; 2],
        };
        fixture.prepare();
        fixture
    }

    pub fn prepare(&mut self) {
        assert!(self.state.outcome.is_none(), "synthetic fixture ended");
        for seat in Seat::BOTH {
            self.state.players[seat.index()].stronghold_hp = 1_000_000;
            if self.state.augments.selecting() {
                let round = self.state.augments.round;
                let _ = self
                    .state
                    .apply_intention(seat, 0, PlayerIntention::Forgo { round });
            }
            let missing = (self.target / 2).saturating_sub(self.state.army_size(seat));
            for _ in 0..missing {
                self.purchase(seat);
            }
        }
    }

    fn purchase(&mut self, seat: Seat) {
        let index = self.next[seat.index()];
        self.next[seat.index()] += 1;
        let pool = kinds(&self.scene);
        let kind = pool[index % pool.len()];
        let p = &mut self.state.players[seat.index()];
        p.energy = 1_000_000;
        p.offers[0] = kind;
        self.state
            .apply_intention(seat, 0, PlayerIntention::Purchase { offer: 0 })
            .unwrap();
        let id = self.state.units.last().unwrap().id;
        if index.is_multiple_of(3) || self.scene == "formation" {
            for _ in 0..2 {
                self.state
                    .apply_intention(
                        seat,
                        0,
                        PlayerIntention::Evolve {
                            unit: id,
                            branch: (index % 2) as u8,
                        },
                    )
                    .unwrap();
            }
        }
        let unit = self.state.units.last_mut().unwrap();
        // Deliberately separated lines make formation engagement scans
        // adversarial; other scenes start in overlapping combat range.
        let center = if self.scene == "formation" {
            if seat == Seat::A {
                3_000
            } else {
                13_000
            }
        } else {
            7_500 + seat.index() as i32 * 800
        };
        unit.x = center + (index % 11) as i32 * 50;
        if self.scene == "healing" || self.scene == "mixed" {
            unit.hp = (unit.max_hp * 2 / 3).max(1);
        }
    }
}

fn kinds(scene: &str) -> &'static [UnitKind] {
    use UnitKind::*;
    match scene {
        "melee" => &[Shieldbearer, BoneBrute, Sawbot, Rampart],
        "ranged" => &[Archer, SiegeWalker, Hexer, Farwatch, Bombard],
        "healing" => &[Bloomkeeper, Bloodweaver, AntidoteBrewer, Rootcaller, Rotcap],
        "formation" => &[Oilpress, Oilpress, Shieldbearer, Drumguard],
        "puppet" => &[Puppetmaster, Puppetmaster, Longaxe, BloodFurnace],
        "cargo" => &[CargoCrab, Smuggler, Nester, Leechroot, Archer],
        "mixed" => &tbg_core::types::FULL_POOL,
        _ => panic!("unknown benchmark scene: {scene}"),
    }
}
