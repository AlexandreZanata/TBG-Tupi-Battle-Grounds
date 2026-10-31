//! P64 T01/T02/T08: every active content id has paired presentation, the
//! English rendering equals the `tbg-core` canon (P41 `i18n_matches_core_canon`)
//! and no Portuguese template keeps an unfilled placeholder.

use tbg_core::content::{def, AUGMENTS, STRONGHOLD_KINDS};
use tbg_core::{ALL_KINDS, ARCHETYPES, FACTIONS};

use super::{augment, content, Lang};

fn filled(text: &str) -> bool {
    !text.is_empty() && !text.contains('{') && !text.contains('}')
}

#[test]
fn i18n_content_coverage_units_and_forms_match_core() {
    for kind in ALL_KINDS {
        let d = def(kind);
        assert_eq!(content::unit_name(Lang::En, kind), d.name, "{kind:?}");
        assert!(filled(content::unit_name(Lang::PtBr, kind)), "{kind:?}");
        for (slot, form) in d.forms.iter().enumerate() {
            let (tier, branch) = ((slot % 2 + 1) as u8, (slot / 2) as u8);
            let en = content::form_name(Lang::En, kind, tier, branch);
            assert_eq!(en, Some(form.name), "{kind:?} form {slot}");
            let pt = content::form_name(Lang::PtBr, kind, tier, branch).unwrap();
            assert!(filled(pt), "{kind:?} form {slot}");
        }
        assert_eq!(content::form_name(Lang::En, kind, 0, 0), None);
        let (good, weak) = content::matchup(Lang::PtBr, kind);
        assert!(filled(good) && filled(weak), "{kind:?}");
    }
}

#[test]
fn i18n_content_coverage_augments_render_core_text() {
    for a in AUGMENTS {
        assert_eq!(content::augment_name(Lang::En, a.id), Some(a.name));
        assert_eq!(augment::text(Lang::En, &a), a.text, "augment {}", a.id);
        let pt = augment::text(Lang::PtBr, &a);
        assert!(filled(pt.as_str()), "augment {}: {pt}", a.id);
        assert!(filled(content::augment_name(Lang::PtBr, a.id).unwrap()));
    }
    // Decimal Energy follows the language (Plunder: 0.15 per kill).
    assert!(augment::text(Lang::PtBr, &AUGMENTS[10]).contains("0,15"));
}

#[test]
fn i18n_content_coverage_strongholds_match_core() {
    for k in STRONGHOLD_KINDS {
        assert_eq!(content::stronghold_name(Lang::En, k.kind), k.name);
        for b in 0..2u8 {
            let name = k.branch_names[usize::from(b)];
            assert_eq!(content::stronghold_branch(Lang::En, k.kind, b), name);
            for tier in 1..=2u8 {
                let step = k.branches[usize::from(b)][usize::from(tier) - 1].name;
                assert_eq!(content::stronghold_step(Lang::En, k.kind, b, tier), step);
            }
        }
        assert!(filled(content::stronghold_name(Lang::PtBr, k.kind)));
    }
}

#[test]
fn i18n_content_coverage_roles_factions_and_statuses() {
    for f in FACTIONS {
        assert_eq!(content::faction(Lang::En, f), format!("{f:?}"));
        assert!(filled(content::faction(Lang::PtBr, f)));
    }
    for a in ARCHETYPES {
        let en = content::archetype(Lang::En, a);
        assert_eq!(en.to_lowercase(), format!("{a:?}").to_lowercase());
        assert!(filled(content::archetype(Lang::PtBr, a)));
    }
    for code in 0..3u8 {
        assert!(filled(content::status(Lang::PtBr, code)));
        assert_eq!(content::status(Lang::En, code), content::status_id(code));
    }
    assert_eq!(
        content::synergy(Lang::PtBr, tbg_core::Faction::Undead, 2),
        "Mortos-Vivos 2/2"
    );
}
