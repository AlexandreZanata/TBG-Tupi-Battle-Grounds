//! Recovery actions for the shared P75 overlay.
use super::*;
pub(super) fn apply(action: Action, ctx: &mut Context<'_>) {
    let s = &mut *ctx.state;
    let form = &mut *ctx.form;
    let lang = &ctx.lang;
    match action {
        Action::RecoveryName => prompt(
            s,
            form,
            *lang,
            Await::Name,
            "p75.name_prompt",
            TextKind::P75Text,
        ),
        Action::RecoveryEmail => prompt(
            s,
            form,
            *lang,
            Await::Email,
            "p75.email_prompt",
            TextKind::P75Email,
        ),
        Action::RecoveryRequest => {
            if s.pending.is_none() {
                s.pending = Some((
                    Purpose::Request,
                    Request::recovery(&s.recovery.name, &s.recovery.email, lang.code()),
                ));
                s.notice = "p75.loading";
            }
        }
        Action::RecoveryCode => prompt(
            s,
            form,
            *lang,
            Await::RecoveryCode,
            "p75.code_prompt",
            TextKind::P75Code,
        ),
        _ => {}
    }
}
