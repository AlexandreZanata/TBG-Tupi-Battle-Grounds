//! Character-indexed bounded text editing independent of Bevy and platform APIs.

#[derive(Clone, Default)]
pub struct Edit {
    pub text: String,
    pub cursor: usize,
    pub anchor: usize,
}

impl Edit {
    fn byte(&self, at: usize) -> usize {
        self.text
            .char_indices()
            .nth(at)
            .map_or(self.text.len(), |(i, _)| i)
    }
    pub fn selected(&self) -> &str {
        &self.text[self.byte(self.cursor.min(self.anchor))..self.byte(self.cursor.max(self.anchor))]
    }
    fn erase_selection(&mut self) {
        let lo = self.cursor.min(self.anchor);
        let hi = self.cursor.max(self.anchor);
        self.text.replace_range(self.byte(lo)..self.byte(hi), "");
        self.cursor = lo;
        self.anchor = lo;
    }
    pub fn insert(&mut self, value: &str, limit: usize, allow: impl Fn(char) -> bool) {
        self.erase_selection();
        let room = limit.saturating_sub(self.text.chars().count());
        let value: String = value
            .chars()
            .filter(|c| !c.is_control() && allow(*c))
            .take(room)
            .collect();
        self.text.insert_str(self.byte(self.cursor), &value);
        self.cursor += value.chars().count();
        self.anchor = self.cursor;
    }
    pub fn move_to(&mut self, at: usize, selecting: bool) {
        self.cursor = at.min(self.text.chars().count());
        if !selecting {
            self.anchor = self.cursor;
        }
    }
    pub fn backspace(&mut self) {
        if self.cursor == self.anchor {
            self.anchor = self.cursor.saturating_sub(1);
        }
        self.erase_selection();
    }
    pub fn delete(&mut self) {
        if self.cursor == self.anchor {
            self.anchor = (self.cursor + 1).min(self.text.chars().count());
        }
        self.erase_selection();
    }
    pub fn display(&self, masked: bool) -> String {
        let chars: Vec<char> = self.text.chars().collect();
        let mut out = String::new();
        for at in 0..=chars.len() {
            if at == self.cursor {
                out.push('|');
            }
            if let Some(&c) = chars.get(at) {
                out.push(if masked { '*' } else { c });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_selection_paste_and_limits_use_characters() {
        let mut e = Edit::default();
        e.insert("João", 16, |_| true);
        e.move_to(3, false);
        e.backspace();
        assert_eq!(e.text, "Joo");
        e.move_to(0, false);
        e.move_to(3, true);
        e.insert("Álex\nandre", 5, |_| true);
        assert_eq!(e.text, "Álexa");
        assert!(!e.display(true).contains("Álexa"));
        e.move_to(0, false);
        e.delete();
        assert_eq!(e.text, "lexa");
        e.insert("ab1!ç", 6, |c| c.is_ascii_alphanumeric());
        assert_eq!(e.text, "ablexa");
    }
}
