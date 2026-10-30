//! Placeholder readability: every kind's figure differs from every other
//! kind's (a recolor that matches no part color silently clones its source).

use super::roster::figure_spec;
use tbg_core::ALL_KINDS;

/// The sorted part colors and shapes of a base form.
fn signature(kind: tbg_core::UnitKind) -> Vec<String> {
    let mut v: Vec<String> = figure_spec(kind, 0, 0)
        .parts
        .iter()
        .map(|p| format!("{:?}/{:?}", p.color.to_srgba(), p.shape))
        .collect();
    v.sort();
    v
}

#[test]
fn every_kind_has_its_own_figure() {
    let mut clones = Vec::new();
    for (i, a) in ALL_KINDS.iter().enumerate() {
        for b in &ALL_KINDS[i + 1..] {
            if signature(*a) == signature(*b) {
                clones.push(format!("{a:?} = {b:?}"));
            }
        }
    }
    assert!(clones.is_empty(), "identical figures: {clones:?}");
}
