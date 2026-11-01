//! Owner's ring-only contract covers every kind, Evolution and retained fallback.
use super::*;

fn no_ownership_paint(spec: &figure::FigureSpec, label: &str) {
    for part in &spec.parts {
        assert!(
            part.color != palette::ALLY && part.color != palette::ENEMY,
            "{label}: ownership paint on {:?}; use the floor ring",
            part.node
        );
    }
}

#[test]
fn ring_only_all_340_forms_share_identical_body_meshes_for_both_seats() {
    let mut meshes = Assets::<Mesh>::default();
    for kind in ALL_KINDS {
        let kit = kits::kit(kind).expect("every registered kind has an authored kit");
        for (tier, branch) in FORMS {
            let spec = (kit.figure)(tier, branch);
            no_ownership_paint(&spec, &format!("{kind:?} {tier}/{branch}"));
            let form = native_form(spec, &mut meshes);
            assert!(std::ptr::eq(form.meshes(true), form.meshes(false)));
        }
    }
    for colors in palette::FACTIONS {
        assert!(!colors.contains(&palette::ALLY) && !colors.contains(&palette::ENEMY));
    }
    for (tier, branch) in FORMS {
        no_ownership_paint(
            &roster::humans::shieldbearer_v2::spec(tier, branch),
            "fallback",
        );
    }
}
