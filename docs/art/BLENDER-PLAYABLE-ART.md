# Playable Blender Arena art

The champion figures use the existing [champion Blender pipeline](BLENDER-CHAMPIONS.md).
This front brings the eight 1v1 Arena assemblies, their Classic Stronghold
skins, four selectable Strongholds and both War Auxiliaries into the shared
game client. The 5v5 source line is outside this delivery.

## Authoring and export

1. The eight Arena study scripts build component collections and place them in
   BattleLine coordinates. The exporter evaluates collection instances,
   bevels, normals, palette materials and painted vertex colors. It batches
   terrain, water and emissive accents, leaving the three foundry gears on
   their original pivots. The Classic Stronghold skins are separate meshes
   positioned on the existing live tower roots.
2. The four [Stronghold generators](BLENDER-STRONGHOLDS.md) produce Base, A1,
   A2, B1 and B2 per seat. Body parts are grouped by damage threshold.
   The exported muzzle sockets drive the existing aim, projectile and recoil
   path. Hatches, swivels, gears and the Sentinel's reloaded bolt stay separate.
3. The [auxiliary studies](BLENDER-WAR-AUXILIARIES.md) provide four offshore
   precincts per Arena. The exporter keeps each rig and prop pivot, evaluates
   construction and work at 8 Hz, and removes duplicate spans. The runtime
   interpolates the exported transforms. The Miner preparation countdown
   comes from the authoritative view. Upgrades and Merchant deliveries select
   their tracks from the same observed events. Reduced motion holds the
   working poses and skips delivery movement.
4. The exporter rotates Blender Z-up coordinates into Bevy Y-up coordinates,
   quantizes vertex positions and normals, stores sRGB colors and indexed
   triangles, then zlib-compresses each pack. The game requests packs through
   its AssetServer on Arena selection, Stronghold appearance or auxiliary
   hire. A failed pack load uses the previous procedural art.

Regenerate from the repository root with Blender 4.5.15 LTS:

```sh
blender -b -t 2 --python-exit-code 1 --python scripts/blender_runtime/export.py -- \
  crates/tbg-client/assets/arena/blender --strongholds
```

The 96 packs total approximately 8.8 MB compressed. The eight terrain packs
contain 301,547 triangles in total (30–49k per Arena), 16 Classic skins
contain 63,028, 40 selectable seat/forms contain 170,240, and the 32 auxiliary
precincts contain 229,352. Only the selected assets are requested during play.

## Gallery

The before/after captures for Linux and the physical Android device are in
[the playable art gallery](blender-playable-gallery/README.md). Linux is the
reference. Windows, desktop web and Android render with the same shared
client. That page states the captured scenarios, the capture commit and the
scenarios still pending.

## Size and performance

- The 96 compressed packs total approximately 8.8 MB. Packs are requested on
  demand instead of at startup.
- The eight terrain packs contain 301,547 triangles, the 16 Classic skins
  63,028, the 40 seat/fortification packs 170,240 and the 32 auxiliary
  precincts 229,352.
- The Android ARM64 release APK is 127,002,880 bytes.
- A pack is requested only when its Arena is selected, its Stronghold
  appearance changes or an auxiliary is hired. A missing pack falls back to
  the previous procedural art.

## Validation

The desktop visual contract lists the affected screens and motion cases.
For a same-build comparison, use the explicit
`--capture_legacy_art` option in a debug or `ui-acceptance` client. The
comparison uses the same camera, UI and simulation path.

The desktop candidate is 0.1.4-colossus.8. The Android candidate uses revision 57, after the installed champion-only
revision 56. It is based on the same shared client. Public activation remains
subject to the existing release and compatibility checks.
