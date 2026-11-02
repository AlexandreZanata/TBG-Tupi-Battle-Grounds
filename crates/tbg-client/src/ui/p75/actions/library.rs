//! Library actions for the shared P75 overlay.
use super::*;
pub(super) fn apply(action: Action, ctx: &mut Context<'_>) {
    let s = &mut *ctx.state;
    match action {
        Action::Slot(slot) => s.library.slot = slot.min(2),
        Action::Standard(id) => {
            let mut slots = s.data["slots"].as_array().cloned().unwrap_or_else(|| {
                vec![
                    json!({"standard":0}),
                    json!({"standard":1}),
                    json!({"standard":2}),
                ]
            });
            slots[usize::from(s.library.slot)] = json!({"standard":id});
            send(
                s,
                Purpose::Library,
                "/api/me/emoji/slots",
                "PUT",
                Some(json!({"slots":slots})),
            );
        }
        Action::EquipCustom(id) => {
            let mut slots = s.data["slots"].as_array().cloned().unwrap_or_else(|| {
                vec![
                    json!({"standard":0}),
                    json!({"standard":1}),
                    json!({"standard":2}),
                ]
            });
            slots[usize::from(s.library.slot)] = json!({"custom":hex(&id)});
            send(
                s,
                Purpose::Library,
                "/api/me/emoji/slots",
                "PUT",
                Some(json!({"slots":slots})),
            );
        }
        _ => {}
    }
}
