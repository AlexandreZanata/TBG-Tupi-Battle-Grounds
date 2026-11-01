//! D-92: deployment identity must be inspectable before binding or reading production config.
use std::process::Command;

#[test]
fn build_info_never_binds_or_reads_runtime_credentials() {
    let output = Command::new(env!("CARGO_BIN_EXE_tbg-server"))
        .arg("--build-info")
        .env("TBG_ADDR", "not a socket")
        .env("TBG_RANKED_STATS", "not a valid policy")
        .env("TBG_DB", "/this/path/must/not/be/opened")
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value, tbg_server::build_info::get());
    assert_eq!(value.as_object().unwrap().len(), 6);
}
