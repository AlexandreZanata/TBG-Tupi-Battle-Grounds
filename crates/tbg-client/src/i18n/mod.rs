//! Active display language and compiled string lookup.
//!
//! The catalog is compiled by `build.rs` into array tables: lookup is one
//! bounds-checked index, with no hash, parse, or fetch on any path. Static
//! labels carry [`Localized`] and are re-texted only when [`Lang`] changes.

use bevy::asset::{AssetId, Assets};
use bevy::prelude::*;
use bevy::text::Font;

pub mod augment;
pub(crate) mod check;
pub mod content;
#[cfg(test)]
mod content_coverage;
#[cfg(test)]
mod tests;

include!(concat!(env!("OUT_DIR"), "/catalog.rs"));

/// Display language. The server and `tbg-core` never see this.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Lang {
    #[default]
    En,
    PtBr,
}

impl Lang {
    /// Compiled lookup: one array index.
    pub fn t(self, msg: Msg) -> &'static str {
        let index = msg as usize;
        debug_assert!(index < MSG_COUNT);
        match self {
            Lang::En => EN[index],
            Lang::PtBr => PT_BR[index],
        }
    }

    /// Dynamic catalog families resolve once while constructing a screen.
    pub fn key(self, key: &str) -> String {
        KEYS.binary_search(&key)
            .map(|i| {
                match self {
                    Self::En => EN[i],
                    Self::PtBr => PT_BR[i],
                }
                .to_string()
            })
            .unwrap_or_else(|_| key.to_string())
    }

    /// Re-text a static label only when the language changes.
    pub fn retext_system(lang: Res<Lang>, mut labels: Query<(&Localized, &mut Text)>) {
        let lang = *lang;
        for (slot, mut text) in &mut labels {
            let want = lang.t(slot.0);
            if text.0 != want {
                text.0 = want.to_string();
            }
        }
    }

    /// `pt*` → PtBr, `en*` → En (so `pt-PT` opens in Portuguese); else `None`.
    /// Startup and the language switch only, never on the hot path.
    pub fn from_code(code: &str) -> Option<Self> {
        let lower = code.to_ascii_lowercase();
        if lower.starts_with("pt") {
            Some(Lang::PtBr)
        } else if lower.starts_with("en") {
            Some(Lang::En)
        } else {
            None
        }
    }
}

// Steps 5-8 wire the live call sites; covered by `tests` until then.
#[allow(dead_code)]
impl Lang {
    /// BCP 47 code for `<html lang>` and the catalog file name.
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::PtBr => "pt-BR",
        }
    }

    /// Fill named `{placeholders}` from `args`; unknown names stay as-is.
    pub fn tf(self, msg: Msg, args: &[(&str, &dyn std::fmt::Display)]) -> String {
        let mut out = self.t(msg).to_string();
        for (name, value) in args {
            out = out.replace(&format!("{{{name}}}"), &value.to_string());
        }
        out
    }

    /// Pick the `one`/`other` key: pt-BR uses `one` for 0 and 1 (CLDR `i = 0,1`).
    pub fn plural(self, one: Msg, other: Msg, n: i64) -> Msg {
        match self {
            Lang::En => {
                if n == 1 {
                    one
                } else {
                    other
                }
            }
            Lang::PtBr => {
                if n == 0 || n == 1 {
                    one
                } else {
                    other
                }
            }
        }
    }

    /// Decimal point follows the language: `1.25` vs `1,25` (Interest `×1.00`).
    pub fn decimal(self, value: f64, places: usize) -> String {
        let text = format!("{value:.places$}");
        match self {
            Lang::En => text,
            Lang::PtBr => text.replace('.', ","),
        }
    }

    /// Like [`Lang::decimal`] with trailing zeros dropped: `15`, `0.15`, `2.5`.
    pub fn compact(self, value: f64, places: usize) -> String {
        let text = format!("{value:.places$}");
        let text = if text.contains('.') {
            text.trim_end_matches('0').trim_end_matches('.')
        } else {
            &text
        };
        match self {
            Lang::En => text.to_string(),
            Lang::PtBr => text.replace('.', ","),
        }
    }
}

/// Static Bevy label: translated at spawn, re-texted only on `Lang` change.
#[derive(Component, Clone, Copy)]
#[allow(dead_code)] // First live labels land in step 8; tested in `tests`.
pub struct Localized(pub Msg);

/// Latin-1 UI face (P41, O-P41-3): Noto Sans, same family as the lobby `Camp`.
/// Subset with the D5 charset; see `assets/fonts/OFL.txt` for the commands.
static UI_FONT: &[u8] = include_bytes!("../../assets/fonts/BattlegroundSansUi.ttf");

/// Language at startup: `<html lang>` on wasm (written by the resolver),
/// `?lang=` through [`crate::config::url_param`] on native; else English.
fn startup_lang() -> Lang {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element())
            .and_then(|e| e.get_attribute("lang"))
            .and_then(|code| Lang::from_code(&code))
            .unwrap_or_default()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        crate::config::url_param("lang")
            .or_else(|| {
                crate::platform::storage::get(crate::platform::storage::Scope::Local, "tbg.lang")
            })
            .and_then(|code| Lang::from_code(&code))
            .unwrap_or_default()
    }
}

pub fn plugin(app: &mut App) {
    app.insert_resource(startup_lang())
        .add_systems(Update, Lang::retext_system.run_if(resource_changed::<Lang>));
    // Same slot `bevy_text` used for its default font, so every existing
    // `TextFont` without a handle keeps working with no other change.
    let mut fonts = app.world_mut().resource_mut::<Assets<Font>>();
    let font = Font::from_bytes(UI_FONT.to_vec());
    fonts.insert(AssetId::default(), font).unwrap();
    fonts
        .insert(
            crate::ui::kit::display_font().id(),
            Font::from_bytes(
                include_bytes!("../../assets/fonts/BattlegroundDisplayUi.ttf").to_vec(),
            ),
        )
        .unwrap();
}
