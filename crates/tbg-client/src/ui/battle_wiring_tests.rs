//! Contract: every button of the desktop battle layer is wired from the page
//! to a `UiAction`. A `data-battle-action` button with no listener in `hud.js`
//! does nothing (the spectator's Leave shipped that way in P47: the player
//! pressed it and stayed in the match), and a message `hud.js` sends that
//! `battle_hud.rs` does not map is silently dropped.

const PAGE: &str = include_str!("../../index.html");
const HUD: &str = include_str!("../../assets/battle/hud.js");
const RUST: &str = include_str!("battle_hud.rs");

/// Every text between `start` and the next `end` in `text`.
fn between<'a>(text: &'a str, start: &str, end: char) -> Vec<&'a str> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find(start) {
        rest = &rest[at + start.len()..];
        if let Some(stop) = rest.find(end) {
            found.push(&rest[..stop]);
        }
    }
    found
}

#[test]
fn every_battle_button_has_a_listener_in_the_page_script() {
    let actions = between(PAGE, "data-battle-action=\"", '"');
    assert!(actions.len() >= 5, "the scan found {actions:?}");
    for action in actions {
        assert!(
            HUD.contains(&format!("[data-battle-action=\"{action}\"]")),
            "the button data-battle-action=\"{action}\" has no click listener in assets/battle/hud.js"
        );
    }
}

#[test]
fn every_message_the_page_sends_is_mapped_to_an_action_in_rust() {
    let sent = between(HUD, "send('", '\'');
    assert!(sent.len() >= 8, "the scan found {sent:?}");
    for message in sent {
        // `buy:` style messages carry an index; the kind is matched as an arm.
        let mapped = match message.strip_suffix(':') {
            Some(kind) => RUST.contains(&format!("\"{kind}\" =>")),
            None => RUST.contains(&format!("value == \"{message}\"")),
        };
        assert!(
            mapped,
            "hud.js sends '{message}' but battle_hud.rs `action()` does not map it"
        );
    }
}
