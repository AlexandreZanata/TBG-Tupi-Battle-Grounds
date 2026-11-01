# Pyre Warden ground-line removal — 2026-11-01

Owner reference: Screenshot_2026-11-01_09-28-23.png. Remove only the yellow
line on the floor when Pyre Warden fires; preserve the approved animation.

Removed the cosmetic ground-strip spawn from `vfx/flame.rs`. The nozzle-bound
flame pieces, colors, count, motion, lifetimes, model, attack timing and event
endpoints are unchanged. The authoritative segment still determines combat;
no core source or gameplay value changes. No portrait change is needed.

Validation: affected-package formatting and `git diff --check`. The native
debug build required for the authorized reinstall will use the nonblocking
shared lock, two Cargo jobs and existing cache. No new tests or full suite
for this isolated cosmetic removal. Final build/installation evidence is local.

## Actual validation and delivery status

Formatting and `git diff --check` passed. The required native build exited75:
**deferred: shared validation slot busy**. Following AGENTS.md, no automatic
retry, polling, waiting, fresh cache or lock bypass was attempted. This is not
a compile failure or passing compilation evidence. No tests ran for this
isolated cosmetic deletion.

Pending native command, in the existing execution worktree:

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 \
  CARGO_TARGET_DIR=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/target \
  PKG_CONFIG_PATH=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/.local/native-deps/root/usr/lib/x86_64-linux-gnu/pkgconfig \
  TBG_SOURCE_COMMIT="$(git rev-parse HEAD)" \
  TBG_DEFAULT_SERVER=wss://tbg.abastevo.com.br/ws \
  cargo build --offline --locked -p tbg-client --bin tbg-client
```

After a successful build: freeze its binary/receipt, then reinstall and verify
its executable/assets hashes using the established local preview workflow.
Native reinstall remains pending; the installed Ogre Archer follow-up
`bc8ef32` was preserved. The source change is committed for GitHub delivery.
