# Current Blender screenshots — 2026-11-15

These are actual Linux engine renders, not painted or assembled gameplay.
The shared image changes do not establish Windows or desktop web runtime parity.

## Shared portraits and Arena previews

All 88 non-Crate base portraits (84 champions and four summons) were captured
at 400×400 from the verified GitHub native artifact with source
36b49b229b66e1e3afada11c5696655e2ef40564, version .10, R16, release profile
and `ui-acceptance`. Its model sources match the current candidate; the later
Stronghold-only numerical revision does not change those champion images.
Lossless WebP encoding passed exact decoded-pixel comparison and the existing
40 KiB limit for every portrait. The Linux contact sheet was inspected.
See [portrait provenance](../media/blender-refresh-20261115/portraits.json).

All eight Arena scenes were rendered at 1536×764 by the identified .10/R17
debug client at 7d46426e52eccd6a5e6080073f96be40b6437974. Each passed
scene/image/text readiness and no-fallback checks. The 768×382 thumbnails use
WebP quality 85, matching the existing asset dimensions. The contact sheet was
inspected with account controls absent from the exported art.
See [Arena provenance](../media/blender-refresh-20261115/arenas.json).

## Website and README frames

Nine 1920×1080 native frames cover Lobby, Arena catalog, Laboratory/Colossus B2,
Forest and Tundra Practice, champion inspection, Stronghold inspection,
War Council and a natural Practice result. Sources are
7d46426e52eccd6a5e6080073f96be40b6437974 and
28f94749891425f3c19aee8b8d80273b14c1544c, version .10/R17, debug profile,
`ui-acceptance`. The final champion/Fortification/Laboratory frames use the
latter source, including the corrected numeric catalog and capture helpers.
Runtime asset digests record the shared Arena texture refresh.

Each process used a private Xvfb display, disposable data/XDG directories and
a private D-Bus with an empty test vault. The desktop vault was never accessed.
The server endpoint was a closed loopback port; Practice ran the normal shared
simulation. Menu panels opened through ordinary UI actions; autoplay submitted
normal bot intentions. No army, Energy, tiers or outcome was injected.
The result frame observes an intentionally idle own Practice seat: zero
purchases, natural Stronghold destruction, 4:36 combat time (370.35 seconds
wall time including loading, countdown and Council pauses). This is not the
required live browser Ranked completion.

The public frames are full-frame 1280×720 and 800×450 WebP resizes at quality
82, preserving HUD and attribution. The hero crops the real 1920×600 scene
region beginning at y=120 and resizes to 1280×400. The social image resizes
the Forest frame to 1200×675 JPEG at quality 85. Nothing was repainted, erased
or composited. Unique filenames avoid old screenshot cache entries.
[Per-frame source, binary, PNG and derivative hashes](../media/blender-refresh-20261115/site.json).

Rejected frames remain outside Git: an early Arena set included account
controls; an incorrect export flag hid menu UI; the first Fortification frame
showed stale catalog descriptions; an early champion selection died before
capture. Those photographs are not delivered.

## Limits

The static site checks passed, including references, dimensions/budgets,
i18n, generated content and release identity. The desktop browser loaded all nine local gallery images and opened the
Portuguese Laboratory lightbox successfully. This verifies the static site,
not the in-game web presentation. Windows and desktop web in-game
observations, exact visual correspondence and corrected production Ranked
purchase/reconnection/natural completion are pending. The .10 candidate is
not a published or competitive-balance-approved release.
