//! Commit the native message form through the existing server-owned chat protocol.
use super::*;
use crate::ui::textform::{self, ActiveForm, TextEntered, TextKind};

pub(super) fn submit(
    mut entered: MessageReader<TextEntered>,
    mut form: NonSendMut<ActiveForm>,
    mut chat: ResMut<Chat>,
    state: Res<State<GameState>>,
    info: Res<MatchInfo>,
    mut link: NonSendMut<MatchLink>,
    time: Res<Time>,
) {
    for message in entered
        .read()
        .filter(|message| message.kind == TextKind::MatchChat)
    {
        if chat.compose_result.is_some()
            && chat.compose_result == info.result_id
            && *state.get() == GameState::InMatch
            && !info.spectating
            && chat.mode == 0
            && matches!(&*link, MatchLink::Net(_))
            && chat
                .policy
                .as_ref()
                .is_some_and(|p| p.chat == ChatPermission::Allowed)
        {
            let text: String = message
                .value
                .chars()
                .filter(|c| allow(*c))
                .take(120)
                .collect();
            let text = text.trim();
            if !text.is_empty() {
                link.send(
                    ClientMsg::ChatSend {
                        text: text.to_owned(),
                    },
                    time.elapsed_secs_f64(),
                );
            }
        }
        textform::close(&mut form);
        chat.compose_result = None;
        chat.focused = false;
        chat.expanded = false;
    }
}
