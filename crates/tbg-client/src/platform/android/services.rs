//! Small checked JNI calls to the matching GameActivity; payloads never enter logs.

use bevy::android::ANDROID_APP;
use jni::{
    objects::{JObject, JString},
    strings::JNIStr,
    JValue, JavaVM,
};

fn with_activity<T>(
    call: impl FnOnce(&mut jni::Env, &JObject) -> jni::errors::Result<T>,
) -> Option<T> {
    let app = ANDROID_APP.get()?;
    // AndroidApp owns both references for this call; never delete its global Activity reference.
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) };
    vm.attach_current_thread(|env| {
        let raw = app.activity_as_ptr() as jni::sys::jobject;
        let activity = unsafe { env.as_cast_raw::<JObject>(&raw)? };
        call(env, &activity)
    })
    .ok()
}
pub fn string(name: &JNIStr) -> Option<String> {
    with_activity(|env, activity| {
        let object = env
            .call_method(activity, name, jni::jni_sig!("()Ljava/lang/String;"), &[])?
            .l()?;
        let string: JString = env.cast_local::<JString>(object)?;
        string.try_to_string(env)
    })
}
pub fn flag(name: &JNIStr) -> bool {
    with_activity(|env, activity| {
        env.call_method(activity, name, jni::jni_sig!("()Z"), &[])?
            .z()
    })
    .unwrap_or(false)
}
pub fn send(name: &JNIStr, value: &str) -> bool {
    with_activity(|env, activity| {
        let text = env.new_string(value)?;
        env.call_method(
            activity,
            name,
            jni::jni_sig!("(Ljava/lang/String;)Z"),
            &[JValue::Object(&text)],
        )?
        .z()
    })
    .unwrap_or(false)
}
pub fn copy(text: &str) -> bool {
    with_activity(|env, activity| {
        let text = env.new_string(text)?;
        env.call_method(
            activity,
            jni::jni_str!("copyText"),
            jni::jni_sig!("(Ljava/lang/String;)V"),
            &[JValue::Object(&text)],
        )?;
        Ok(())
    })
    .is_some()
}
pub fn paste() -> Option<String> {
    string(jni::jni_str!("pasteText"))
}
pub fn open_page(url: &str) {
    let _ = send(jni::jni_str!("openPublicPage"), url);
}
