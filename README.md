# TBG — Tupi Battle Grounds

An open-source, browser-first, real-time 1v1 strategy game. Buy units from
a rotating three-offer shop, watch them fight automatically on a single
lane, evolve them along branching paths, and stack faction synergies until
one `Stronghold` falls.

> **Status: pre-alpha, not playable yet.** The repository has been reset to
> a small Rust workspace with a complete MVP plan. Development starts at
> [ROADMAP task 1.1](docs/ROADMAP.md).

## The game

| | |
|-|-|
| Mode | Live 1v1, both players act at the same time, 3–4 minute matches |
| Board | One `BattleLine`; units walk and fight on their own |
| Economy | Regenerating `Energy` spent on units and evolutions |
| Shop | Three offers, refilled on purchase, rotating on a shared timer |
| Depth | Branching `Evolution`, faction `Synergy`, `Tank`/`DPS`/`Support`/`Ranged`/`Control` roles |
| Factions | Humans, Undead, Machines, Plants |
| Fairness | Server-authoritative; free-to-play with cosmetics only, never pay-to-win |

Full rules: [docs/GAME-DESIGN.md](docs/GAME-DESIGN.md).

## Technology

Rust everywhere: a Bevy client compiled to WebAssembly (WebGL2), a shared
deterministic rules crate, and an Axum/Tokio authoritative server, deployed
as one binary plus static files on a single small VPS. Art is built in code
as stylized toy figurines — no rigging pipeline. Details:
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) ·
[docs/DECISIONS.md](docs/DECISIONS.md).

```
crates/tbg-core    rules, content, bot, protocol (shared, deterministic)
crates/tbg-client  Bevy browser client
crates/tbg-server  authoritative game server
docs/              GAME-DESIGN, ARCHITECTURE, ROADMAP, DECISIONS
archive/           legacy Unity/Nakama/Python project (read-only)
```

## Build and check

Requires `rustup`; `rust-toolchain.toml` selects stable and installs the
`wasm32-unknown-unknown` target.

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
```

Browser builds use [Trunk](https://trunkrs.dev) from roadmap task 1.1
onward.

## Contributing

Humans and AI agents follow [AGENTS.md](AGENTS.md). Pick a task from
[docs/ROADMAP.md](docs/ROADMAP.md).

## History

The previous Unity/Nakama/Python implementation is preserved under
[`archive/`](archive/ARCHIVE-MANIFEST.md) and in git tags
`pre-tbg20-20261027` and `pre-simplify-20261027`. Its game rules carry
over; its technology does not.

## License

No `LICENSE` file yet. MIT is the intended license; the owner adds it
before the first public release (roadmap task 7.6). Third-party notices:
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
