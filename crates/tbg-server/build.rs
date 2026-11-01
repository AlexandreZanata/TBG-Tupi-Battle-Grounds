//! Embed the actual compiler target in offline deployment identity.
fn main() {
    println!(
        "cargo:rustc-env=TBG_BUILD_TARGET={}",
        std::env::var("TARGET").unwrap_or_default()
    );
}
