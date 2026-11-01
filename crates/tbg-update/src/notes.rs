//! Release notes content contract (P66 §3.4, BR-UP10, U21): one structured
//! document per release, plain text only, English and pt-BR both required
//! with matching shapes, every character inside the P41 D5 charset the
//! native UI font draws, and a gameplay section whenever the match
//! contract changed. The release tool refuses anything else before signing.

use serde::{Deserialize, Serialize};

use crate::limits::{NOTES_ITEMS, NOTES_ITEM_CHARS, NOTES_TITLE_CHARS};

pub const LOCALES: [&str; 2] = ["en", "pt-BR"];
pub const KINDS: [&str; 4] = ["highlights", "gameplay", "fixes", "known_issues"];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Localized {
    pub en: String,
    #[serde(rename = "pt-BR")]
    pub pt_br: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Items {
    pub en: Vec<String>,
    #[serde(rename = "pt-BR")]
    pub pt_br: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Section {
    pub kind: String,
    pub items: Items,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Notes {
    pub schema: u32,
    pub build_id: String,
    /// UTC date `YYYY-MM-DD`.
    pub date: String,
    pub title: Localized,
    pub sections: Vec<Section>,
}

/// The P41 D5 charset (printable ASCII, Latin-1 and the listed marks).
pub fn drawable(c: char) -> bool {
    matches!(c, '\u{20}'..='\u{7E}' | '\u{A0}'..='\u{FF}')
        || matches!(
            c,
            '\u{2013}'
                | '\u{2014}'
                | '\u{2018}'
                | '\u{2019}'
                | '\u{201C}'
                | '\u{201D}'
                | '\u{2022}'
                | '\u{2026}'
                | '\u{2192}'
                | '\u{25C6}'
        )
}

fn text_ok(s: &str, max: usize) -> Result<(), String> {
    if s.trim().is_empty() {
        return Err("empty text".into());
    }
    if s.chars().count() > max {
        return Err(format!("text longer than {max} characters"));
    }
    match s.chars().find(|c| !drawable(*c)) {
        Some(c) => Err(format!(
            "character U+{:04X} outside the D5 charset",
            c as u32
        )),
        None => Ok(()),
    }
}

fn date_ok(d: &str) -> bool {
    let b = d.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

impl Notes {
    /// Validate; `rules_changed` demands a non-empty gameplay section.
    pub fn validate(&self, build_id: &str, rules_changed: bool) -> Result<(), String> {
        if self.schema != 1 {
            return Err("notes schema must be 1".into());
        }
        if self.build_id != build_id {
            return Err("notes name another build".into());
        }
        if !date_ok(&self.date) {
            return Err("date must be YYYY-MM-DD".into());
        }
        text_ok(&self.title.en, NOTES_TITLE_CHARS)?;
        text_ok(&self.title.pt_br, NOTES_TITLE_CHARS)?;
        let (mut en, mut pt) = (0, 0);
        for s in &self.sections {
            if !KINDS.contains(&s.kind.as_str()) {
                return Err(format!("unknown section {}", s.kind));
            }
            if s.items.en.len() != s.items.pt_br.len() || s.items.en.is_empty() {
                return Err(format!(
                    "section {} needs matching non-empty EN/pt-BR items",
                    s.kind
                ));
            }
            for item in s.items.en.iter().chain(&s.items.pt_br) {
                text_ok(item, NOTES_ITEM_CHARS)?;
            }
            en += s.items.en.len();
            pt += s.items.pt_br.len();
        }
        if en > NOTES_ITEMS || pt > NOTES_ITEMS {
            return Err(format!("more than {NOTES_ITEMS} items per locale"));
        }
        let kinds: Vec<&str> = self.sections.iter().map(|s| s.kind.as_str()).collect();
        if kinds
            .iter()
            .enumerate()
            .any(|(i, k)| kinds[..i].contains(k))
        {
            return Err("a section kind appears twice".into());
        }
        if rules_changed && !kinds.contains(&"gameplay") {
            return Err("the match contract changed: a gameplay section is required".into());
        }
        Ok(())
    }
}
