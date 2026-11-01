//! P66 §6.5 / D-92: the match contract is computed, not hand-maintained.
//! The wire epoch is the frozen `PROTOCOL_VERSION` (its bytes are pinned by
//! `contract_protocol`), and the ruleset part is exactly the newest entry
//! of the append-only `ruleset-versions.txt` registry that
//! `contract_ruleset_version` enforces. A gameplay source edit without a new
//! revision therefore fails here as well as there.

use tbg_core::contract::MatchContract;

fn newest_registry_entry() -> (u32, u64) {
    let registry = include_str!("fixtures/ruleset-versions.txt");
    let line = registry
        .lines()
        .rfind(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .expect("registry has an entry");
    let (rev, hash) = line.split_once(' ').expect("revision and hash");
    (
        rev.parse().unwrap(),
        u64::from_str_radix(hash.trim(), 16).unwrap(),
    )
}

#[test]
fn contract_is_the_protocol_plus_the_registered_ruleset() {
    let c = MatchContract::current();
    assert_eq!(c.wire, tbg_core::protocol::PROTOCOL_VERSION);
    assert_eq!((c.ruleset, c.source_hash), newest_registry_entry());
}

#[test]
fn contract_ids_round_trip_and_reject_malformed_text() {
    let c = MatchContract::current();
    assert_eq!(MatchContract::parse(&c.id()), Some(c));
    for bad in [
        "",
        "w29",
        "w29-r2",
        "w29-r2-123",
        "wx-r2-0123456789abcdef",
        "w29-r-0123456789abcdef",
        "w29-r2-0123456789ABCDEF",
        "w29-r2-0123456789abcdef0",
        "w29-r2-0123456789abcdeg",
    ] {
        assert_eq!(MatchContract::parse(bad), None, "{bad}");
    }
}
