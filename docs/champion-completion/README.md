# Champion completion production catalog

Owner-authorized seventeen-champion preview based on main `0e23a1a`.
The [catalog](catalog.json) defines bilingual names, default skins, four Evolution
names, geometry, weapon/projectile, life clips, signature victory, and crowd intent.
The seventeen linked briefs have shared native low-poly implementations.
Look/fun approval, competitive balance and unobserved motion/platform cases remain pending. Authoritative numeric proposals live
in [GAME-DESIGN](../GAME-DESIGN.md#champion-completion--proposed-local-preview-cc01cc17).

The [reference manifest](reference-manifest.json) records all seventeen five-form
AI concept images, stored outside git to avoid large binary assets. Their local
review gallery is <http://localhost:8963/production-gallery.html>; the directory
is `.local/champion-concepts-2026-11-01` in the original checkout. The gallery now leads with actual engine portraits and all 85 forms, with
concept illustrations clearly separated inside disclosures. The original AI
images are references, not imported meshes.
Gorgehowl v2 replaces the original long-neck/throat-sac concept to avoid Acidmaw's
identity: short neck, spade head, crescent ear trumpets and ribbed sternum.

Engine models use the shared palette, fixed rig-v2 nodes, merged per-joint
meshes, and five explicit forms. No textures, alternate renderer or mobile work
is introduced. Space optimization concerns visible geometry and cosmetic crowd
placement, never Shop slots, collision rules or simulation range.

Implementation, observed checks and deferred platform coverage are recorded in
[the phase report](../phases/CHAMPION-COMPLETION.md).


Owner rule: no body affiliation plaques, squares, ribbons or generic team
tabards, including every Evolution and summon. Blue/red foot rings identify
teams; body geometry and intrinsic materials are identical for both seats.
See [ART-DIRECTION](../ART-DIRECTION.md#2-look) and the
[body audit](body-identity-audit.json). All 340 form contracts passed.

The local artifact `engine-final/capture-manifest.json` records 67 refreshed
portraits and 17 five-form lineups with original build provenance. All 84
captures passed readiness and content-hash checks. Portrait WebP encoding
is pixel-identical to engine output and within the established 40 KiB budget.


Linux preview installed as **TBG · 17 Champions Preview**. Launch with
`~/.local/bin/tbg-champions`; use Troops for all five forms and animation
controls, then the Training editor or local Practice. The installed revision
and exact checks are recorded in the phase report. The original game/data
are preserved. Windows and desktop-web visual parity remain pending.


Current laboratory baseline: **PR44, head 9fb7be0**, installed preview
**0.1.2-cc.1**. This replaces the earlier laboratory, retains all seventeen
champions and ring-only bodies, and includes the latest P71/P73 presentation.
See [the PR44 integration/install report](../phases/CHAMPION-COMPLETION-PR44.md)
for exact source, commands, captures and deferred checks.
