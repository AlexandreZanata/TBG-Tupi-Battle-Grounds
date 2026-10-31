//! HTML text entry overlay (P11, D-20): name/key input through the one
//! real DOM `<input>` in `index.html`, so phones get their keyboard, paste,
//! and suggestions. No JS in the page; Bevy owns the listeners, which live
//! in the `NonSend` resource below (never `Closure::forget`). The client
//! has no keyboard controls, so there is nothing to suppress while open.

use bevy::prelude::*;
use std::sync::{Arc, Mutex};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

/// What the open form is collecting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextKind {
    Name,
    Key,
    Room,
}

/// A submitted value; the opener stays open until it calls `close` (so a
/// server error can be shown in the form via `set_error`).
#[derive(Message)]
pub struct TextEntered {
    pub kind: TextKind,
    pub value: String,
}

#[derive(Message)]
pub struct TextCancelled;

/// Request state (`NonSend`: JS closures are not `Send`; store with
/// `insert_non_send`, touch through `NonSendMut`). Slots bridge to `poll`.
#[derive(Default)]
pub struct ActiveForm {
    pub(super) kind: Option<TextKind>,
    pub(super) edit: super::kit::edit::Edit,
    pub(super) masked: bool,
    pub(super) focus: usize,
    pub(super) prompt: String,
    pub(super) error: Arc<Mutex<String>>,
    submitted: Arc<Mutex<Option<String>>>,
    cancelled: Arc<Mutex<bool>>,
    #[cfg(target_arch = "wasm32")]
    dom: Option<Dom>,
}

pub fn plugin(app: &mut App) {
    app.add_message::<TextEntered>()
        .add_message::<TextCancelled>()
        .insert_non_send(ActiveForm::default())
        .add_systems(Startup, wire_dom)
        .add_systems(Update, poll)
        .add_plugins(super::textform_bevy::plugin);
}

/// Show the form for `kind` with `prompt` as its label.
pub fn open(form: &mut ActiveForm, kind: TextKind, prompt: &str) {
    form.kind = Some(kind);
    form.edit = Default::default();
    form.focus = 0;
    form.masked = kind == TextKind::Key;
    form.prompt = prompt.into();
    *form.error.lock().unwrap_or_else(|e| e.into_inner()) = String::new();
    *form.submitted.lock().unwrap_or_else(|e| e.into_inner()) = None;
    *form.cancelled.lock().unwrap_or_else(|e| e.into_inner()) = false;
    #[cfg(target_arch = "wasm32")]
    if let Some(dom) = &form.dom {
        if bevy_form() {
            return;
        }
        dom.label.set_text_content(Some(prompt));
        // The lobby styles the dialog by this marker, never by the
        // (localized) label text.
        let marker = match kind {
            TextKind::Name => "name",
            TextKind::Key => "key",
        };
        let _ = dom.form.set_attribute("data-kind", marker);
        set_error_text(dom, None);
        dom.input.set_value("");
        dom.form.set_hidden(false);
        let _ = dom.input.focus();
    }
}

pub fn is_open(form: &ActiveForm) -> bool {
    form.kind.is_some()
}

/// Hide the form and forget what it was collecting.
pub fn close(form: &mut ActiveForm) {
    form.kind = None;
    #[cfg(target_arch = "wasm32")]
    if let Some(dom) = &form.dom {
        dom.form.set_hidden(true);
    }
}

