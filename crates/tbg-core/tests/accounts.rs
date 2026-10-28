//! Account name/key contract tests (GAME-DESIGN BR-P01, BR-P02).

use tbg_core::account::{check_name, encode_secret, format_key, parse_key};
use tbg_core::protocol::AccountError;

#[test]
fn br_p01_valid_names_pass() {
    for name in ["Aldric", "bob_99", "Z_1", "SixteenCharName1"] {
        assert!(check_name(name).is_ok(), "{name}");
    }
}

#[test]
fn br_p01_bad_shapes_rejected() {
    // Too short/long, leading digit/underscore, symbols, non-ASCII.
    for name in [
        "",
        "ab",
        "1abc",
        "_abc",
        "has space",
        "has-dash",
        "SeventeenCharName17",
        "çé",
    ] {
        assert_eq!(check_name(name), Err(AccountError::NameInvalid), "{name}");
    }
}

#[test]
fn br_p01_reserved_and_blocked_rejected() {
    for name in ["Admin", "SERVER", "Guest", "moderator", "Bot"] {
        assert_eq!(check_name(name), Err(AccountError::NameInvalid), "{name}");
    }
    // Blocked roots, including leetspeak with the 0→o/1→i/3→e/4→a/5→s/7→t map.
    for name in ["BigFuck", "sh1thead", "P0rraX", "c4r4lh0"] {
        assert_eq!(check_name(name), Err(AccountError::NameInvalid), "{name}");
    }
    // Innocent names containing no blocked root still pass.
    assert!(check_name("Brickmason").is_ok());
}

#[test]
fn br_p02_key_round_trip() {
    let secret = encode_secret(&[7u8, 1, 9, 3, 5, 2, 8, 4, 6, 0]);
    let key = format_key("Aldric", &secret);
    assert_eq!(key.len(), 6 + 1 + 19);
    let Some((name, back)) = parse_key(&key) else {
        panic!("round trip failed");
    };
    assert_eq!(name, "Aldric");
    assert_eq!(back, secret);
}

#[test]
fn br_p02_key_normalization() {
    // Lowercase plus O/I/L look-alikes normalize to the same secret.
    let secret = encode_secret(&[9u8, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
    let key = format_key("Bob", &secret);
    let loose = key.to_lowercase().replace('0', "o").replace('1', "i");
    let Some((_, back)) = parse_key(&loose) else {
        panic!("normalization failed");
    };
    assert_eq!(back, secret);
}

#[test]
fn br_p02_bad_keys_rejected() {
    // Wrong group count, too short, non-alphabet chars, oversized input.
    for key in [
        "Bob-AAAA-BBBB-CCCC",
        "Bob-AAAA-BBBB-CCCC-DDDD-EEEE",
        "Bob-AAAA-BBBB-CCCC-DDD!",
        "Bob-",
        "no-dashes-here-at-all",
        &("x".repeat(41)),
    ] {
        assert!(parse_key(key).is_none(), "{key}");
    }
}
