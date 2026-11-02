//! GD-01: native Colossus forms, finite life/action poses and bounded geometry.
use super::*;
use std::collections::BTreeSet;
#[test]
fn gd01_colossus_has_five_distinct_finite_forms_with_one_obelisk() {
    let kit = kits::kit(UnitKind::Colossus).unwrap();
    let mut seen = BTreeSet::new();
    for (t, b) in FORMS {
        let spec = (kit.figure)(t, b);
        assert!(seen.insert(format!(
            "{:?}",
            spec.parts
                .iter()
                .map(|v| (v.node.index(), v.at, format!("{:?}", v.color)))
                .collect::<Vec<_>>()
        )));
        assert!(
            spec.parts
                .iter()
                .filter(|v| matches!(v.node, rig::Node::Prop(rig::Socket::WeaponR)))
                .count()
                > 10
        );
        assert!(!spec
            .parts
            .iter()
            .any(|v| matches!(v.node, rig::Node::Prop(rig::Socket::Back))));
        let mut meshes = Assets::<Mesh>::default();
        let form = native_form(spec, &mut meshes);
        assert!(form.triangles <= 6000);
        let mut actor = anim::Animator::new(
            [Entity::PLACEHOLDER; NODE_COUNT],
            form.plan,
            Entity::PLACEHOLDER,
            UnitKind::Colossus,
            1,
        );
        actor.form = (t, b);
        for clip in [
            anim::Clip::Idle,
            anim::Clip::Walk,
            anim::Clip::AttackMelee,
            anim::Clip::Spawn,
            anim::Clip::Evolve,
            anim::Clip::Death,
            anim::Clip::Victory,
        ] {
            for step in 0..16 {
                let pose = actor.pose(clip, step as f32 * actor.duration(clip) / 16.0);
                assert!(
                    presentation::pose_world(UnitKind::Colossus, &form.plan, &pose)
                        .iter()
                        .all(|m| m.is_finite())
                );
            }
        }
    }
}
