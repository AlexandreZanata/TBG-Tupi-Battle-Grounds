//! Active display language and compiled string lookup.
//!
//! The catalog is compiled by `build.rs` into array tables: lookup is one
//! bounds-checked index, with no hash, parse, or fetch on any path. Static
//! labels carry [`Localized`] and are re-texted only when [`Lang`] changes.

use bevy::prelude::*;

pub(crate) mod check;
pub mod content;
#[cfg(test)]
mod tests;

include!(concat!(env!("OUT_DIR"), "/catalog.rs"));

/// Display language. The server and `tbg-core` never see this.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Lang {
    #[default]
    En,
    /// First constructed in step 4 from `<html lang>`.
    #[allow(dead_code)]
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
}

// Steps 4-8 wire the live call sites; covered by `tests` until then.
#[allow(dead_code)]
impl Lang {
    /// BCP 47 code for `<html lang>` and the catalog file name.
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::PtBr => "pt-BR",
        }
    }

    /// `pt*` → PtBr, `en*` → En (so `pt-PT` opens in Portuguese); else `None`.
    /// Startup only, never on the hot path.
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
}

/// Static Bevy label: translated at spawn, re-texted only on `Lang` change.
#[derive(Component, Clone, Copy)]
#[allow(dead_code)] // First live labels land in step 8; tested in `tests`.
pub struct Localized(pub Msg);

pub fn plugin(app: &mut App) {
    app.init_resource::<Lang>()
        .add_systems(Update, Lang::retext_system.run_if(resource_changed::<Lang>));
}
