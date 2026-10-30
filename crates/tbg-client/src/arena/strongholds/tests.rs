//! Stronghold look contracts: damage states from the kind's own max HP
//! (BR-SK07 keeps the ratio across tiers), distinct forms, damage that
//! never empties a tower, Artillery cadence, and the War Banner radius.

use std::collections::BTreeSet;

use super::parts::{build, Form};
use super::{artillery, aura, damage_level, pieces, visible, Look};
use crate::art::mesh;
use tbg_core::{StrongholdKind, StrongholdView};

const FORMS: [Form; 5] = [Form::Base, Form::A1, Form::A2, Form::B1, Form::B2];

fn names(look: Look) -> BTreeSet<&'static str> {
    visible(look).map(|p| p.def.name).collect()
}

fn look(kind: StrongholdKind, form: Form, damage: u8) -> Look {
    let tier = form.tier();
    let branch = u8::from(form.b());
    Look {
        kind,
        tier,
        branch,
        damage,
    }
}

#[test]
fn damage_states_follow_the_kinds_own_max_hp() {
    assert_eq!(damage_level(4_500, 4_500), 0);
    assert_eq!(damage_level(3_001, 4_500), 0);
    assert_eq!(
        damage_level(3_000, 4_500),
        1,
        "exactly 66 % hides the crown"
    );
    assert_eq!(damage_level(1_501, 4_500), 1);
    assert_eq!(
        damage_level(1_500, 4_500),
        2,
        "exactly 33 % hides the middle"
    );
    assert_eq!(damage_level(0, 4_500), 2);
    // A Bastion at 3 400 / 5 175 is under 66 % of its own max, though it
    // would be above 66 % of the reference 4 500.
    let bastion = StrongholdView {
        kind: StrongholdKind::Bastion,
        tier: 0,
        branch: 1,
        max_hp: 5_175,
    };
    let seen = Look::from_view(&bastion, 3_400);
    assert_eq!(seen.damage, 1);
    assert_eq!(seen.branch, 0, "no branch is chosen before tier 1");
}

#[test]
fn every_form_has_its_own_silhouette() {
    for kind in StrongholdKind::SELECTABLE {
        let forms: Vec<_> = FORMS.iter().map(|f| names(look(kind, *f, 0))).collect();
        for (i, a) in forms.iter().enumerate() {
            for b in &forms[i + 1..] {
                assert_ne!(a, b, "{kind:?}: two forms share every part");
            }
        }
        // Tier 1 and tier 2 each add a part the tier before lacks.
        for (before, after) in [(0, 1), (1, 2), (0, 3), (3, 4)] {
            assert!(
                !forms[after].is_subset(&forms[before]),
                "{kind:?}: {:?} adds nothing over {:?}",
                FORMS[after],
                FORMS[before]
            );
        }
    }
}

#[test]
fn damage_only_removes_pieces_and_keeps_the_kind() {
    for kind in [StrongholdKind::Classic]
        .into_iter()
        .chain(StrongholdKind::SELECTABLE)
    {
        for form in FORMS {
            let healthy = names(look(kind, form, 0));
            let hurt = names(look(kind, form, 1));
            let dying = names(look(kind, form, 2));
            assert!(hurt.is_subset(&healthy) && dying.is_subset(&hurt));
            assert!(
                dying.len() < healthy.len(),
                "{kind:?} {form:?} never shows damage"
            );
            let base = pieces(kind, Form::Base)[0].def.name;
            assert!(dying.contains(base), "{kind:?} {form:?} loses its base");
        }
    }
}

#[test]
fn artillery_recoil_matches_the_authoritative_cadence() {
    let interval = |f| artillery::gun(f).interval;
    assert_eq!(interval(Form::Base), 1.0, "20 ticks at 20 ticks/s");
    assert_eq!(interval(Form::B1), 0.85, "Quick Reload: 17 ticks");
    assert_eq!(interval(Form::B2), 0.7, "Advanced Mechanism: 14 ticks");
    for form in FORMS {
        let gun = artillery::gun(form);
        assert!(
            gun.recover < gun.interval,
            "{form:?} kicks again before settling"
        );
    }
    let kick = |f| artillery::gun(f).kick;
    assert!(kick(Form::A2) > kick(Form::A1) && kick(Form::A1) > kick(Form::Base));
    assert!(kick(Form::B2) < kick(Form::Base));
}

#[test]
fn war_banner_radius_is_the_authoritative_one() {
    let tiles: Vec<i32> = FORMS
        .iter()
        .map(|f| aura::radius_milli(*f) / 1000)
        .collect();
    assert_eq!(tiles, [4, 4, 4, 5, 6]);
}

/// Prints the triangle count of every form (`cargo test -- --nocapture`)
/// and keeps each under a budget a phone can draw twice per frame.
#[test]
fn every_form_stays_within_its_triangle_budget() {
    for kind in [StrongholdKind::Classic]
        .into_iter()
        .chain(StrongholdKind::SELECTABLE)
    {
        let counts: Vec<usize> = FORMS
            .iter()
            .map(|f| {
                visible(look(kind, *f, 0))
                    .map(|p| mesh::triangles(&build(p.def, true)))
                    .sum()
            })
            .collect();
        println!("{kind:?} triangles (0, A1, A2, B1, B2): {counts:?}");
        assert!(counts.iter().all(|c| *c <= 3_000), "{kind:?}: {counts:?}");
    }
}
