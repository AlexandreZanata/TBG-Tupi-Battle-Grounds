//! Catalog parity rules, shared by `build.rs` and `i18n::tests`.
//!
//! Pure `std`: the build script includes this file with `#[path]`, so it must
//! not use any crate dependency. Returns the sorted key list on success so
//! codegen and runtime tables agree on `Msg` order.

#![allow(dead_code)] // Wired to the build by `build.rs`; runtime call sites land in steps 5-8.

/// Characters a catalog value may use (D5): printable ASCII without DEL,
/// Latin-1, and the listed punctuation, arrow, and diamond.
pub fn in_charset(c: char) -> bool {
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

/// Sorted, deduplicated `{name}` placeholders in a catalog value.
pub fn placeholders(value: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = value;
    while let Some(open) = rest.find('{') {
        rest = &rest[open + 1..];
        if let Some(close) = rest.find('}') {
            let name = &rest[..close];
            if !name.is_empty()
                && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                && !out.contains(&name)
            {
                out.push(name);
            }
            rest = &rest[close + 1..];
        } else {
            break;
        }
    }
    out.sort_unstable();
    out
}

/// True when the key is `lowercase`, dots and `_` only (D1).
fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'_')
}

/// Split a key into (head, id, tail) at its single all-digit segment.
fn indexed(key: &str) -> Option<(String, usize, String)> {
    let mut found = None;
    for (i, part) in key.split('.').enumerate() {
        if !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()) {
            if found.is_some() {
                return None;
            }
            found = Some(i);
        }
    }
    let at = found?;
    let parts: Vec<&str> = key.split('.').collect();
    let id: usize = parts[at].parse().ok()?;
    Some((parts[..at].join("."), id, parts[at + 1..].join(".")))
}

/// Check an English/pt-BR catalog pair; `Ok` holds the sorted shared keys.
///
/// The build fails (panics naming the key) when: a key is missing in pt-BR
/// or extra in it; placeholder sets differ; a value holds a character
/// outside the font charset; an indexed family (`augment.{n}.name`) has a gap.
pub fn check_pair(en: &[(&str, &str)], pt: &[(&str, &str)]) -> Result<Vec<String>, String> {
    for (key, _) in en.iter().chain(pt.iter()) {
        if !valid_key(key) {
            return Err(format!(
                "i18n: bad key shape (lowercase, dots and _ only): {key}"
            ));
        }
    }
    let mut en_sorted: Vec<(&str, &str)> = en.to_vec();
    let mut pt_sorted: Vec<(&str, &str)> = pt.to_vec();
    en_sorted.sort_unstable();
    pt_sorted.sort_unstable();
    let mut keys = Vec::with_capacity(en_sorted.len());
    let mut pt_iter = pt_sorted.iter().peekable();
    for (key, value) in &en_sorted {
        match pt_iter.peek() {
            None => return Err(format!("i18n: key missing in pt-BR: {key}")),
            Some((pk, _)) if *pk < *key => {
                return Err(format!("i18n: key extra in pt-BR (absent in en): {pk}"));
            }
            Some((pk, pv)) if *pk == *key => {
                let want = placeholders(value);
                let have = placeholders(pv);
                if want != have {
                    return Err(format!(
                        "i18n: placeholders differ for {key}: en={want:?} pt={have:?}"
                    ));
                }
                pt_iter.next();
            }
            _ => return Err(format!("i18n: key missing in pt-BR: {key}")),
        }
        keys.push((*key).to_string());
    }
    if let Some((extra, _)) = pt_iter.next() {
        return Err(format!("i18n: key extra in pt-BR (absent in en): {extra}"));
    }
    for (key, value) in en.iter().chain(pt.iter()) {
        if let Some(c) = value.chars().find(|c| !in_charset(*c)) {
            return Err(format!(
                "i18n: {key} has char outside the font charset: {c:?} (U+{:04X})",
                c as u32
            ));
        }
    }
    // Indexed families share (head, tail) around one digit segment and must
    // hold exactly ids 0..n with no gap.
    let mut families: Vec<((String, String), Vec<usize>)> = Vec::new();
    for key in &keys {
        let Some((head, id, tail)) = indexed(key) else {
            continue;
        };
        match families
            .iter_mut()
            .find(|(f, _)| *f == (head.clone(), tail.clone()))
        {
            Some((_, ids)) => ids.push(id),
            None => families.push(((head, tail), vec![id])),
        }
    }
    for ((head, tail), mut ids) in families {
        ids.sort_unstable();
        if ids.iter().enumerate().any(|(i, id)| *id != i) {
            return Err(format!(
                "i18n: indexed family {head}.{{n}}.{tail} has a gap: {ids:?}"
            ));
        }
    }
    Ok(keys)
}

/// PascalCase enum variant for a catalog key (`a.b_c` → `ABC`).
pub fn variant(key: &str) -> String {
    key.split(['.', '_'])
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_ascii_uppercase().to_string() + c.as_str(),
            }
        })
        .collect()
}
