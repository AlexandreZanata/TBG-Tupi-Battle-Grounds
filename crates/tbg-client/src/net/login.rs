//! Resume handles are usable only with the original account family or as guest handles.

pub(super) fn resume_matches(saved: Option<&str>, current: Option<&str>) -> bool {
    saved == current
}

/// A guest handle is also a scoped capability, never forward it to another endpoint.
pub(super) fn origin_matches(saved: Option<&str>, current: Option<&str>) -> bool {
    saved.is_some() && saved == current
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resume_handles_are_not_forwarded_between_origins_or_ports() {
        assert!(origin_matches(
            Some("https://one.example"),
            Some("https://one.example")
        ));
        assert!(!origin_matches(
            Some("https://one.example"),
            Some("https://other.example")
        ));
        assert!(!origin_matches(
            Some("http://127.0.0.1:8080"),
            Some("http://127.0.0.1:8081")
        ));
        assert!(!origin_matches(None, None));
        assert!(!origin_matches(None, Some("https://one.example")));
    }
    #[test]
    fn account_resume_needs_the_same_family_and_cannot_become_a_guest() {
        assert!(resume_matches(Some("a"), Some("a")));
        assert!(!resume_matches(Some("a"), Some("b")));
        assert!(!resume_matches(Some("a"), None));
        assert!(!resume_matches(None, Some("a")));
        assert!(resume_matches(None, None));
    }
}
