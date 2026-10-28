//! Post-P14 (BR-P03): a `Welcome` that creates a fresh session (token
//! differs from the one the link sent) must log in again before any mode
//! request, otherwise the server queues a guest while the UI shows an
//! account.

use tbg_core::protocol::Token;

/// How long an automatic login may take before the link proceeds as guest.
pub(super) const LOGIN_TIMEOUT: f64 = 3.0;

/// `previous` is the token sent in `Hello`; `new` is the one in `Welcome`.
pub(super) fn needs_login(
    previous: Option<Token>,
    new: Token,
    key_stored: bool,
    in_match: bool,
) -> bool {
    !in_match && key_stored && previous != Some(new)
}

#[cfg(test)]
mod tests {
    use super::needs_login;

    const T1: [u8; 16] = [1; 16];
    const T2: [u8; 16] = [2; 16];

    #[test]
    fn resumed_session_never_logs_in_again() {
        assert!(!needs_login(Some(T1), T1, true, false));
        assert!(!needs_login(Some(T1), T1, true, true));
    }

    #[test]
    fn fresh_session_with_stored_key_logs_in() {
        assert!(needs_login(Some(T1), T2, true, false));
        assert!(needs_login(None, T2, true, false));
    }

    #[test]
    fn no_login_without_key_or_inside_a_match() {
        assert!(!needs_login(Some(T1), T2, false, false));
        assert!(!needs_login(Some(T1), T2, true, true));
    }
}
