# 5v5 Practice and Blender Valley work front

The preview starts from the 1v1 Practice panel. The 5v5 companion offers
Role selection and the existing offline host with one human Seat, four
allied bots and five opposing bots. Native clients opt in by installing
or pointing to a companion. The browser preview stages both clients on
one origin and opts in with a query parameter. See the
[testing instructions](README.md#test-from-the-1v1-practice-panel).

The feature is based on `phase/p76-moba-core` at `5fce24db`, which already
contains the deterministic MOBA mode and the initial Blender Tupi Valley
study. The arena now uses editable collection instances and a 24-piece
component board. The match loads 15 embedded mesh variants (420,822 bytes)
baked from those collections. The map and snapshot drive their placement,
fog and damage. The complete Blender assembly is available for editing.
The game generates the terrain, water, base courts, lanes and forest.
[Blender source and reproduction commands](../art/BLENDER-TUPI-VALLEY-5V5.md)

## Work-front validation

- `cargo fmt -p tbg-client -- --check` and
  `cargo fmt --manifest-path moba/Cargo.toml -p tbg-moba-client -- --check`
  passed.
- Under the shared nonblocking lock and two build jobs,
  `cargo check --offline --locked --manifest-path moba/Cargo.toml -p tbg-moba-client --bin tbg-moba`
  passed on Linux in 2.50 s using `/data/tbg-moba-target`.
- Under the same lock,
  `cargo check --offline --locked -p tbg-client --bin tbg-client`
  passed on Linux in 35.96 s using `/data/tbg-open-fronts-target`.
  The first attempt failed because the host lacked `alsa.pc`. The second
  used `PKG_CONFIG_PATH` from locally extracted ALSA and udev development
  packages linked to the installed runtime libraries.
- Under the same lock,
  `cargo check --offline --locked -p tbg-client --bin tbg-client --target wasm32-unknown-unknown`
  passed in 1m 09s with six warnings in existing config and update code.
- Blender 4.5.15 rendered the overview, match camera and component board
  with `--samples 8 --preview-width 1280`. Reopening the project with
  `verify.py` reported `Validated two scenes, 24 board assets, 2 bridges, 22 Towers, 8 Camps and 15 meshes.`
- `python3 scripts/check-desktop-ui.py`, `bash -n moba/scripts/stage-practice-web.sh`
  and `git diff --check` passed. The source gate checks shared routing.

## Consolidation checks

- Run the 1v1 Practice entry, Role selection, an offline 5v5 match with
  four allied and five opposing bots, return and a fresh 1v1 match on
  Linux, Windows and desktop web. Exercise the unavailable and failed
  companion entry states and both supported languages.
- Build the local combined browser preview with
  `moba/scripts/stage-practice-web.sh` and exercise the same-origin
  handoff, asset loading and return. The 5v5 WASM build remains pending.
- Capture the new `moba_practice_screens` and `moba_motion_screens`
  cases at the desktop contract viewports. Include both bridges, the
  plaza, all Camp approaches, Nexus deployment, and intact, damaged and
  destroyed Towers while fog changes. Exact visual correspondence
  remains pending for all three targets.
