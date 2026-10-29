# Desktop battle HUD — owner follow-up to P16-A

P16-A was finalized at `05f2e41` before this follow-up began. The draw-call
ceiling was removed at the owner's request in D-28. This change implements
the separately supplied battle HUD reference, without advancing P16 or P19.

## Reference

`.local/references/desktop-battle-hud/battle-hud-reference.png`

This is the owner's attached `Low-Poly Island Battle Arena.png`, kept local
and ignored. Comparison sheets containing it also remain in that folder.

## Presentation contract

- Desktop: landscape at least 1280x720, matching the lobby breakpoint.
- The upper iron-and-gold frame contains the player names, blue/red Stronghold
  HP bars and match clock. Both bars show actual current HP and the canonical
  maximum, including the partially empty portion of the bar.
- The Energy panel uses the same frame and a purple meter, numeric current/max
  Energy, and Army SlotCount. It contains no lightning icon.
- The three Shop cards contain real base-model portraits, unit names,
  Archetype, SlotCount, tier label, Energy cost, and the real rotation meter.
  Unaffordable cards remain inspectable; pending purchases are marked.
- Surrender remains two-step and moves to the upper right. Ranked's loss
  confirmation, countdown/overtime, Synergy, long-press/right-click inspect,
  and canvas unit selection retain their behavior.
- All values come from the client's existing authoritative view, using its
  `you` seat to select own/enemy HP. No game rule, protocol, or server change.
- The existing Bevy HUD is the fallback on phones/tablets, native builds, or
  when the HTML layer is unavailable. Debug metrics remain opt-in over the
  desktop battle HUD with `?measure`.

## Portrait regeneration

Open `?gallery=<unit>&portrait` at 400x400. It renders one real base form in
its existing rig and material against the card's studio background, without
arena, floor, title, or animation cycling. Save the browser capture as WebP
under `crates/tbg-client/assets/battle/portraits/<unit-id-lowercase>.webp`.
All eight units must have a portrait. Screenshots are presentation assets,
not a replacement for the existing figure models.

## Validation

All 96 workspace tests passed, including the seat-relative Stronghold HP
scenario. Formatting, Clippy with warnings denied, the WASM target check,
JavaScript syntax, and the release Trunk build passed. Server integration
tests needed local socket access outside the restricted sandbox; the first
sandbox run reported `Operation not permitted`, and the permitted rerun
passed. Full test output is in `docs/media/desktop-battle-hud/workspace-tests.txt`.

Browser inspection covered 1920x1080, 1280x800, and a 390x844 phone viewport.
The desktop HUD displayed real damage (including 1872, 793, and 1950 HP),
with the remaining blue fill proportional to 4500 HP. Two online guests
showed mirrored 1950/4500 and 4500/4500 values from opposite seats. An
online purchase changed Energy from 8.5 to 5.6 and Army from 0 to 1; the
new offer loaded its matching portrait. Inspection, reconnect after reload,
and two-step surrender completed, with opposite Victory/Defeat results.
Keyboard purchase retained card focus. The phone viewport hid the HTML HUD
and displayed the existing Bevy controls; resizing back restored the desktop
HUD. No browser warnings or errors were observed.

All eight base portraits were inspected. They total 50,846 bytes; all three
images in the observed live offers loaded successfully. The three local
WOFF2 fonts total 51,544 bytes. Release WASM compressed with Brotli quality 9
is 5,561,758 bytes; all static assets total about 290 KB. These are size
measurements, not a network transfer or physical-device performance claim.

Final screenshots and measurements are in `docs/media/desktop-battle-hud/`.
Owner visual approval and testing on a physical phone remain pending.
