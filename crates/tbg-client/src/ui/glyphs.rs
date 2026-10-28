//! Guard: the default UI font is an ASCII-only subset, so any other
//! character in on-screen text renders as a box (`·`, `—`, `–`, `…`, `→`
//! all did). This test scans every string literal in the client sources.

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

    /// Non-ASCII characters inside `"..."` on a line, ignoring `//` comments.
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
            } else if in_str && !c.is_ascii() {
                return Some(c.to_string());
            }
            prev = c;
        }
        None
    }

    #[test]
    fn ui_strings_are_ascii() {
        let mut files = Vec::new();
        sources(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
            &mut files,
        );
        let mut found = Vec::new();
        for file in files {
            let text = std::fs::read_to_string(&file).unwrap();
            for (n, line) in text.lines().enumerate() {
                if let Some(c) = bad_literal(line) {
                    found.push(format!("{}:{} has {c:?}", file.display(), n + 1));
                }
            }
        }
        assert!(
            found.is_empty(),
            "non-ASCII text the font cannot draw:\n{}",
            found.join("\n")
        );
    }
}
