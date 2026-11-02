//! Shared case/leetspeak folding for blocked terms, independent of simulation content.
pub fn fold(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|c| match c {
            '0' => 'o',
            '1' => 'i',
            '3' => 'e',
            '4' => 'a',
            '5' => 's',
            '7' => 't',
            _ => c,
        })
        .collect()
}
