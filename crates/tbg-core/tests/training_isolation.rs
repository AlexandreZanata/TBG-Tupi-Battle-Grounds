//! P62 isolation (D-80, TROOPS-TRAINING V10): the authoritative server never
//! enables or constructs the local training laboratory.

use std::path::{Path, PathBuf};

fn sources(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).expect("read dir").flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(sources(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    out
}

#[test]
fn server_never_uses_training() {
    let server = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tbg-server");
    let manifest = std::fs::read_to_string(server.join("Cargo.toml")).expect("manifest");
    assert!(
        !manifest.contains("\"training\""),
        "tbg-server enables tbg-core's `training` feature"
    );
    for path in sources(&server.join("src")) {
        let text = std::fs::read_to_string(&path).expect("read source");
        for word in ["tbg_core::training", "TrainingSession"] {
            assert!(!text.contains(word), "{}: uses `{word}`", path.display());
        }
    }
}
