//! Bounded image-file selection; platform adapters preserve the same visible action.
use std::sync::{Arc, Mutex};
static OCCUPIED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub type Output = Arc<Mutex<Option<Result<(Vec<u8>, String), ()>>>>;
#[cfg(target_os = "android")]
pub(crate) mod android;
pub fn choose() -> Output {
    let output = Output::default();
    if OCCUPIED.swap(true, std::sync::atomic::Ordering::AcqRel) {
        if let Ok(mut slot) = output.lock() {
            *slot = Some(Err(()));
        }
        return output;
    }
    let slot = output.clone();
    #[cfg(target_os = "android")]
    android::choose(slot);
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
    std::thread::spawn(move || {
        let value = (|| {
            let file = rfd::FileDialog::new()
                .add_filter("PNG / GIF / WebP", &["png", "gif", "webp"])
                .pick_file()
                .ok_or(())?;
            let meta = std::fs::metadata(&file).map_err(|_| ())?;
            if meta.len() > 1048576 {
                return Err(());
            }
            let name = file
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or(())?
                .to_owned();
            use std::io::Read;
            let mut bytes = Vec::new();
            std::fs::File::open(file)
                .map_err(|_| ())?
                .take(1048577)
                .read_to_end(&mut bytes)
                .map_err(|_| ())?;
            if bytes.len() > 1048576 {
                return Err(());
            }
            Ok((bytes, name))
        })();
        OCCUPIED.store(false, std::sync::atomic::Ordering::Release);
        if let Ok(mut slot) = slot.lock() {
            *slot = Some(value);
        }
    });
    #[cfg(target_arch = "wasm32")]
    bevy::tasks::IoTaskPool::get()
        .spawn(async move {
            let value = if let Some(file) = rfd::AsyncFileDialog::new()
                .add_filter("PNG / GIF / WebP", &["png", "gif", "webp"])
                .pick_file()
                .await
            {
                let name = file.file_name();
                let bytes = file.read().await;
                if bytes.len() <= 1048576 {
                    Ok((bytes, name))
                } else {
                    Err(())
                }
            } else {
                Err(())
            };
            OCCUPIED.store(false, std::sync::atomic::Ordering::Release);
            if let Ok(mut slot) = slot.lock() {
                *slot = Some(value);
            }
        })
        .detach();
    output
}
