//! Guard: every shipped string must be drawable by the D5 Latin-1 subset.
//!
//! The rule covers catalog values (`assets/i18n/*.json`) and every other
//! string literal in client sources; the build script enforces the catalog
//! half and this test the literal half. Test-only `\u{...}` escapes stay
//! ASCII in source and are unaffected.

#[cfg(test)]
mod tests {
    use std::path::Path;

    fn sources(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                sources(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    /// The D5 charset: printable ASCII, Latin-1, and the listed punctuation.
    fn drawable(c: char) -> bool {
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

    /// Non-D5 characters inside `"..."` on a line, ignoring `//` comments.
    fn bad_literal(line: &str) -> Option<String> {
        let code = line.trim_start();
        if code.starts_with("//") {
            return None;
        }
        let mut in_str = false;
        let mut prev = ' ';
        for (i, c) in code.char_indices() {
            if !in_str && c == '/' && code[i..].starts_with("//") {
                return None;
            }
            if c == '"' && prev != '\\' {
                in_str = !in_str;
            } else if in_str && !drawable(c) {
                return Some(c.to_string());
            }
            prev = c;
        }
        None
    }

    #[test]
    fn ui_strings_are_drawable() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut files = Vec::new();
        sources(&manifest.join("src"), &mut files);
        files.push(manifest.join("assets/i18n/en.json"));
        files.push(manifest.join("assets/i18n/pt-BR.json"));
        let mut found = Vec::new();
        for file in files {
            let text = std::fs::read_to_string(&file).unwrap();
            for (n, line) in text.lines().enumerate() {
                if file.extension().is_some_and(|e| e == "json") {
                    if let Some(c) = line.chars().find(|c| !drawable(*c)) {
                        found.push(format!("{}:{} has {c:?}", file.display(), n + 1));
                    }
                } else if let Some(c) = bad_literal(line) {
                    found.push(format!("{}:{} has {c:?}", file.display(), n + 1));
                }
            }
        }
        assert!(
            found.is_empty(),
            "characters the D5 subset cannot draw:\n{}",
            found.join("\n")
        );
    }
}
