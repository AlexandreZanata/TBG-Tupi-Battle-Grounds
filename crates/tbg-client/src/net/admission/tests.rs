//! P66 client admission: the computed contract equals `tbg-core`'s, slot
//! paths are taken only from the allowed shape, and anything unexpected
//! falls back to the legacy path the server still checks.

use super::*;
use serde_json::json;

#[test]
fn the_build_script_contract_equals_the_core_contract() {
    // P66 §6.5: the client never links the rule sources, so this pins the
    // `build.rs` hash to the canonical one.
    assert_eq!(
        contract_id(),
        tbg_core::contract::MatchContract::current().id()
    );
}

#[test]
fn ws_urls_keep_the_server_host_and_name_the_contract() {
    assert_eq!(
        ws_url(
            "wss://play.example.com/ws",
            "/ws/slot-b",
            "w1-r2-0011223344556677"
        )
        .as_deref(),
        Some("wss://play.example.com/ws/slot-b?contract=w1-r2-0011223344556677")
    );
    assert_eq!(
        ws_url("ws://127.0.0.1:8080/ws", "/ws", "c").as_deref(),
        Some("ws://127.0.0.1:8080/ws?contract=c")
    );
    for bad in [
        "/ws/slot-ab",
        "/ws/../x",
        "https://evil/ws",
        "/ws/slot-B",
        "/api",
    ] {
        assert_eq!(ws_url("wss://h/ws", bad, "c"), None, "{bad}");
    }
    assert_eq!(ws_url("https://h/ws", "/ws", "c"), None);
    assert_eq!(ws_url("wss://user@h/ws", "/ws", "c"), None);
}

#[test]
fn admission_answers_become_routes_with_a_safe_fallback() {
    let server = "wss://play.example.com/ws";
    let ok = resolve(
        Some(json!({"status": "ok", "endpoint": "/ws/slot-b"})),
        server,
    );
    assert!(
        matches!(ok, Route::Ready { ref endpoint, fresh: true, .. } if endpoint == "/ws/slot-b")
    );
    assert!(matches!(
        resolve(
            Some(json!({"status": "update_required", "contract": "x"})),
            server
        ),
        Route::UpdateRequired
    ));
    // Unknown answers, a hostile endpoint or no answer: legacy `/ws`.
    for answer in [
        None,
        Some(json!({"status": "unavailable"})),
        Some(json!({"status": "ok", "endpoint": "https://evil.example/ws"})),
        Some(json!({"status": "ok"})),
    ] {
        let route = resolve(answer, server);
        assert!(matches!(route, Route::Ready { ref endpoint, .. } if endpoint == "/ws"));
    }
}
