//! Social actions for the shared P75 overlay.
use super::*;
pub(super) fn apply(action: Action, ctx: &mut Context<'_>) {
    let s = &mut *ctx.state;
    let chat = &mut *ctx.chat;
    let link = &mut *ctx.link;
    let info = ctx.info;
    match action {
        Action::Chat => {
            if !info.spectating && ctx.game == GameState::InMatch && chat.mode < 2 {
                chat.expanded = !chat.expanded;
                chat.focused = !cfg!(target_os = "android")
                    && chat.expanded
                    && chat.mode == 0
                    && chat
                        .policy
                        .as_ref()
                        .is_some_and(|p| p.chat == tbg_core::protocol::ChatPermission::Allowed);
            }
        }
        #[cfg(target_os = "android")]
        Action::Compose => {
            if !info.spectating
                && ctx.game == GameState::InMatch
                && chat.mode == 0
                && chat
                    .policy
                    .as_ref()
                    .is_some_and(|p| p.chat == tbg_core::protocol::ChatPermission::Allowed)
            {
                crate::ui::textform::open(
                    ctx.form,
                    TextKind::MatchChat,
                    &ctx.lang.key("android.chat_compose"),
                );
            }
        }
        Action::Mute => chat.muted = !chat.muted,
        Action::Emoji(slot) => {
            if !info.spectating
                && ctx.game == GameState::InMatch
                && chat.mode < 2
                && chat.policy.as_ref().is_some_and(|p| p.emojis)
            {
                link.send(tbg_core::protocol::ClientMsg::EmojiSend { slot }, ctx.now);
            }
        }
        Action::MarkReport => {
            s.marked = true;
            s.notice = "p75.report_after";
        }
        Action::Mode => {
            chat.mode = (chat.mode + 1) % 3;
            crate::config::save_flag("p75_chat_off", chat.mode == 2);
            crate::config::save_flag("p75_chat_emoji", chat.mode == 1);
        }
        _ => {}
    }
}
