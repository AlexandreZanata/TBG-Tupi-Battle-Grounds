//! P67 C18/C19 synthetic presentation contract; no emote or skin consumer exists yet.

use serde_json::{json, Value};
use tbg_core::{content, Seat, StrongholdKind, StrongholdView, UnitView, ALL_KINDS};

/// Only optional presentation is classified here. Actual combat events do not
/// enter an emote channel, so unknown future cue types remain visible by default.
#[derive(Clone, Copy)]
enum Cue {
    EmoteText,
    EmoteAudio,
    EmoteAnimation,
    Combat,
}

#[derive(Default, PartialEq, Eq, Debug)]
struct Preferences {
    mute_opponent_emotes: bool,
    hide_opponent_cosmetics: bool,
}

impl Preferences {
    /// A local storage document, not a server command or a new public protocol.
    fn read(doc: &Value) -> Option<Self> {
        let fields = doc.as_object()?;
        if fields.len() != 3 || doc["schemaVersion"] != 1 {
            return None;
        }
        Some(Self {
            mute_opponent_emotes: doc["muteOpponentEmotes"].as_bool()?,
            hide_opponent_cosmetics: doc["hideOpponentCosmetics"].as_bool()?,
        })
    }

    fn document(&self) -> Value {
        json!({
            "schemaVersion":1,
            "muteOpponentEmotes":self.mute_opponent_emotes,
            "hideOpponentCosmetics":self.hide_opponent_cosmetics,
        })
    }

    fn audible_or_visible(&self, viewer: Seat, owner: Seat, cue: Cue) -> bool {
        !(self.mute_opponent_emotes
            && viewer != owner
            && matches!(cue, Cue::EmoteText | Cue::EmoteAudio | Cue::EmoteAnimation))
    }

    /// Baselines remain an owner/art gate. Without one, retain readable equipment
    /// rather than fabricate a replacement kind/form or erase a visible enemy.
    fn ornament<'a>(
        &self,
        viewer: Seat,
        owner: Seat,
        equipped: &'a str,
        approved_baseline: Option<&'a str>,
    ) -> &'a str {
        if self.hide_opponent_cosmetics && viewer != owner {
            return approved_baseline.unwrap_or(equipped);
        }
        equipped
    }
}

#[test]
fn p67_c18_only_opponents_optional_emotes_are_suppressed_and_document_roundtrips() {
    // BR-CO06: global audio and critical gameplay cues never share this switch.
    let document: Value = serde_json::from_str(include_str!(
        "../../../docs/modules/commerce/contracts/examples/opponent-preferences.json"
    ))
    .unwrap();
    let preferences = Preferences::read(&document).unwrap();
    assert_eq!(
        Preferences::read(&preferences.document()),
        Some(preferences)
    );
    let preferences = Preferences::read(&document).unwrap();
    for viewer in Seat::BOTH {
        for cue in [Cue::EmoteText, Cue::EmoteAudio, Cue::EmoteAnimation] {
            assert!(!preferences.audible_or_visible(viewer, viewer.other(), cue));
            assert!(preferences.audible_or_visible(viewer, viewer, cue));
            assert!(Preferences::default().audible_or_visible(viewer, viewer.other(), cue));
        }
        assert!(preferences.audible_or_visible(viewer, viewer.other(), Cue::Combat));
        assert!(preferences.audible_or_visible(viewer, viewer, Cue::Combat));
    }
    for bad in [
        json!({}),
        json!({"schemaVersion":2,"muteOpponentEmotes":true,"hideOpponentCosmetics":true}),
        json!({"schemaVersion":1,"muteOpponentEmotes":"true","hideOpponentCosmetics":true}),
        json!({"schemaVersion":1,"muteOpponentEmotes":true,"hideOpponentCosmetics":true,"recipient":"tampered"}),
    ] {
        assert_eq!(Preferences::read(&bad), None);
    }
}

#[test]
fn p67_c19_all_canonical_forms_and_stronghold_kinds_keep_mechanical_identity() {
    // This proves the fixture resolver contract, not integrated model swapping.
    let preferences = Preferences {
        mute_opponent_emotes: true,
        hide_opponent_cosmetics: true,
    };
    let mut forms = 0;
    for viewer in Seat::BOTH {
        for kind in ALL_KINDS {
            for (tier, branch) in [(0, 0), (1, 0), (1, 1), (2, 0), (2, 1)] {
                if tier != 0 && content::form_of(kind, tier, branch).is_none() {
                    continue;
                }
                forms += 1;
                for owner in Seat::BOTH {
                    for fx in [0, tbg_core::view::FX_SUMMON, tbg_core::view::FX_CARRIED] {
                        let unit = UnitView {
                            id: 42,
                            seat: owner,
                            kind,
                            tier,
                            branch,
                            x: 7000,
                            lane_z: 0,
                            hp: 50,
                            max_hp: 100,
                            status: 2,
                            fx,
                            carrier: Some(41),
                            shield: 7,
                            reduction: 500,
                            special: Some(tbg_core::view::SpecialView {
                                ready_in: 10,
                                cast: None,
                                windup: tbg_core::content::WINDUP,
                            }),
                            effects: None,
                        };
                        let before = serde_json::to_value(&unit).unwrap();
                        let rendered = preferences.ornament(
                            viewer,
                            unit.seat,
                            "fixture-equipped",
                            Some("fixture-reviewed-baseline"),
                        );
                        assert_eq!(
                            rendered,
                            if owner == viewer {
                                "fixture-equipped"
                            } else {
                                "fixture-reviewed-baseline"
                            }
                        );
                        assert_eq!(serde_json::to_value(&unit).unwrap(), before);
                        assert_eq!(
                            preferences.ornament(viewer, unit.seat, "fixture-equipped", None),
                            "fixture-equipped"
                        );
                    }
                }
            }
        }
        for kind in std::iter::once(StrongholdKind::Classic).chain(StrongholdKind::SELECTABLE) {
            let stronghold = StrongholdView {
                kind,
                tier: 2,
                branch: 1,
                max_hp: 1000,
            };
            let before = stronghold;
            assert_eq!(
                preferences.ornament(
                    viewer,
                    viewer.other(),
                    "fixture-free-style",
                    Some("fixture-kind-baseline")
                ),
                "fixture-kind-baseline"
            );
            assert_eq!(stronghold, before);
        }
    }
    assert_eq!(forms, ALL_KINDS.len() * 5 * Seat::BOTH.len());
}