/// Show a server-side refusal inside the open form; the form stays open.
pub fn set_error(form: &ActiveForm, msg: &str) {
    *form.error.lock().unwrap_or_else(|e| e.into_inner()) = msg.into();
    #[cfg(target_arch = "wasm32")]
    if let Some(dom) = &form.dom {
        set_error_text(dom, Some(msg));
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (form, msg);
}

fn poll(
    mut form: NonSendMut<ActiveForm>,
    mut entered: MessageWriter<TextEntered>,
    mut cancelled: MessageWriter<TextCancelled>,
) {
    if form.kind.is_none() {
        return;
    }
    let cancel = std::mem::take(&mut *form.cancelled.lock().unwrap_or_else(|e| e.into_inner()));
    if cancel {
        close(&mut form);
        cancelled.write(TextCancelled);
        return;
    }
    let submitted = form
        .submitted
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take();
    if let (Some(value), Some(kind)) = (submitted, form.kind) {
        entered.write(TextEntered { kind, value });
    }
}

pub(super) fn submit(form: &ActiveForm) {
    *form.submitted.lock().unwrap_or_else(|e| e.into_inner()) =
        Some(form.edit.text.trim().to_string());
}

pub(super) fn cancel(form: &ActiveForm) {
    *form.cancelled.lock().unwrap_or_else(|e| e.into_inner()) = true;
}

#[cfg(target_arch = "wasm32")]
struct Dom {
    form: web_sys::HtmlElement,
    label: web_sys::HtmlElement,
    input: web_sys::HtmlInputElement,
    error: web_sys::HtmlElement,
    _submit: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
    _cancel: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
}

#[cfg(target_arch = "wasm32")]
fn set_error_text(dom: &Dom, msg: Option<&str>) {
    dom.error.set_text_content(msg);
    dom.error.set_hidden(msg.is_none());
}

#[cfg(target_arch = "wasm32")]
fn element<T: wasm_bindgen::JsCast>(doc: &web_sys::Document, id: &str) -> Option<T> {
    doc.get_element_by_id(id)?.dyn_into::<T>().ok()
}

/// Attach permanent listeners once; they bridge into `ActiveForm`'s slots.
#[cfg(target_arch = "wasm32")]
fn wire_dom(mut form: NonSendMut<ActiveForm>) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(doc) = window.document() else {
        return;
    };
    let (Some(form_el), Some(label), Some(input), Some(error)) = (
        element::<web_sys::HtmlElement>(&doc, "tbg-form"),
        element::<web_sys::HtmlElement>(&doc, "tbg-form-label"),
        element::<web_sys::HtmlInputElement>(&doc, "tbg-input"),
        element::<web_sys::HtmlElement>(&doc, "tbg-form-error"),
    ) else {
        return;
    };
    let submitted = form.submitted.clone();
    let submit = wasm_bindgen::closure::Closure::new(move |ev: web_sys::Event| {
        ev.prevent_default();
        if let Some(input) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("tbg-input"))
            .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
        {
            *submitted.lock().unwrap_or_else(|e| e.into_inner()) = Some(input.value());
        }
        // A new attempt hides the previous refusal: an old "That name is
        // taken" must never stay under a different name while it is checked.
        if let Some(error) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("tbg-form-error"))
            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
        {
            error.set_text_content(None);
            error.set_hidden(true);
        }
    });
    form_el
        .add_event_listener_with_callback("submit", submit.as_ref().unchecked_ref())
        .ok();
    let cancelled = form.cancelled.clone();
    let cancel = wasm_bindgen::closure::Closure::new(move |_: web_sys::Event| {
        *cancelled.lock().unwrap_or_else(|e| e.into_inner()) = true;
    });
    if let Some(btn) = doc.get_element_by_id("tbg-form-cancel") {
        btn.add_event_listener_with_callback("click", cancel.as_ref().unchecked_ref())
            .ok();
    }
    form.dom = Some(Dom {
        form: form_el,
        label,
        input,
        error,
        _submit: submit,
        _cancel: cancel,
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn wire_dom() {}

/// Mobile web keeps the existing DOM input so its virtual keyboard opens (D-20).
pub(super) fn bevy_form() -> bool {
    if !crate::config::bevy_ui() {
        return false;
    }
    #[cfg(target_arch = "wasm32")]
    {
        return web_sys::window()
            .and_then(|w| w.inner_width().ok())
            .and_then(|v| v.as_f64())
            .is_some_and(|width| width >= 900.0);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        true
    }
}
