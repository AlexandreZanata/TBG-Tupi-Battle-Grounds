TBG 0.1.0-native.2 — consolidated Linux preview, protocol 29.

- Shared native header, polished lobby, account, Ranking, Settings, Arcanas,
  Strongholds, Arenas, battle HUD, cards, Fortification and War Council.
- All 46 conventional champions and five refined arenas; 50 actual-model
  portraits cover the Shop roster and animated summons.
- Troops catalog with real 3D form/animation previews, team editor,
  wooden targets and local training meters. Practice difficulty selection
  includes an explicitly assisted Impossible profile (bot regeneration ×2).
- Winner-only arena celebration, including dead participants, repeated
  champions and last Evolution forms, paged in groups of up to 64.
- Same authoritative server as browser players. Protocol 29 clients are
  required together; older clients must update. Historical ranked data is
  preserved across the storage/summary migration.

Linux x86_64 preview. The locally published build targets Ubuntu 24.04 or a
compatible desktop with Vulkan/OpenGL drivers, ALSA and glibc 2.39. The
AppImage bundles userspace dependencies while retaining host graphics drivers.
Extract the tar.gz and run ./install.sh to add the TBG application-menu
launcher, or launch the AppImage. Both formats have SHA-256 files and embedded
build-info.json provenance. The installer preserves preferences and older
installed versions.

This is a prerelease candidate from the consolidation PR, published before
manual merge. The complete roster's numeric content remains Proposed:
competitive balance acceptance, measured difficulty ordering and owner
look/fun sign-off are pending. Targeted checks are recorded in
`docs/phases/CONSOLIDATED-LINUX-RELEASE.md`; the full workspace suite and long
balance gates were not requested. Windows and macOS artifacts are not part
of this Linux release.

The TBG Visible Attribution License permits commercial reuse with persistent
on-screen creator/profile/repository credit. Third-party notices retain their
own terms. Existing kit Victory movements are shared with Dance previews;
per-character dance signatures are prepared for future artwork.
