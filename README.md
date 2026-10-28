# TBG — Tupi Battle Grounds

An open-source, browser-first, real-time 1v1 strategy game. Buy units from
a rotating three-offer shop, watch them fight automatically on a single
lane, evolve them along branching paths, and stack faction synergies until
one `Stronghold` falls.

> **Status: MVP feature-complete, validated locally; not public yet.** Practice
> vs bot, online quick match and private rooms, 8 units with evolutions and
> synergies, code-built 3D figurines, effects, and sound all run in the
> browser. Pending: validation on the production VPS, human playtests, and the
> license file — see [ROADMAP](docs/ROADMAP.md).

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
docs/              GAME-DESIGN, ARCHITECTURE, ROADMAP, DECISIONS, phases/
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

## Run it locally

```bash
cargo install --locked trunk                       # once
(cd crates/tbg-client && trunk build --release)    # build the browser client
TBG_STATIC_DIR=crates/tbg-client/dist cargo run --release -p tbg-server
```

Open `http://localhost:8080/` (or `http://<your LAN IP>:8080/` on a phone).
URL options: `?autoplay=practice|quick` (a bot plays for you), `?lag=300`
(simulated latency), `?gallery` / `?gallery=N` (art review), `?stress=30`,
`?shadows=0`, `?room=CODE`. Bot load test:
`cargo run --release -p tbg-server --bin tbg-bots -- --matches 50`.
Accounts: guests play everything except ranked; "Create account" in the
menu gives you a name plus a key (your only credential — save it; the
server stores only its hash). Log in with the key on any device. Load
tests with `--accounts` need `TBG_REGISTER_PER_IP_HOUR` raised on the test
server only.
Global ranking and player stats: the **Ranking** button in the menu, or
the public page at `http://localhost:8080/ranking/` (JSON API:
`/api/leaderboard`, `/api/players/{name}`).
Deploying to a VPS: [deploy/README.md](deploy/README.md).

## Contributing

Humans and AI agents follow [AGENTS.md](AGENTS.md). Work is organized in
phases: [docs/ROADMAP.md](docs/ROADMAP.md) is the index, each phase has an
executable spec in [docs/phases/](docs/phases/).

## History

The previous Unity/Nakama/Python implementation is preserved under
[`archive/`](archive/ARCHIVE-MANIFEST.md) and in git tags
`pre-tbg20-20261027` and `pre-simplify-20261027`. Its game rules carry
over; its technology does not.

## License

No `LICENSE` file yet. MIT is the intended license; the owner adds it
before the first public release (phase P8). Third-party notices:
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
