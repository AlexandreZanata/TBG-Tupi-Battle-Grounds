//! AB-01: every native marine/Plant form has distinct finite geometry and clips.
use super::*;
use std::collections::BTreeSet;
#[test]
fn abyssal_all_45_forms_are_distinct_finite_and_within_mesh_budget() {
    let mut count = 0;
    for kind in tbg_core::COMPLETION_POOL
        .into_iter()
        .filter(|k| kits::abyssals::contains(*k))
    {
        let kit = kits::kit(kind).unwrap();
        let mut signatures = BTreeSet::new();
        for (t, b) in FORMS {
            let spec = (kit.figure)(t, b);
            let mut signature = spec
                .parts
                .iter()
                .map(|p| {
                    format!(
                        "{:?}/{:?}/{:?}/{:?}",
                        p.node.index(),
                        p.shape,
                        p.at,
                        p.color
                    )
                })
                .collect::<Vec<_>>();
            signature.sort();
            assert!(signatures.insert(signature), "duplicate {kind:?} {t}/{b}");
            let mut meshes = Assets::<Mesh>::default();
            let form = native_form(spec, &mut meshes);
            assert!(form.triangles <= 6000, "{kind:?}: {}", form.triangles);
            let mut anim = anim::Animator::new(
                [Entity::PLACEHOLDER; NODE_COUNT],
                form.plan,
                Entity::PLACEHOLDER,
                kind,
                1,
            );
            anim.form = (t, b);
            for clip in [
                anim::Clip::Idle,
                anim::Clip::Walk,
                anim::Clip::AttackMelee,
                anim::Clip::AttackRanged,
                anim::Clip::Cast,
                anim::Clip::Hit,
                anim::Clip::Death,
                anim::Clip::Spawn,
                anim::Clip::Evolve,
                anim::Clip::Victory,
            ] {
                for sample in 0..12 {
                    let pose = anim.pose(clip, sample as f32 * kit.duration(clip) / 12.0);
                    for matrix in presentation::pose_world(kind, &form.plan, &pose) {
                        assert!(matrix.is_finite(), "{kind:?} {clip:?}");
                    }
                }
            }
            count += 1;
        }
    }
    assert_eq!(count, 45);
}
