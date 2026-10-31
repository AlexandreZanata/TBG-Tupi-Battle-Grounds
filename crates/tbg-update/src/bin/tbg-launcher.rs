//! `tbg-launcher`: the stable entry point of a managed install (P66).
//!
//! - no arguments / `run [game args…]`: start the installed game
//! - `helper <check|stage BUILD|notes|ack BUILD|status>`: JSON-lines
//!   answers for the game's update UI
//! - `install --root-json PATH --metadata-url URL --targets-url URL
//!   [--channel stable] [--no-desktop]`: trusted bootstrap or adoption
//! - `--version`

use std::ffi::OsString;

use tbg_update::layout::Layout;
use tbg_update::{helper, install, launcher};

fn layout() -> Layout {
    match Layout::default_root() {
        Some(root) => Layout::new(root),
        None => {
            eprintln!("tbg-launcher: no home directory");
            std::process::exit(1);
        }
    }
}

fn value(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn run_install(args: &[String]) -> i32 {
    let (Some(root_json), Some(metadata_url), Some(targets_url)) = (
        value(args, "--root-json"),
        value(args, "--metadata-url"),
        value(args, "--targets-url"),
    ) else {
        eprintln!("usage: tbg-launcher install --root-json PATH --metadata-url URL --targets-url URL [--channel stable] [--no-desktop]");
        return 2;
    };
    let options = install::Options {
        root_json: root_json.into(),
        source: tbg_update::trust::Source {
            metadata_url,
            targets_url,
            channel: value(args, "--channel").unwrap_or_else(|| "stable".into()),
        },
        desktop: !args.iter().any(|a| a == "--no-desktop"),
    };
    match install::install(&layout(), &options) {
        Ok(build) => {
            println!("installed {build}");
            0
        }
        Err(e) => {
            eprintln!("tbg-launcher: install failed: {e}");
            1
        }
    }
}

fn run_helper(args: &[String]) -> i32 {
    let l = layout();
    match (args.first().map(String::as_str), args.get(1)) {
        (Some("check"), _) => helper::check(&l),
        (Some("stage"), Some(build)) => helper::stage(&l, build),
        (Some("notes"), _) => helper::notes(&l),
        (Some("ack"), Some(build)) => helper::ack(&l, build),
        (Some("status"), _) => helper::status(&l),
        _ => {
            eprintln!("usage: tbg-launcher helper <check|stage BUILD|notes|ack BUILD|status>");
            2
        }
    }
}

fn main() {
    let raw: Vec<OsString> = std::env::args_os().skip(1).collect();
    let args: Vec<String> = raw
        .iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let code = match args.first().map(String::as_str) {
        Some("--version") => {
            println!(
                "tbg-launcher {} · updater ABI {} · bootstrap {}",
                env!("CARGO_PKG_VERSION"),
                tbg_update::descriptor::UPDATER_ABI,
                tbg_update::descriptor::BOOTSTRAP
            );
            0
        }
        Some("helper") => run_helper(&args[1..]),
        Some("install") => run_install(&args[1..]),
        Some("run") => launcher::run(&layout(), &raw[1..]),
        _ => launcher::run(&layout(), &raw),
    };
    std::process::exit(code);
}
