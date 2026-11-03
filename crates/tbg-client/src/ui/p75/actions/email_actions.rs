//! Email actions actions for the shared P75 overlay.
use super::*;
pub(super) fn apply(action: Action, ctx: &mut Context<'_>) {
    let s = &mut *ctx.state;
    let lang = &ctx.lang;
    match action {
        Action::EmailResend => send(
            s,
            Purpose::Action,
            "/api/me/email/resend",
            "POST",
            Some(json!({})),
        ),
        Action::Hold => s.email.hold = !s.email.hold,
        Action::EmailChange => {
            let body = json!({"account_key":s.email.key,"email":s.email.email,"locale":lang.code(),"lost_old":s.email.hold});
            send(s, Purpose::Action, "/api/me/email", "POST", Some(body));
            s.email.key.clear();
        }
        Action::EmailConfirm => {
            let body = json!({"old_code":s.email.old_code,"new_code":s.email.new_code});
            send(
                s,
                Purpose::Action,
                "/api/me/email/change/confirm",
                "POST",
                Some(body),
            );
        }
        Action::EmailCancel => send(
            s,
            Purpose::Action,
            "/api/me/email/change/cancel",
            "POST",
            Some(json!({})),
        ),
        _ => {}
    }
}
