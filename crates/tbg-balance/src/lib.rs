//! `tbg-balance`: the offline balance lab (D-33). Plays complete matches
//! with the server's own `step()` back to back on every core, measures
//! them against the gate, and searches for small number changes.

pub mod arcanas;
pub mod cli;
pub mod commands;
pub mod duels;
pub mod gate;
pub mod knobs;
pub mod meta;
pub mod metrics;
pub mod policy;
pub mod report;
pub mod runner;
pub mod search;
pub mod stats;
