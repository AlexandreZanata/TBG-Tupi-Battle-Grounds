//! Readable live effect durations from the authoritative public snapshot.

use tbg_core::{
    content::TICK_HZ,
    view::{DotView, EffectsView},
    MatchView, UnitView,
};
fn seconds(t: u16) -> String {
    format!("{:.1}s", f32::from(t) / TICK_HZ as f32)
}

pub(crate) fn describe(v: &MatchView, u: &UnitView) -> String {
    let mut parts = Vec::new();
    let marked = v
        .units
        .iter()
        .filter(|a| a.kind == tbg_core::UnitKind::Crowcaller)
        .filter_map(|a| a.effects.and_then(|e| e.mark))
        .filter(|m| m.target == u.id)
        .map(|m| m.ticks)
        .max();
    if let Some(t) = marked {
        parts.push(format!("Marked {}", seconds(t)));
    }
    if let Some(e) = u.effects {
        control(&mut parts, u, &e);
        health(&mut parts, u, &e);
        damage(&mut parts, &e);
        if let Some(m) = e.mark {
            let name = v
                .units
                .iter()
                .find(|a| a.id == m.target)
                .map(|a| tbg_core::content::def(a.kind).name)
                .unwrap_or("target");
            parts.push(if u.kind == tbg_core::UnitKind::Mantis {
                format!("Focus on {name}")
            } else {
                format!("Mark on {name} {}", seconds(m.ticks))
            });
        }
    }
    if let Some(s) = u.special {
        let text = match s.cast {
            Some(c) => format!("Casting {}", seconds(c.left)),
            None if s.ready_in == 0 => "Ability ready".into(),
            None => format!("Ability in {}", seconds(s.ready_in)),
        };
        parts.push(text);
    }
    parts.join("; ")
}
fn control(parts: &mut Vec<String>, u: &UnitView, e: &EffectsView) {
    if e.control_ticks > 0 {
        parts.push(if u.status == 2 {
            format!("Root {}", seconds(e.control_ticks))
        } else {
            format!("Slow {}% {}", e.slow_bp / 100, seconds(e.control_ticks))
        });
    }
    if e.ward_ticks > 0 {
        parts.push(format!("Ward {}", seconds(e.ward_ticks)));
    }
    if let Some(q) = e.queued_slow {
        parts.push(format!("Then Slow {}% {}", q.bp / 100, seconds(q.ticks)));
    }
}
fn health(parts: &mut Vec<String>, u: &UnitView, e: &EffectsView) {
    if u.shield > 0 {
        parts.push(format!("Shield {} ({})", u.shield, seconds(e.shield_ticks)));
    }
    if u.reduction > 0 {
        parts.push(format!(
            "Damage reduction {}% ({})",
            u.reduction / 100,
            seconds(e.reduction_ticks)
        ));
    }
    if let Some(c) = e.heal_cut {
        parts.push(format!("Healing -{}% ({})", c.bp / 100, seconds(c.ticks)));
    }
    if e.life > 0 {
        parts.push(format!("Expires in {}", seconds(e.life)));
    }
}
fn damage(parts: &mut Vec<String>, e: &EffectsView) {
    if let Some(w) = e.weaken {
        parts.push(format!(
            "Basic damage -{}% ({})",
            w.bp / 100,
            seconds(w.ticks)
        ));
    }
    for (name, dot) in [("Bleed", e.bleed), ("Venom", e.venom)] {
        if let Some(DotView {
            value,
            pulses,
            next_in,
        }) = dot
        {
            parts.push(format!(
                "{name} {value} ×{pulses}, next {}",
                seconds(next_in)
            ));
        }
    }
}
