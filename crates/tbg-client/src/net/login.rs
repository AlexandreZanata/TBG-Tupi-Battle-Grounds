//! Resume handles are usable only with the original account family or as guest handles.

pub(super) fn resume_matches(saved: Option<&str>, current: Option<&str>) -> bool {
    saved == current
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn account_resume_needs_the_same_family_and_cannot_become_a_guest() {
        assert!(resume_matches(Some("a"), Some("a")));
        assert!(!resume_matches(Some("a"), Some("b")));
        assert!(!resume_matches(Some("a"), None));
        assert!(!resume_matches(None, Some("a")));
        assert!(resume_matches(None, None));
    }
}
