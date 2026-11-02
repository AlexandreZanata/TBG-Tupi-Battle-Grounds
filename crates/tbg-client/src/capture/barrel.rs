//! Opt-in native witness of rendered rolling casks, reload visibility and real area-heal events.
use bevy::prelude::*;
#[derive(Resource, Default)]
pub(super) struct Status {
    pub ready: bool,
    flight_frames: u32,
    empty: bool,
    burst: bool,
    area: bool,
    intercepted: bool,
    paused: Option<(u32, u32, Vec<tbg_core::barrel::Barrel>)>,
}
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Status>().add_systems(
        PostUpdate,
        observe
            .after(crate::art::anim::animate)
            .after(bevy::transform::TransformSystems::Propagate),
    );
}
fn observe(
    latest: Res<crate::sync::LatestView>,
    trial: Res<crate::training::Trial>,
    rolling: Res<crate::vfx::barrels::Entities>,
    units: Query<(&crate::visuals::UnitVisual, &crate::art::anim::Animator)>,
    visibility: Query<&Visibility>,
    mut status: ResMut<Status>,
) {
    let Some(view) = &latest.0 else { return };
    if !view.barrels.is_empty() && rolling.len() > 0 {
        status.flight_frames += 1;
    }
    for (v, a) in &units {
        if v.kind() != tbg_core::UnitKind::GustDrummer || v.dying {
            continue;
        }
        let reload = a.completion.map_or(0, |f| f.barrel_ready_in);
        let joint = a.joints[crate::art::rig::Node::Prop(crate::art::rig::Socket::WeaponR).index()];
        if let Ok(shown) = visibility.get(joint) {
            if reload > 0 {
                assert_eq!(
                    *shown,
                    Visibility::Hidden,
                    "OG-02: carried cask stays visible during authoritative reload"
                );
                status.empty = true;
            }
        }
    }
    let burst = view.events.iter().find_map(|e| match e {
        tbg_core::Event::BarrelBurst {
            seat,
            contact,
            center,
            radius,
            ..
        } => Some((*seat, *contact, *center, *radius)),
        _ => None,
    });
    if let Some((seat, contact, center, radius)) = burst {
        status.burst = true;
        status.intercepted |= view.units.iter().any(|u| u.id == contact && u.seat != seat);
        status.area |= view
            .units
            .iter()
            .filter(|u| u.seat == seat && (u.x - center).abs() <= radius)
            .count()
            >= 2
            && view
                .events
                .iter()
                .any(|e| matches!(e,tbg_core::Event::Heal{amount,..} if *amount>0));
    }
    if let Some(s) = &trial.session {
        if matches!(s.lifecycle(), tbg_core::training::Lifecycle::Paused(_))
            && !s.state().barrels.is_empty()
        {
            let now = (s.id().0, s.active_ticks(), s.state().barrels.clone());
            if let Some((id, tick, before)) = &status.paused {
                if (*id, *tick) == (now.0, now.1) {
                    assert_eq!(*before, now.2, "OG-02: paused cask moved");
                }
            }
            status.paused = Some(now);
        }
    }
    let need_intercept = crate::config::url_param("capture_barrel_intercept").is_some();
    if !status.ready
        && status.flight_frames >= 3
        && status.empty
        && status.burst
        && status.area
        && status.paused.is_some()
        && (!need_intercept || status.intercepted)
    {
        status.ready = true;
        eprintln!("Native rolling-barrel witness passed: rendered_frames={}, reload_hidden=true, area_heal=true, paused=true, intercepted={}",status.flight_frames,status.intercepted);
    }
}
