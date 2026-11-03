# Official README and landing screenshots — Linux release

Owner request, 2026-11-02: use the local Linux release for real gameplay photos,
including the base and snow Arenas, War Auxiliaries, Augments, champion inspection,
tower upgrades and a complete match. Make the official website and browser game
prominent in the repository README.

## Capture source

- Official CI [native release run 41562707254](https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/actions/runs/41562707254),
  Linux artifact `tupi-battle-grounds-linux-23e8097d04acd23ad46917dc89b9741a56de4b12`
  (artifact ID `13197583272`). This is a CI release-profile package; this document
  does not claim that the version has a public GitHub Release page.
- Source: `23e8097d04acd23ad46917dc89b9741a56de4b12`; version
  `0.1.4-colossus.5`; target `x86_64-unknown-linux-gnu`; profile `release`;
  packaged features: `ui-acceptance`.
- Binary SHA-256:
  `17082166a998d45e75548d5fab25efb3e86b2a190d05c69a58d74dbda40cf48e`.
- Asset manifest SHA-256:
  `86b17180f5239e46bf1ef11f1d1b62ac154ffcf6781117b6fc4b7bc3d195ee2a`.
- Match contract: `w40-r16-f2e253b98ccb313a`. The later main change `83cffd54`
  changes server auth transaction serialization, not the captured client.
- Archive checksum, binary build metadata and every file-manifest hash/byte
  length were verified before delivery. No client compilation was needed.

## Method and coverage

The packaged native executable ran on a private Linux X11 virtual desktop at
1920 × 1080, viewed through a loopback-only VNC canvas scaled to 1280 × 720.
The browser was only the viewer: these are native Linux frames, not WASM frames.
The cursor was hidden in the viewer. Capture times use UTC on 2026-11-03
(the owner's local date was 2026-11-02, America/Cuiaba).

Both Practice matches used the ordinary UI with the Easy bot. No capture army,
Energy grant, tier injection, council fast-forward, simulation patch, or
post-production painting was used. Purchases, auxiliary hires and upgrades,
champion selection and Stronghold selection came from normal mouse input.
The bundled acceptance feature was used only to set the native window size.

- `release-war-auxiliaries`: Verdant Forest, the base Arena; both Mineiro and
  Mercador hired and evolved through tiers I–III with earned Energy. Their
  characters/structures stand outside the BattleLine; their cards show tier III.
- `release-augments`: the first Forest War Council at 1:00, with the actual four
  Augment offers. The battle paused naturally.
- `release-stronghold-panel`: clicking the player's Bastion opens Stronghold
  inspection and Fortification choices during combat.
- `release-forest-result`: the same complete Forest match ended by Stronghold
  destruction at 2:46 (defeat; seven champion purchases). The result is retained
  honestly rather than replaced with a staged victory.
- `release-tundra-battle`: ordinary Frozen Tundra gameplay with champions from
  both sides. That session also ended naturally at 3:21 by Stronghold destruction.
- `release-champion-panel`: clicking Celestial Archer in that Tundra match shows
  live statistics and both Evolution choices.
- `release-arena-catalog`: native selection UI with Forest, Tundra, Volcanic
  Forge and Ancient Ruins previews; this frame is a catalog, not an extra match.
- `release-lobby`: native lobby, including Practice, Casual, friend challenges,
  private rooms, champion collection and other navigation.

Only proportional resizing, metadata stripping and WebP quality 86 encoding
were applied. Full images remain 1280 × 720; landing thumbnails are 800 × 450.
The README and landing reuse the same image files. The existing official
`wordmark-760.webp` (760 × 304, approximately 50 KiB) is reused as the first
README image instead of adding an oversized logo.

## Validation and limits

Level: documentation/static-media work front, plus the owner's named Linux
capture sessions. No Rust build, Rust test suite, balance gate, packaging build,
production migration, VPS operation, real email or secrets were involved.

- `python3 web/scripts/check.py`: passed (manifests, local references, JSON,
  JavaScript syntax, paired translations, release gates and content sync).
- WebP dimensions and decoding, README local links and `git diff --check` are
  checked on the final tree; exact delivery results are recorded below.
- These screenshots do not certify Windows/browser pixel or motion parity,
  competitive balance, production activation or runtime performance.

A native crash was observed after the Tundra result when opening the Arena
catalog: Bevy reported a queued command targeting an entity already despawned.
The completed-match captures predate the crash. Its investigation and final
status are recorded in the delivery notes below; capture evidence alone is not
an assertion that navigation is error-free.

## Delivery notes

The complete packaged file manifest verified **413 file hashes and byte lengths**.
The eight selected frames produce 16 WebP files (full size plus thumbnails).
The native navigation incident did not recur after restarting with a backtrace:
opening Arenas from the lobby and navigating Arena catalog → champion Practice
configuration → Arena catalog both succeeded. No runtime patch was guessed.
The post-match-specific incident remains an observed, unresolved runtime issue
outside this documentation/static-media diff; it is not marked as passing.
The release screenshots certify only the frames and complete matches described
above. They are not a general native acceptance report.

Final static validation before commit:

- README local references: **30 passed** (including logo and all gallery images).
- WebP decoding and dimensions: **16 passed**; total new image bytes:
  **974764** (full images and thumbnails combined).
- Browser observation at the normal desktop viewport: pt-BR gallery and expanded
  image displayed correctly; closing restored the gallery; English switch updated
  every new caption and accessible image description. No phone checks requested.
- `python3 web/scripts/check.py`: passed after the final content changes.
- `git diff --check`: passed. No Rust tests were run for this static-only change.
