# Ogres / Ogros — OG-01 authored preview

Eight new Ogres join the existing Ogre Archer (stable code `SiegeWalker`).
Rivet Ballista fills its Machine slot. The authored catalog has 72 conventional
champions, eight Factions, nine per Faction. Ogres have one Tank, three Dps,
two Ranged, two Control and one Support. The online pool remains 46;
all 26 preview additions are visibly marked on the public catalog.

Local review with actual engine portraits, five-form sheets and separate AI
reference disclosures: <http://localhost:8964/>. Landing preview:
<http://localhost:8604/#champions>. These are local previews; no public deployment.

- [Ogre Archer — five forms](previews/siegewalker-forms.webp): exact Archer range,
  measured bazooka/arm clearance, no presentation pedestal.
- [Basalt Bulwark / Bastião de Basalto](basaltbulwark.md) · [forms](previews/basaltbulwark-forms.webp).
- [Cinder Maul / Marreta de Brasa](cindermaul.md) · [forms](previews/cindermaul-forms.webp).
- [Hook Brawler / Brigão do Gancho](hookbrawler.md) · [forms](previews/hookbrawler-forms.webp).
- [Flint Runner / Corredor de Sílex](flintrunner.md) · [forms](previews/flintrunner-forms.webp).
- [Clay Binder / Molda-Barro](claybinder.md) · [forms](previews/claybinder-forms.webp).
- [Gust Drummer / Tambor do Vendaval](gustdrummer.md) · [forms](previews/gustdrummer-forms.webp).
- [Hide Mender / Remendeiro de Couro](hidemender.md) · [forms](previews/hidemender-forms.webp).
- [Pebble Slinger / Lança-Seixos](pebbleslinger.md) · [forms](previews/pebbleslinger-forms.webp).
- [Rivet Ballista / Balista de Rebites](rivetballista.md) · [forms](previews/rivetballista-forms.webp).

The [catalog](catalog.json) carries bilingual names/forms and gameplay intent.
The [prompt set](image-prompts.json) and [reference manifest](reference-manifest.json)
identify all nine original built-in image_gen sheets. Full reference PNGs stay
outside git under `.local/ogre-faction-2026-11-01/concepts` in the original checkout.
[Native preview provenance](native-previews.json) identifies each committed sheet's
actual source revision; review sheets use WebP quality 90, with the original PNG hash retained. Shop portraits use pixel-identical lossless encoding.

[Validation, D-33 evidence and limitations](../phases/OGRE-FACTION.md) distinguish
observed Linux behavior from pending Windows/game-web motion, look/fun acceptance
and full competitive balance. New values are Proposed under GAME-DESIGN OG-01.
