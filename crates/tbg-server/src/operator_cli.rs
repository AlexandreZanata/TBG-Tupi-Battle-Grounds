//! Local operator break-glass commands, parsed before general server configuration.
use std::io::{self, IsTerminal, Write};
pub fn run(args: Vec<String>) -> std::process::ExitCode {
    match execute(args) {
        Ok(Some(code)) => {
            println!("{code}");
            std::process::ExitCode::SUCCESS
        }
        Ok(None) => std::process::ExitCode::SUCCESS,
        Err(code) => {
            eprintln!("operator: {code}");
            std::process::ExitCode::FAILURE
        }
    }
}
fn execute(args: Vec<String>) -> Result<Option<String>, &'static str> {
    let command = args.first().ok_or("missing command")?;
    let supported = [
        "claim-code",
        "revoke-sessions",
        "disable",
        "enable",
        "set-owner-email",
    ];
    if !supported.contains(&command.as_str()) {
        return Err("unknown command");
    }
    if command != "claim-code" {
        if !io::stdin().is_terminal() {
            return Err("TTY confirmation required");
        }
        print!("Confirm operator action by typing YES: ");
        io::stdout().flush().map_err(|_| "terminal")?;
        let mut confirm = String::new();
        io::stdin()
            .read_line(&mut confirm)
            .map_err(|_| "terminal")?;
        if confirm.trim() != "YES" {
            return Err("cancelled");
        }
    }
    let operator = args
        .windows(2)
        .find(|p| p[0] == "--operator")
        .and_then(|p| p[1].parse::<i64>().ok());
    if operator.is_none() && !args.iter().any(|v| v == "--owner") && command != "set-owner-email" {
        return Err("select --owner or --operator <id>");
    }
    let email = (command == "set-owner-email")
        .then(|| args.get(1).cloned())
        .flatten();
    let path = std::env::var_os("TBG_DB").ok_or("TBG_DB required")?;
    let store =
        crate::store::Store::open(std::path::Path::new(&path)).map_err(|_| "store unavailable")?;
    let actor = format!(
        "cli:{}",
        std::env::var("USER")
            .unwrap_or_else(|_| "operator".into())
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
            .take(32)
            .collect::<String>()
    );
    let command = command.to_owned();
    let now = crate::store::release::now_unix();
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| "runtime")?
        .block_on(async move {
            store
                .release(move |db| {
                    crate::store::operators::cli::execute(
                        db,
                        &command,
                        operator,
                        email.as_deref(),
                        &actor,
                        now,
                    )
                })
                .await
                .ok_or("store unavailable")?
                .map_err(|_| "operator action refused")
        })
}
