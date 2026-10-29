//! `tbg-balance` entry point: parse, dispatch, report errors as exit 1.

fn main() {
    let result = tbg_balance::cli::parse(std::env::args().skip(1))
        .and_then(|args| tbg_balance::commands::dispatch(&args));
    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
