//! Opt-in offline decision traces; never changes simulation state or online frames.
use std::{
    fs::File,
    io::{BufWriter, Write},
};
use tbg_moba_core::MobaMatch;
pub(super) struct Probe(BufWriter<File>);
impl Probe {
    pub fn open() -> Option<Self> {
        let path = crate::config::url_param("decision-log")?;
        match File::create(path) {
            Ok(file) => Some(Self(BufWriter::new(file))),
            Err(error) => {
                bevy::log::warn!("Decision log unavailable: {error}");
                None
            }
        }
    }
    pub fn record(&mut self, g: &MobaMatch, micros: u128) {
        let _ = writeln!(self.0, "tick,{},{micros},{}", g.tick, g.units.len());
        for u in &g.units {
            let _ = writeln!(
                self.0,
                "unit,{},{},{:?},{:?},{},{},{},{:?},{:?},{},{},{},\"{:?}\"",
                g.tick,
                u.id,
                u.kind,
                u.side,
                u.pos.x,
                u.pos.y,
                u.target,
                u.decision.phase,
                if u.decision.since == g.tick {
                    u.decision.changed_because
                } else {
                    u.decision.reason
                },
                u.decision.since,
                u.last_attack,
                u.hp,
                u.brain
            );
        }
        let _ = self.0.flush();
    }
}
