//! Bounded reproducible decision inspection, without graphics or a server.
use tbg_moba_core::{host, MobaMatch};
fn main() {
    let mut game = MobaMatch::new(11);
    let mut inputs = Vec::new();
    for _ in 0..1801 {
        host::tick(&mut game, &mut inputs);
    }
    println!("seed=11 tick={} hash={:016x}", game.tick, game.state_hash());
    for u in game
        .units
        .iter()
        .filter(|u| matches!(u.kind, tbg_moba_core::UnitKind::Troop(_)))
    {
        println!(
            "id={} kind={:?} at={:?} mission={:?} phase={:?} target={} reason={:?}",
            u.id, u.kind, u.pos, u.brain, u.decision.phase, u.target, u.decision.reason
        );
    }
}
