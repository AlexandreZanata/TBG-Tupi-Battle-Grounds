# TBG — Tupi Battle Grounds

Real-time auto lane strategy PvP for mobile. Players deploy and evolve units on a single battle line while combat runs live — authoritative server, free-to-play with cosmetics only.

## What it does

Competitive 1v1 matches inspired by lane-strategy mobile titles: continuous in-match shop, faction synergies, and evolution branches. Server-authoritative simulation via Nakama; Unity 2D client.

## Documentation

Start at **[docs/README.md](docs/README.md)**:

- [Modular character assembler](docs/design/modular-character-assembler.md) — **current** pipeline
- [Character content (`content/characters/library/`)](content/characters/library/)
- [Infrastructure](docs/architecture/infrastructure-overview.md)
- Legacy backup: [`archive/`](archive/) (not used by CI)

## Tools

```bash
pip install -r tools/requirements.txt
python3 -m tools.character_studio serve   # Studio + Assembler
./scripts/verify.sh
```

## For coding agents

Read **[AGENTS.md](AGENTS.md)** first. Cursor rules in `.cursor/rules/` apply automatically.

```bash
./agent-harness/resolve-rules.sh api auth game-server
```

## Verify

```bash
./scripts/verify.sh
```

## Key constraints

- **Language:** 100% English in code, docs, commits, and agent output
- **Architecture:** Authoritative Nakama match module; client sends `PlayerIntention` only
- **Monetization:** Cosmetics only — no pay-to-win
- **Art:** 2D recommended for MVP; mechanics before visuals
- **Secrets:** `.env` only — never commit credentials

## Project status

Pre-production — documentation and agent harness in place; application code not yet started.

## License

`[TBD]`

Third-party harness attribution: [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)
