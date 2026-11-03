//! Email fields actions for the shared P75 overlay.
use super::*;
pub(super) fn apply(action: Action, ctx: &mut Context<'_>) {
    let s = &mut *ctx.state;
    let form = &mut *ctx.form;
    let lang = &ctx.lang;
    match action {
        Action::EmailVerify => prompt(
            s,
            form,
            *lang,
            Await::Verify,
            "p75.code_prompt",
            TextKind::P75Code,
        ),
        Action::KeyProof => prompt(
            s,
            form,
            *lang,
            Await::Key,
            "account.prompt.key",
            TextKind::P75Secret,
        ),
        Action::NewEmail => prompt(
            s,
            form,
            *lang,
            Await::NewEmail,
            "p75.email_prompt",
            TextKind::P75Text,
        ),
        Action::OldCode => prompt(
            s,
            form,
            *lang,
            Await::OldCode,
            "p75.old_code",
            TextKind::P75Code,
        ),
        Action::NewCode => prompt(
            s,
            form,
            *lang,
            Await::NewCode,
            "p75.new_code",
            TextKind::P75Code,
        ),
        _ => {}
    }
}
