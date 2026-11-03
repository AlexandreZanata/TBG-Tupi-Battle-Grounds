# Landing media refresh and Android beta — 2026-11-03

The owner requested the README's current real game photos throughout the landing,
and automated Android beta publication in the existing desktop release format.
Base: main `b9574efaaf58abad0ccd63f2b8965b0f62e17018`.

## Real image sources

The eight Linux gallery images and main Tundra hero are the same bytes used by
the README. Their CI release-profile package, ordinary Forest/Tundra matches,
source and limitations are in [the Linux capture report](readme-linux-release.md).
They show client `0.1.4-colossus.5`, not a newly captured `.6` runtime.

Three complete Android frames come from the P69 API 36 x86_64 emulator, APK 54,
executable source `cd958bd03ee3b80dc8f9c86da774c1460ebc7b7f`:

- `practice-hard54-audio-combat.png`: real Hard Practice combat in Tundra.
- `lock-return54.png`: War Council after the Home/lock/unlock return.
- `colossus-b2-ability54.png`: Guest catalog's Eternal Monument, Colossus B2.

These Guest frames contain no account key or personal phone content. They are
internal-build emulator evidence, not proof of a physical ARM64 run or public
revision 55. See [P69 consolidation](../phases/P69-emulator-20261103.json) for
the actual runtime scope and pending physical-device/16 KB acceptance.

Full frames were proportionally resized to 1280 and 800 pixels wide, metadata
stripped and encoded as WebP quality 86. No painting, staged army, color grading
or replacement UI was used. The decorative Forest background is a 1280 x 400 crop
at x=0, y=80 from `release-war-auxiliaries-1280.webp`, excluding most HUD chrome
behind text. Foreground hero/gallery frames remain complete. The 1200 x 675 JPEG
social card is that full Forest frame resized. Wordmark/branding are retained.
Exact original-frame/output hashes are in [the inventory](landing-android-beta-20261103.json).

All 84 portraits and eight Arena previews retain the canonical client files.
Generated URLs include their first 16 SHA-256 characters to refresh stale caches.

## Observed local presentation

The staged landing was viewed in the in-app browser at its normal desktop size
and 390 x 844. The final Forest background, Tundra hero and eleven gallery entries
displayed. Android battle/catalog images opened in the viewer and closed back to
the page. PT/EN captions and accessible descriptions changed with the language
control. The mobile gallery retained horizontal scrolling; enlarged frames fit.

The pending Android dialog displayed Android 12+, ARM64 and landscape requirements
and offered no incompatible download. Static validation, translation parity and
media decoding passed. This is a scoped landing review, not in-game Windows or
browser pixel/motion acceptance.

## Activation boundary

The live API was observed at protocol 38; these packages use protocol 40/R16.
The catalog must reject that mismatch. VPS SSH timed out and the owner confirmed
this network cannot reach the VPS. No live landing files, backend, DB or timers
were changed. GitHub publication and prepared static artifacts are distinct from
activation on `tupibg.com`.
