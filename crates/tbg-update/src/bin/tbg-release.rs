//! `tbg-release`: release repository tooling (P66 Step 7).
//!
//! - `init-keys --keys DIR`: one Ed25519 key per TUF role (never overwrites)
//! - `init-repo --keys DIR --repo DIR [--root-days 365]`
//! - `publish --keys DIR --repo DIR --package FILE --notes FILE
//!    --descriptor FILE [--previous-contract ID] [--days 30]`
//! - `check-notes --notes FILE --build ID [--rules-changed]`

use tbg_update::publish::{self, Release};
use tbg_update::{descriptor::ReleaseDescriptor, fsx, notes::Notes};

fn value(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn need(args: &[String], name: &str) -> Result<String, String> {
    value(args, name).ok_or_else(|| format!("missing {name}"))
}

fn path(args: &[String], name: &str) -> Result<std::path::PathBuf, String> {
    let path = std::path::PathBuf::from(need(args, name)?);
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path))
    }
}

fn days(args: &[String], name: &str, default: i64) -> i64 {
    value(args, name)
        .and_then(|v| v.parse().ok())
        .filter(|d| *d > 0)
        .unwrap_or(default)
}

async fn run(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("init-keys") => publish::init_keys(path(args, "--keys")?.as_ref()),
        Some("init-repo") => {
            publish::init_repo(
                path(args, "--keys")?.as_ref(),
                path(args, "--repo")?.as_ref(),
                days(args, "--root-days", 365),
            )
            .await
        }
        Some("publish") => {
            let descriptor: ReleaseDescriptor =
                fsx::read_json(path(args, "--descriptor")?.as_ref(), 64 * 1024)
                    .map_err(|e| e.to_string())?;
            let release = Release {
                package: path(args, "--package")?,
                notes: path(args, "--notes")?,
                descriptor,
                previous_contract: value(args, "--previous-contract"),
                metadata_days: days(args, "--days", 30),
            };
            publish::publish(
                path(args, "--keys")?.as_ref(),
                path(args, "--repo")?.as_ref(),
                &release,
            )
            .await
        }
        Some("check-notes") => {
            let notes: Notes = fsx::read_json(
                path(args, "--notes")?.as_ref(),
                tbg_update::limits::NOTES_BYTES,
            )
            .map_err(|e| e.to_string())?;
            notes.validate(
                &need(args, "--build")?,
                args.iter().any(|a| a == "--rules-changed"),
            )
        }
        _ => Err("usage: tbg-release <init-keys|init-repo|publish|check-notes> …".into()),
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = run(&args).await {
        eprintln!("tbg-release: {e}");
        std::process::exit(1);
    }
}
