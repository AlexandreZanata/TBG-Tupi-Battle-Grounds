# Laboratory proportions and Augment card cleanup

Owner request: use the supplied laboratory screenshot's proportions, with a
smaller arena picture and standard difficulty controls above it. Remove the
misaligned green Augment shadow and all Bronze/Silver/Gold/Diamond text labels.
Retain visual rarity frames, prices and the selected-card outline.

The laboratory now uses the standard 38-pixel controls, 8-pixel gaps and a
10-pixel difficulty-note inset. The remaining height still belongs to the
centered, proportional arena picture; selector and Play stay at the bottom.
No arena asset, rule, default difficulty, selection action or crop code changes.

Augment cards no longer contain a BoxShadow component or a rarity text node.
The obsolete rarity text/color update code is removed. Frame assets, illustration,
name/description/price positions, selected border, affordability guards and
confirmation actions are unchanged. No assets or game-rule values change.

Validation level: work front with the owner's standing native build/reinstall
authorization. Runtime revision: `e0d8f4691db0427cb51bf0109adf2c4e1286351c`.
Implementation commits: `d098927` (laboratory), `e0d8f46` (Augments).

Completed checks:

- `cargo fmt -p tbg-client -- --check`, `git diff --check`: passed.
- `python3 scripts/check-desktop-ui.py`: passed, no routing errors;
  packaged visual equivalence remains unverified. Contract JSON parsing,
  unique screen IDs, required platforms and motion references passed.
- Native release build: exit 0, `Finished release profile [optimized] target(s)
  in 4m 11s`. Reused the existing cache, with the shared host lock and two jobs:

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 \
  CARGO_TARGET_DIR=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/target \
  PKG_CONFIG_PATH=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/.local/native-deps/root/usr/lib/x86_64-linux-gnu/pkgconfig \
  TBG_DEFAULT_SERVER=wss://api.tupibg.com/ws \
  bash -eu -c 'export TBG_SOURCE_COMMIT="$(git rev-parse HEAD)"; cargo build --offline --locked --release -p tbg-client --bin tbg-client --features ui-acceptance'
```

The executable was copied to commit-specific staging while the lock was held.
Linux release metadata: `0.1.2-rc.1`, x86_64, `ui-acceptance`, protocol 31,
contract `w31-r6-097385fee9a10b0f`.

Scoped Linux observations:

- `python3 /tmp/tbg-arena-cover-capture.py BINARY WORKTREE OUTPUT 1366x768
  BoneBrute practice idle celestial`: exit 0. Visually compared with the
  owner's first reference at the same dimensions: difficulty rows, gaps, note
  and arena picture follow its proportions; both bottom buttons remain visible.
- The same capture at 1920x970: exit 0; proportionate picture, controls visible,
  no flattened arena or unused bottom spacer.
- `xvfb-run -a -s '-screen 0 1920x970x24 -nolisten tcp' python3
  /tmp/tbg-augment-polish-capture.py BINARY WORKTREE OUTPUT`: exit 0.
  Real local War Council, unselected and two successive pointer selections.
  Selected-card outline and enabled confirmation were visible; no green shadow
  extends below the card, and no rarity labels are present. Frame artwork and
  prices remain. Inspected Diamond and Silver frames in the captured round.
- The initial interactive selection capture expired before input arrived;
  its later screenshots are not selection evidence. The final helper performs
  the same pointer actions immediately after its initial screenshot, before
  the real countdown expires. No simulation state or outcome was modified.
- An optional Python pixel diagnostic could not start because the default
  `python3` has no Pillow. No dependency was installed for this small adjustment;
  the scoped checks above are visual inspections, not a pixel-comparison pass.

Evidence stays local under the primary checkout's
`.local/training-augment-output/`: `laboratory-768/`, `laboratory-970/`,
`augments-selection/`, and `installed-768/`, with screenshots and renderer logs.
No screenshots, packages, binaries or supplied reference images enter git.

Packaging used `scripts/package-native.py --platform linux --binary BINARY
--output OUTPUT/package --no-archive` under the shared host lock with two jobs.
The package's `install.sh` installed
`/home/iiii/.local/opt/tupi-battle-grounds/0.1.2-rc.1-e0d8f4691db0`.
All 355 manifest entries passed size/SHA-256 verification. Both launcher links
and the quoted desktop Exec point at the new executable; its build metadata
matches the runtime revision. Executable SHA-256:
`97cef685d2dfa46cdf5076749c9324edba3524e8e6716c74d7970233528c2f3b`.
Opened the actual installed executable with its packaged assets at 1366x768:
exit 0; visually inspected the laboratory again. Existing data, user windows
and older versions were preserved. The final documentation commit does not
change executable source.

Pending consolidation, not passed: Windows/web same-source packaged full-frame/
motion comparisons; remaining locale/viewport/DPI, eight-arena and four-rarity
matrix cases; resize, keyboard focus, unaffordable and resolved states. The shared
implementation propagates to all targets but does not prove rendered parity.
Queue the existing competitive-confirmation regression once at consolidation:

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo test --locked -p tbg-client --bin tbg-client \
  br_ag05_ag11_picking_is_local_and_confirming_queues_one_intention -- --test-threads=1
```

No Cargo tests, extra target build, full suite, broad gallery or public deployment
ran. No push under the latest owner-provided AGENTS.md.
