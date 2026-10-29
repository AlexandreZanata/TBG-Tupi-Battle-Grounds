//! Step 2 contract: parity rules, placeholder fill, plural, decimal, and
//! the change-guarded retext (D2, invariant 3).
//!
//! Accented expectations use `\u{...}` escapes: the `glyphs` guard scans every
//! string literal under `src/` (step 3 reworks it to the D5 charset rule).

use super::*;
use crate::i18n::check::check_pair;

fn seed() -> (
    Vec<(&'static str, &'static str)>,
    Vec<(&'static str, &'static str)>,
) {
    (
        vec![("a.title", "Hello"), ("b.count", "Need {cost} Energy")],
        vec![
            ("a.title", "Ol\u{E1}"),
            ("b.count", "Precisa de {cost} Energia"),
        ],
    )
}

#[test]
fn parity_ok_for_matching_pair() {
    let (en, pt) = seed();
    assert_eq!(check_pair(&en, &pt).unwrap(), vec!["a.title", "b.count"]);
}

#[test]
fn parity_names_the_bad_key() {
    let (en, _) = seed();
    let missing = vec![("a.title", "Ol\u{E1}")];
    assert!(check_pair(&en, &missing).unwrap_err().contains("b.count"));
    let extra = vec![
        ("a.title", "Ol\u{E1}"),
        ("b.count", "Precisa de {cost} Energia"),
        ("c.new", "Novo"),
    ];
    assert!(check_pair(&en, &extra).unwrap_err().contains("c.new"));
}

#[test]
fn parity_rejects_placeholder_and_charset_drift() {
    let (en, _) = seed();
    let placeholders = vec![("a.title", "Ol\u{E1}"), ("b.count", "Precisa de Energia")];
    assert!(check_pair(&en, &placeholders)
        .unwrap_err()
        .contains("placeholders differ for b.count"));
    let charset = vec![
        ("a.title", "Ol\u{E1}"),
        ("b.count", "Precisa de \u{203A} Energia"),
    ];
    assert!(check_pair(&en, &charset).unwrap_err().contains("b.count"));
}

#[test]
fn parity_rejects_indexed_gaps() {
    let en = vec![
        ("augment.0.name", "A"),
        ("augment.1.name", "B"),
        ("augment.3.name", "D"),
    ];
    let pt = en.clone();
    assert!(check_pair(&en, &pt)
        .unwrap_err()
        .contains("augment.{n}.name"));
}

#[test]
fn placeholder_fill_uses_named_args() {
    assert_eq!(
        Lang::En.tf(Msg::ToastNeedEnergy, &[("cost", &5)]),
        "Need 5 Energy"
    );
    assert_eq!(
        Lang::PtBr.tf(Msg::ToastNeedEnergy, &[("cost", &5)]),
        "Precisa de 5 Energia"
    );
}

#[test]
fn plural_follows_each_language_rule() {
    let (one, other) = (Msg::LobbyPlayersOne, Msg::LobbyPlayersOther);
    assert_eq!(Lang::En.plural(one, other, 1), one);
    assert_eq!(Lang::En.plural(one, other, 0), other);
    assert_eq!(Lang::En.plural(one, other, 2), other);
    assert_eq!(Lang::PtBr.plural(one, other, 0), one);
    assert_eq!(Lang::PtBr.plural(one, other, 1), one);
    assert_eq!(Lang::PtBr.plural(one, other, 2), other);
}

#[test]
fn decimal_point_follows_the_language() {
    assert_eq!(Lang::En.decimal(1.0, 2), "1.00");
    assert_eq!(Lang::PtBr.decimal(1.0, 2), "1,00");
    assert_eq!(Lang::PtBr.decimal(1.25, 2), "1,25");
}

#[test]
fn compiled_tables_match_core_canon_for_rejections() {
    // Step 9 grows this into `i18n_matches_core_canon`; the rejection half is
    // already exact: every catalog value equals the legacy literal.
    let legacy = [
        (tbg_core::Rejection::NotEnoughEnergy, "Not enough Energy"),
        (tbg_core::Rejection::NotEvolvable, "Summons cannot evolve"),
        (tbg_core::Rejection::NotRunning, "The match is not running"),
        (
            tbg_core::Rejection::Paused,
            "Battle paused - choose an Augment",
        ),
    ];
    for (reason, text) in legacy {
        assert_eq!(content::rejection(Lang::En, reason), text);
    }
    // pt-BR keeps the glossary terms (O-P41-1 proposed): Energia, Evolu\u{E7}\u{E3}o, Aprimoramento.
    assert!(
        content::rejection(Lang::PtBr, tbg_core::Rejection::NotEnoughEnergy).contains("Energia")
    );
}

#[test]
fn retext_runs_only_when_lang_changes() {
    #[derive(Resource, Default)]
    struct SawWrite(bool);
    // Change flags are only visible inside a schedule, so the probe system
    // observes them right after `retext_system` in the same update.
    fn probe(text: Single<Ref<Text>>, mut saw: ResMut<SawWrite>) {
        saw.0 = text.is_changed();
    }
    let mut app = App::new();
    app.init_resource::<Lang>()
        .init_resource::<SawWrite>()
        .add_systems(
            Update,
            (Lang::retext_system.run_if(resource_changed::<Lang>), probe).chain(),
        );
    let label = app
        .world_mut()
        .spawn((Localized(Msg::ResultVictory), Text::new("WRONG")))
        .id();
    app.update();
    assert_eq!(app.world().get::<Text>(label).unwrap().0, "VICTORY");
    assert!(app.world().resource::<SawWrite>().0);
    *app.world_mut().resource_mut::<Lang>() = Lang::PtBr;
    app.update();
    assert_eq!(app.world().get::<Text>(label).unwrap().0, "VIT\u{D3}RIA");
    assert!(app.world().resource::<SawWrite>().0);
    app.update();
    assert_eq!(app.world().get::<Text>(label).unwrap().0, "VIT\u{D3}RIA");
    assert!(!app.world().resource::<SawWrite>().0);
}
