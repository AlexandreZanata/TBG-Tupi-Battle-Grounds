//! HTML account controls use the same coordinator and localized public state as Bevy.
use super::*;
use serde_json::json;
use std::sync::{Arc, Mutex};
use wasm_bindgen::{closure::Closure, prelude::*};

#[wasm_bindgen(module = "/assets/auth/ui.js")]
extern "C" {
    #[wasm_bindgen(js_name=bindAuthUi)]
    fn bind(callback: &JsValue);
    #[wasm_bindgen(js_name=showAuthUi)]
    fn show(public_view: &str);
}
struct Listener {
    _callback: Closure<dyn FnMut(String)>,
    actions: Arc<Mutex<Option<String>>>,
}
pub(super) fn plugin(app: &mut App) {
    let actions = Arc::new(Mutex::new(None));
    let input = actions.clone();
    let callback = Closure::new(move |action: String| {
        if action.len() <= 32 {
            if let Ok(mut slot) = input.lock() {
                *slot = Some(action);
            }
        }
    });
    bind(callback.as_ref());
    app.insert_non_send(Listener {
        _callback: callback,
        actions,
    })
    .add_systems(Update, publish.run_if(in_state(GameState::Menu)));
}
fn publish(
    listener: NonSend<Listener>,
    lang: Res<Lang>,
    mut actions: ResMut<super::super::ActionQueue>,
) {
    let view = auth::snapshot();
    if let Some(action) = listener.actions.lock().ok().and_then(|mut v| v.take()) {
        match action.as_str() {
            "continue" => {
                auth::request(AuthCommand::ContinueHere);
            }
            "guest" => {
                auth::request(AuthCommand::Guest);
            }
            "retry" => {
                auth::request(if view.status == Status::LogoutPending {
                    AuthCommand::Logout
                } else {
                    AuthCommand::Refresh
                });
            }
            "login" => actions.0.push(super::super::UiAction::HaveKey),
            "remember:true" | "remember:false" => {
                if view.remember != (action == "remember:true") {
                    auth::request(AuthCommand::ToggleRemember);
                }
            }
            _ => {}
        }
    }
    let key = super::status_key(&view).unwrap_or("");
    let choices: &[(&str, &str)] = match view.status {
        Status::Conflict => &[("continue", "auth.continue_here")],
        Status::ReauthRequired => &[("login", "lobby.account.key"), ("guest", "auth.guest")],
        Status::LogoutPending | Status::PersistenceUnavailable => &[("retry", "auth.retry")],
        Status::Guest if view.legacy_available => &[("login", "lobby.account.key")],
        _ => &[],
    };
    let public = json!({"visible":!key.is_empty(),"message":if key.is_empty(){String::new()}else{lang.key(key)},"error":view.error.map(|e|lang.key(super::failure_key(e))),"remember":view.remember,"actions":choices.iter().map(|(action,key)|json!({"action":action,"label":lang.key(key)})).collect::<Vec<_>>()});
    show(&public.to_string());
}
