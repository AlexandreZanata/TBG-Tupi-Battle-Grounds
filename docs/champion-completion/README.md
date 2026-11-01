# Champion completion production catalog

Owner-authorized seventeen-champion preview based on main `0e23a1a`.
The [catalog](catalog.json) defines bilingual names, default skins, four Evolution
names, geometry, weapon/projectile, life clips, signature victory, and crowd intent.
The seventeen linked briefs are Proposed direction, not a claim that every
motion detail has passed visual review. Authoritative numeric proposals live
in [GAME-DESIGN](../GAME-DESIGN.md#champion-completion--proposed-local-preview-cc01cc17).

The [reference manifest](reference-manifest.json) records all seventeen five-form
AI concept images, stored outside git to avoid large binary assets. Their local
review gallery is <http://localhost:8963/production-gallery.html>; the directory
is `.local/champion-concepts-2026-11-01` in the original checkout. These are
reference illustrations, not imported meshes or runtime screenshots.
Gorgehowl v2 replaces the original long-neck/throat-sac concept to avoid Acidmaw's
identity: short neck, spade head, crescent ear trumpets and ribbed sternum.

Engine models use the shared palette, fixed rig-v2 nodes, merged per-joint
meshes, and five explicit forms. No textures, alternate renderer or mobile work
is introduced. Space optimization concerns visible geometry and cosmetic crowd
placement, never Shop slots, collision rules or simulation range.

Implementation, observed checks and deferred platform coverage are recorded in
[the phase report](../phases/CHAMPION-COMPLETION.md).
