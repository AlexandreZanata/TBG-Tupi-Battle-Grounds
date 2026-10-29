# P16-A visual evidence

These are original game screenshots. The owner's reference images and
side-by-side comparison sheets remain in the ignored `.local/references/p16a/`.
No screenshot of an account key is included.

- `lobby-guest-*`: desktop lobby with the Ranked account call to action.
- `lobby-account-*`: desktop lobby after registration; stats come from the temporary local server.
- `arena-empty-*`: `?arena`, a presentation-only inspection view with no menu or orbit.
- `arena-busy-*`: `?arena&stress=30&measure`, thirty animated mixed-unit visuals with names and HP bars.
- `menu-phone-*` and `match-phone-*`: the existing Bevy interface at 390x844.
- `archer-lineup-*`: `?gallery=archer&lineup`, default base/Longbow/Marksman forms, without an Archer feature flag.
- `ranked-reconnect-*`: both results of a local Ranked match after reloading one client during play.

The phone-sized viewport runs on the desktop hardware. It does not establish
physical-phone FPS, touch, or 4G performance. The stress scene is a renderer
workload, not a change to the simulation's SlotCount rules.

For isolated local validation, the existing server ran on 127.0.0.1:18080 with
`TBG_DB=/tmp/tbg-p16a-validation.sqlite` and the release client directory.
The browser URLs used `?server=ws://127.0.0.1:18080/ws`; the default development
port 8080 was already occupied. No existing server or database was modified.

To replace an illustration with owner-supplied art, put the WebP in
`crates/tbg-client/assets/lobby/` and change its entry in `art.json`.
Keep each image at most 200 KB and all lobby art at most 2 MB.
