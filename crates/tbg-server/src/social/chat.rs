//! Latin-1 plain-text admission and whole-word case/leetspeak masking.
use std::collections::HashSet;
use tbg_core::protocol::{SocialRefusal, CHAT_MAX_CHARS};
pub fn sanitize(input: &str) -> Result<String, SocialRefusal> {
    if input.len() > CHAT_MAX_CHARS * 2
        || !input
            .chars()
            .all(|c| (' '..='~').contains(&c) || ('\u{a0}'..='\u{ff}').contains(&c))
    {
        return Err(SocialRefusal::InvalidText);
    }
    let text = input
        .trim()
        .split(' ')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    if text.is_empty() || text.chars().count() > CHAT_MAX_CHARS {
        return Err(SocialRefusal::InvalidText);
    }
    Ok(text)
}
fn word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
pub fn mask(text: &str, terms: &[String]) -> String {
    let terms: HashSet<String> = tbg_core::account::BLOCKED_SUBSTRINGS
        .iter()
        .map(|s| crate::moderation::terms::fold(s))
        .chain(terms.iter().map(|s| crate::moderation::terms::fold(s)))
        .collect();
    let chars: Vec<char> = text.chars().collect();
    let folded: Vec<char> = crate::moderation::terms::fold(text).chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let mut end = None;
        if i == 0 || !word(chars[i - 1]) {
            for j in i + 1..=(i + 32).min(chars.len()) {
                if (j == chars.len() || !word(chars[j]))
                    && terms.contains(&folded[i..j].iter().collect::<String>())
                {
                    end = Some(j);
                }
            }
        }
        if let Some(j) = end {
            out.extend(std::iter::repeat_n('*', j - i));
            i = j;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}
