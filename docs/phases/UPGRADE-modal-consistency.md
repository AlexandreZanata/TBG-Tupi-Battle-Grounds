# Champion and Stronghold upgrade modal consistency

Owner request: only the single second Evolution choice should expand across the
modal; preserve two initial choices. Style Stronghold Fortification like champion
Evolution, using the supplied three Linux screenshots as reference.

Changes: shared option/header/close geometry and colors; equal flex shares for two
choices, full row for one. Fortification's inline choice text replaces its badges
and coin-box layout; the unchosen branch is hidden after tier 1. Blocking feedback
remains visible when needed. The left information panels remain in their locations.
Prices, purchase conditions, pending-intent handling and authoritative actions are
unchanged. The shared desktop contract and state/motion matrix are updated.

Validation level: work front with owner-authorized native build and reinstall.
Runtime revision: `ba65ac881a9ec1384e040372d593446f31f189da`.

Completed checks:

- `cargo fmt -p tbg-client -- --check` and `git diff --check`: passed.
- `python3 scripts/check-desktop-ui.py`: passed, source routing only;
  packaged visual equivalence remains unverified. Contract JSON parsing, unique
  screen IDs and motion-screen membership passed.
- Native release command below: exit 0, `Finished release profile [optimized]
  target(s) in 5m 08s`. Build metadata reports version `0.1.2-rc.1`, protocol
  31, release profile and the runtime revision above.

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 \
  CARGO_TARGET_DIR=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/target \
  PKG_CONFIG_PATH=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/.local/native-deps/root/usr/lib/x86_64-linux-gnu/pkgconfig \
  TBG_DEFAULT_SERVER=wss://api.tupibg.com/ws \
  bash -eu -c 'export TBG_SOURCE_COMMIT="$(git rev-parse HEAD)"; cargo build --offline --locked --release -p tbg-client --bin tbg-client --features ui-acceptance'
```

The binary was copied to commit-specific local staging before releasing the lock.
The opt-in focus capture follows real purchase actions, waits for affordable
Fortification offers, and records choice widths in actual UI nodes. Captures used
`python3 /tmp/tbg-upgrade-capture.py BINARY WORKTREE OUTPUT SIZE BRANCH`:

- Branch A at 1920x970: champion and Stronghold tier 0 widths were
  `[193, 193]`; tier 1 was `[392]`; tier 2 had no enabled purchase choice.
- Branch B at 1366x768: tier 0 widths were `[152, 152]`; tier 1 was
  `[310]`; tier 2 had no enabled purchase choice, for both kinds of unit.
- Both flows exited 0 and completed champion/tower purchases, retained focus
  and outside dismissal. Representative champion tier 1, Stronghold tier 0
  and branch-B tier 1 screenshots were visually inspected.

Local evidence lives outside git in `.local/upgrade-modal-output/`, relative to
the primary checkout: `branch-a-970/`, `branch-b-768/` and `installed-970/`.
Each contains tier 0/1/2 screenshots, arena open/dismissed screenshots and
`renderer.log`. These are scoped Linux observations, not the full contract matrix.

Packaging used `scripts/package-native.py --platform linux --binary BINARY
--output OUTPUT/package --no-archive` under the same host lock and two-job
environment. The packaged `install.sh` installed:
`/home/iiii/.local/opt/tupi-battle-grounds/0.1.2-rc.1-ba65ac881a9e`.
All 355 manifest entries passed size/SHA-256 verification. Both user launchers
and the desktop entry resolve to this release; executable build metadata matches.
Installed executable SHA-256:
`84df16ff8adf1389f0c771aca5d735a4e6d3ad65195489b722a7cad9b4af78ed`.
The installed executable was separately run at 1920x970, branch A, using its
packaged assets: exit 0, the same widths and complete purchase/focus/dismissal
flow. Its Stronghold tier-1 screenshot was visually inspected. Existing user
windows, preferences and older installed releases were preserved.

Deferred: Windows/web same-source packaged full-frame/motion checks, both locale
and viewport/DPI matrices, and disabled-state cases remain for consolidation.
Source sharing and Linux observations do not establish cross-platform parity.
Queue this focused regression command once at consolidation:

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo test --locked -p tbg-client --bin tbg-client \
  ui::fortify::tests:: -- --test-threads=1
```

No full suite, balance gate or broad gallery ran. No push under the latest
owner-provided AGENTS.md. The final documentation commit records this runtime;
it does not introduce another executable revision.
