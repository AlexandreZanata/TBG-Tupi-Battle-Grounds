//! Reports actions for the shared P75 overlay.
use super::*;
pub(super) fn apply(action: Action, ctx: &mut Context<'_>) {
    let s = &mut *ctx.state;
    let form = &mut *ctx.form;
    let lang = &ctx.lang;
    let info = ctx.info;
    match action {
        Action::ReportMatch => {
            if let Some(id) = info.result_id {
                if info.outcome.is_some() && !info.spectating {
                    s.page = Page::Report;
                    s.report = crate::ui::report::Flow {
                        context: json!({"context":"match","result_id":hex(&id)}),
                        ..default()
                    };
                }
            }
        }
        Action::ReportProfile(name) => {
            s.page = Page::Report;
            s.report = crate::ui::report::Flow {
                context: json!({"context":"profile","target":name.text()}),
                ..default()
            };
        }
        Action::ReportHistory(id) => {
            s.page = Page::Report;
            s.report = crate::ui::report::Flow {
                context: json!({"context":"match","result_id":hex(&id)}),
                ..default()
            };
        }
        Action::ReportEmoji(emoji) => {
            if let Some(id) = info.result_id {
                s.page = Page::Report;
                s.report = crate::ui::report::Flow {
                    context: json!({"context":"emoji","result_id":hex(&id),"emoji":hex(&emoji)}),
                    ..default()
                };
            }
        }
        Action::Reason(reason) => s.report.reason = reason.min(7),
        Action::Comment => prompt(
            s,
            form,
            *lang,
            Await::Comment,
            "p75.comment",
            TextKind::P75Text,
        ),
        Action::SendReport => {
            let body = s.report.body();
            send(s, Purpose::Report, "/api/me/reports", "POST", Some(body));
        }
        _ => {}
    }
}
