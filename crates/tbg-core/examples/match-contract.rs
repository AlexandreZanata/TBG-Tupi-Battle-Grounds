//! Print the canonical match identity for release packaging without starting a server.

fn main() {
    println!("{}", tbg_core::contract::MatchContract::current().id());
}
