//! Dispatch overlay actions to the identity, report, social and library flows.
use super::*;
mod email_actions;
mod email_fields;
mod library;
mod pages;
mod recovery;
mod reports;
mod social;
mod uploads;
pub(super) struct Context<'a> {
    pub state: &'a mut State,
    pub chat: &'a mut super::super::match_chat::Chat,
    pub form: &'a mut ActiveForm,
    pub info: &'a MatchInfo,
    pub game: GameState,
    pub lang: Lang,
    pub link: &'a mut crate::link::MatchLink,
    pub now: f64,
}
pub(super) fn apply(action: Action, ctx: &mut Context<'_>) {
    match action {
        Action::Close | Action::Account | Action::LostKey | Action::Emojis => {
            pages::apply(action, ctx)
        }
        Action::RecoveryName
        | Action::RecoveryEmail
        | Action::RecoveryRequest
        | Action::RecoveryCode => recovery::apply(action, ctx),
        Action::EmailVerify
        | Action::KeyProof
        | Action::NewEmail
        | Action::OldCode
        | Action::NewCode => email_fields::apply(action, ctx),
        Action::EmailResend
        | Action::Hold
        | Action::EmailChange
        | Action::EmailConfirm
        | Action::EmailCancel => email_actions::apply(action, ctx),
        #[cfg(target_os = "android")]
        Action::Compose => social::apply(action, ctx),
        Action::Chat | Action::Mute | Action::Emoji(..) | Action::MarkReport | Action::Mode => {
            social::apply(action, ctx)
        }
        Action::ReportMatch
        | Action::ReportProfile(..)
        | Action::ReportHistory(..)
        | Action::ReportEmoji(..)
        | Action::Reason(..)
        | Action::Comment
        | Action::SendReport => reports::apply(action, ctx),
        Action::Slot(..) | Action::Standard(..) | Action::EquipCustom(..) => {
            library::apply(action, ctx)
        }
        Action::Upload | Action::UploadLabel | Action::UploadSend | Action::Withdraw(..) => {
            uploads::apply(action, ctx)
        }
    }
}
