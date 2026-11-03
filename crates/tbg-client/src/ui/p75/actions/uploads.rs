//! Uploads actions for the shared P75 overlay.
use super::*;
pub(super) fn apply(action: Action, ctx: &mut Context<'_>) {
    let s = &mut *ctx.state;
    let form = &mut *ctx.form;
    let lang = &ctx.lang;
    match action {
        Action::Upload => {
            if s.library.file.is_none() {
                s.library.file = Some(crate::platform::file_pick::choose());
            }
        }
        Action::UploadLabel => prompt(s, form, *lang, Await::Label, "p75.label", TextKind::P75Text),
        Action::UploadSend => {
            if s.pending.is_none() {
                if let Some((bytes, name)) = s.library.selected.take() {
                    let mime = if name.to_ascii_lowercase().ends_with(".gif") {
                        "image/gif"
                    } else if name.to_ascii_lowercase().ends_with(".webp") {
                        "image/webp"
                    } else {
                        "image/png"
                    };
                    s.pending = Some((
                        Purpose::Upload,
                        Request::upload(&s.library.label, bytes, mime),
                    ));
                    s.notice = "p75.loading";
                }
            }
        }
        Action::Withdraw(id) => send(
            s,
            Purpose::Action,
            &format!("/api/me/emoji/{id}"),
            "DELETE",
            Some(json!({})),
        ),
        _ => {}
    }
}
