# tests/

Cross-crate integration tests (Phase 03+). Unit tests live beside the
code they cover (`#[cfg(test)]` modules in each crate).

Policy (from the project test pyramid — unit 75% / integration 20% /
E2E 5%):

- Integration tests assert documented contracts (`docs/game-design/`,
  `tbg-protocol`), never mirror implementation internals.
- Deterministic-simulation tests use fixed seeds; same seed must yield
  the same outcome.
- No network, filesystem, or database fixtures in `tbg-core` tests.
