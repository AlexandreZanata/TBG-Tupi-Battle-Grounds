//! Guards for the default Archer v2: each form fits the one-slot triangle
//! budget (ARCHITECTURE §10), and the side-on draw is reachable — the bow arm
//! nearly straight, the string arm bent, neither stretched past its length.

use super::{
    archer_v2, archer_v2_clips, figure, mesh,
    rig::{self, Joint, Node},
    roster,
};

/// Shared bow strings and nocked arrow (`BowAssets::triangles` for P15).
const BOW_EXTRA: usize = 74;

#[test]
fn every_candidate_form_fits_the_one_slot_budget() {
    for tier in 0..3 {
        let spec = roster::archer_v2(tier);
        let figure: usize = figure::build_joint_meshes(&spec)
            .iter()
            .flatten()
            .map(mesh::triangles)
            .sum();
        println!(
            "Archer v2 tier {tier}: {figure} + {BOW_EXTRA} = {} triangles",
            figure + BOW_EXTRA
        );
        assert!(
            figure + BOW_EXTRA <= 1_500,
            "tier {tier}: {} triangles",
            figure + BOW_EXTRA
        );
    }
}

#[test]
fn full_draw_is_within_both_arms_reach() {
    let p = archer_v2::plan();
    let reach = p.arm * 1.9;
    let (anchor, bow) = archer_v2_clips::full_draw();
    let string_arm = (anchor - rig::rest(Node::Body(Joint::UpperArmR), &p)).length();
    let bow_arm = (bow - rig::rest(Node::Body(Joint::UpperArmL), &p)).length();
    assert!(
        string_arm < reach * 0.92,
        "string arm {string_arm} of {reach}"
    );
    assert!(bow_arm < reach, "bow arm {bow_arm} of {reach}");
    assert!(
        bow_arm > reach * 0.9,
        "bow arm not extended: {bow_arm} of {reach}"
    );
}
