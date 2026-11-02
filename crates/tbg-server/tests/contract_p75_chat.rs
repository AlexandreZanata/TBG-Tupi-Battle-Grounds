//! BR-CH/EM, IC-06/12 and S07/09/11: bounded seat-only cosmetic messages.
#[path = "common/p75.rs"]
mod common;
#[path = "common/ws.rs"]
mod ws;
use common::*;
use serde_json::json;
use tbg_core::protocol::{ClientMsg, ServerMsg, SocialRefusal};
#[tokio::test]
async fn br_ch01_social_traffic_never_changes_the_match_and_only_seats_receive_it() {
    let (tx, mut replays) = tokio::sync::mpsc::unbounded_channel();
    let s = Server::configured(|c| c.replays = Some(tx)).await;
    let a = s.register("Chatter", "chat@example.invalid").await;
    let code = s.mail("account_verify").await.code.unwrap();
    assert_eq!(
        s.request(
            "POST",
            "/api/me/email/verify",
            json!({"code":code}),
            a["access_token"].as_str()
        )
        .await
        .status,
        200
    );
    s.sql("UPDATE admin_settings SET value=1 WHERE key='chat_enabled';INSERT INTO moderation_events(at_unix,kind,payload_json) VALUES(1000000,'settings','{}');");
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    let mut left = ws::connect(&s.url, Some(&a)).await;
    let mut right = ws::connect(&s.url, None).await;
    let mut spectator = ws::connect(&s.url, None).await;
    ws::send(&mut left, ClientMsg::QuickMatch).await;
    ws::send(&mut right, ClientMsg::QuickMatch).await;
    ws::until(&mut left, |m| matches!(m, ServerMsg::MatchStart { .. })).await;
    ws::until(&mut right, |m| matches!(m, ServerMsg::MatchStart { .. })).await;
    let policy = ws::until(&mut left, |m| matches!(m, ServerMsg::MatchSocial(_))).await;
    assert!(
        matches!(policy,ServerMsg::MatchSocial(p)if p.chat==tbg_core::protocol::ChatPermission::Allowed)
    );
    ws::send(
        &mut left,
        ClientMsg::ChatSend {
            text: "  hello   sh1t  ".into(),
        },
    )
    .await;
    for client in [&mut left, &mut right] {
        assert!(
            matches!(ws::until(client,|m|matches!(m,ServerMsg::Chat{..})).await,ServerMsg::Chat{text,..}if text=="hello ****")
        );
    }
    ws::send(
        &mut right,
        ClientMsg::ChatSend {
            text: "guest".into(),
        },
    )
    .await;
    assert!(matches!(
        ws::until(&mut right, |m| matches!(m, ServerMsg::SocialRefused { .. })).await,
        ServerMsg::SocialRefused {
            reason: SocialRefusal::Restricted
        }
    ));
    ws::send(&mut left, ClientMsg::EmojiSend { slot: 0 }).await;
    for client in [&mut left, &mut right] {
        ws::until(client, |m| matches!(m, ServerMsg::Emoji { .. })).await;
    }
    ws::send(
        &mut spectator,
        ClientMsg::ChatSend {
            text: "outside".into(),
        },
    )
    .await;
    assert!(matches!(
        ws::until(&mut spectator, |m| matches!(
            m,
            ServerMsg::SocialRefused { .. }
        ))
        .await,
        ServerMsg::SocialRefused {
            reason: SocialRefusal::NotAvailable
        }
    ));
    ws::send(&mut left, ClientMsg::Surrender).await;
    ws::until(&mut right, |m| matches!(m, ServerMsg::MatchEnd { .. })).await;
    let replay = tokio::time::timeout(std::time::Duration::from_secs(2), replays.recv())
        .await
        .unwrap()
        .unwrap();
    let baseline = replay.run();
    assert_eq!(
        baseline.hash(),
        replay.final_hash,
        "IC-06 replay contains only gameplay and produces identical final state"
    );
    assert_eq!(replay.ops.len(), 1, "social traffic never enters replay");
}
#[test]
fn ch04_s07_s09_sanitizer_and_whole_word_masking_never_fetch_urls() {
    use tbg_server::social::chat::*;
    for invalid in [
        "<script>\n",
        "\u{202e}",
        "\u{200b}",
        "\u{7f}",
        &"a".repeat(121),
    ] {
        assert!(sanitize(invalid).is_err());
    }
    assert_eq!(sanitize(" hello   café ").unwrap(), "hello café");
    assert_eq!(
        mask("sh1t shitake SHIT https://example.invalid", &[]),
        "**** shitake **** https://example.invalid"
    );
}
#[test]
fn ch05_em05_s11_budgets_are_bounded_and_expire() {
    use tbg_server::social::limits::Limits;
    let mut limit = Limits::default();
    assert!(limit.chat("one", 0).is_ok());
    assert_eq!(limit.chat("one", 1), Err(SocialRefusal::Duplicate));
    assert!(limit.chat("two", 1).is_ok());
    assert!(limit.chat("three", 2).is_ok());
    assert_eq!(limit.chat("four", 3), Err(SocialRefusal::RateLimited));
    assert!(limit.chat("four", 5000).is_ok());
    assert!(limit.emoji(0).is_ok());
    assert_eq!(limit.emoji(1999), Err(SocialRefusal::RateLimited));
    assert!(limit.emoji(2000).is_ok());
}
