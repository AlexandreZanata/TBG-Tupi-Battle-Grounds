//! Pages actions for the shared P75 overlay.
use super::*;
pub(super) fn apply(action: Action, ctx: &mut Context<'_>) {
    let s = &mut *ctx.state;
    let form = &mut *ctx.form;
    match action {
        Action::Close => {
            s.page = Page::None;
            s.email = default();
            textform::close(form);
        }
        Action::Account => {
            s.page = Page::Account;
            s.data = Value::Null;
            send(s, Purpose::Account, "/api/me/account", "GET", None);
        }
        Action::LostKey => {
            textform::close(form);
            s.page = Page::Recovery;
            s.recovery = default();
        }
        Action::Emojis => {
            s.page = Page::Emojis;
            send(s, Purpose::Library, "/api/me/emoji", "GET", None);
        }
        _ => {}
    }
}
