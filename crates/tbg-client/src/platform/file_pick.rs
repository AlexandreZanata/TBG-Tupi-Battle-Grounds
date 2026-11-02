//! Bounded image-file selection; platform adapters preserve the same visible action.
use std::sync::{Arc, Mutex};
pub type Output = Arc<Mutex<Option<Result<(Vec<u8>, String), ()>>>>;
pub fn choose() -> Output {
    let output = Output::default();
    let slot = output.clone();
    #[cfg(not(target_arch = "wasm32"))]
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
            let bytes = std::fs::read(file).map_err(|_| ())?;
            if bytes.len() > 1048576 {
                return Err(());
            }
            Ok((bytes, name))
        })();
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
            if let Ok(mut slot) = slot.lock() {
                *slot = Some(value);
            }
        })
        .detach();
    output
}
