//! Command-line parsing without dependencies: one command, then
//! `--flag value` pairs; unknown flags and malformed numbers are errors.

use std::collections::BTreeMap;

pub const HELP: &str = "\
tbg-balance — offline balance lab (D-33, docs/phases/P16C-balance-lab.md)

USAGE: tbg-balance <command> [--flag value]...

COMMANDS
  run         one matchup: --a POLICY --b POLICY
  duels       Energy-equal unit duel matrix and flags
  meta        payoff matrix of the whole pool and its Nash mix
  gate        every balance row with 95 % intervals (default: confirmation seeds)
  throughput  matches per second for 1..N threads
  screen      every knob at -2 and +2 steps (screening seeds)
  search      screen, then the (1+λ) search; writes proposal.md
  confirm     canon vs --apply on confirmation seeds, every reaction time
  value       P16-D marginal value: reference | shadow | interactions | scenarios
  validate    P16-D discovery vs confirmation rank check and static baseline
  arcana      every complete Arcana preset vs the default (policy --a, default bot)

FLAGS
  --pairs N         seed pairs per matchup (each pair = 2 seat-swapped games)
  --threads N       worker threads (default: all cores)
  --seed-start S    first seed of the block (screen and search use fixed blocks)
  --reaction R      views between decisions for non-bot policies (default 5)
  --apply SPEC      ruleset changes: name=value,... (knob names from `screen`)
  --out DIR         output directory (default target/balance/<command>)
  --a, --b POLICY   policies for `run` (bot, heavy, cheap, evolver, saver,
                    faction-humans, favor-archer, random, ...)
  --generations G   search generations (default 30)
  --lambda L        children per generation (default 16)
  --top K           knobs searched, best screened first (default 24)
  --search-seed S   search randomness (default 1)
  --strict          gate exits non-zero when a required row fails
  --mode NAME       value mode: reference (default), shadow, interactions,
                    scenarios
";

const FLAGS: [&str; 14] = [
    "pairs",
    "threads",
    "seed-start",
    "reaction",
    "apply",
    "out",
    "a",
    "b",
    "generations",
    "lambda",
    "top",
    "search-seed",
    "strict",
    "mode",
];

pub struct Args {
    pub command: String,
    flags: BTreeMap<String, String>,
}

pub fn parse(args: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut command = None;
    let mut flags = BTreeMap::new();
    let mut it = args.peekable();
    while let Some(arg) = it.next() {
        if arg == "--help" || arg == "-h" {
            return Err(HELP.into());
        }
        let Some(name) = arg.strip_prefix("--") else {
            if command.replace(arg.clone()).is_some() {
                return Err(format!("unexpected argument `{arg}`"));
            }
            continue;
        };
        if !FLAGS.contains(&name) {
            return Err(format!("unknown flag `--{name}` (see --help)"));
        }
        let value = if name == "strict" {
            "true".to_string()
        } else {
            it.next().ok_or(format!("`--{name}` needs a value"))?
        };
        flags.insert(name.to_string(), value);
    }
    let command = command.ok_or_else(|| HELP.to_string())?;
    Ok(Args { command, flags })
}

impl Args {
    pub fn str(&self, name: &str) -> Option<&str> {
        self.flags.get(name).map(String::as_str)
    }

    pub fn flag(&self, name: &str) -> bool {
        self.flags.contains_key(name)
    }

    /// A positive integer flag, or `default`.
    pub fn num(&self, name: &str, default: u64) -> Result<u64, String> {
        match self.flags.get(name) {
            None => Ok(default),
            Some(v) => match v.parse::<u64>() {
                Ok(n) if n > 0 || name == "seed-start" => Ok(n),
                _ => Err(format!("`--{name} {v}`: expected a positive integer")),
            },
        }
    }
}
