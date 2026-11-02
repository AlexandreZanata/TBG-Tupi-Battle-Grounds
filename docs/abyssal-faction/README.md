# Abyssals, Plant replacement and organic Ogre refinement

Proposed AB-01/OG-02: Trident transfers from Plants as the Abyssal Tank.
Eight new Abyssals plus Pod Snapper preserve nine champions per Faction,
81 conventional champions total. The Nacre Bastion illustration is an unused
reserved concept and is not counted or implemented. The online FULL_POOL
remains 46; new kinds are private-preview only.

- [Nacre Duelist / Espadachim de Nácar](nacreduelist.md) — Abyssals / Dps
- [Reef Lancer / Lanceira do Recife](reeflancer.md) — Abyssals / Dps
- [Trench Fist / Punho da Fossa](trenchfist.md) — Abyssals / Dps
- [Pearl Slinger / Fundeiro de Pérolas](pearlslinger.md) — Abyssals / Ranged
- [Abyss Harpooner / Arpoadora do Abismo](abyssharpooner.md) — Abyssals / Ranged
- [Undertow Herald / Arauto da Ressaca](undertowherald.md) — Abyssals / Control
- [Trench Lamplighter / Faroleiro da Fossa](trenchlamplighter.md) — Abyssals / Control
- [Foam Weaver / Tecelã de Espuma](foamweaver.md) — Abyssals / Support
- [Pod Snapper / Estala-Vagens](podsnapper.md) — Plants / Ranged

Ogre update: all nine organic champion kits are reviewed against the owner's
Hook Brawler reference. GustDrummer keeps its stable id but becomes Barrel
Keeper / Mestre do Barril, a long-reload rolling-cask Support. See the full
[phase report](../phases/ABYSSALS-OGRE-REFINEMENT.md) and GAME-DESIGN AB-01/OG-02.

Concept sheets are illustrative built-in imagegen output; actual native
captures are separately labeled and preserve their renderer source/hash.
The prompts and reference manifest are committed here. Original PNGs reside
in the original checkout's ignored `.local/abyssals-2026-11-01/concepts`.
No generated bitmap is presented as a rendered in-game model.

Linux is the visual reference. Windows/game-web visual correspondence,
competitive balance acceptance and owner look/fun sign-off remain pending.

The current final Pearl Slinger screen is `balance-final.csv`. The earlier `balance-final-changed.csv` records the intermediate candidate before the last measured adjustment, and is not the current tuning result.

## Actual native review sheets

Each sheet contains five forms in three poses. These are engine captures,
with per-image build/source hashes in [the manifest](previews/manifest.json).
The compact images use WebP format encoding; original PNGs and ready logs are
preserved in the ignored local artifact directory.

### Refined Ogres

- [SiegeWalker](previews/siegewalker-forms.webp)
- [BasaltBulwark](previews/basaltbulwark-forms.webp)
- [CinderMaul](previews/cindermaul-forms.webp)
- [HookBrawler](previews/hookbrawler-forms.webp)
- [FlintRunner](previews/flintrunner-forms.webp)
- [ClayBinder](previews/claybinder-forms.webp)
- [GustDrummer](previews/gustdrummer-forms.webp)
- [HideMender](previews/hidemender-forms.webp)
- [PebbleSlinger](previews/pebbleslinger-forms.webp)

### Abyssals

- [Trident](previews/trident-forms.webp)
- [NacreDuelist](previews/nacreduelist-forms.webp)
- [ReefLancer](previews/reeflancer-forms.webp)
- [TrenchFist](previews/trenchfist-forms.webp)
- [PearlSlinger](previews/pearlslinger-forms.webp)
- [AbyssHarpooner](previews/abyssharpooner-forms.webp)
- [UndertowHerald](previews/undertowherald-forms.webp)
- [TrenchLamplighter](previews/trenchlamplighter-forms.webp)
- [FoamWeaver](previews/foamweaver-forms.webp)

### Plant replacement

- [PodSnapper](previews/podsnapper-forms.webp)

Local review gallery: `http://localhost:8965/`. Landing preview: `http://localhost:8604/#champions`. These loopback services are not a public deployment.

## Installed Linux witness

Preview `0.1.3-abyssals.1`, source `5553393`, wire 35 / ruleset 10. All 379 installed asset hashes matched. Open **TBG · 81 Champions Preview** or run `tbg-champions`. Five native Training scenes passed; their [screenshots and witnesses](training/manifest.json) and [validation summary](linux-validation.json) preserve the installed binary hash. The ordinary client and its data remain separate.

[Area-heal scene](training/barrel-allies.webp) · [Enemy interception](training/barrel-intercept.webp) · [Abyssals in Training](training/abyssal-crowd.webp) · [Barrel Keeper rear-heal](training/barrel-rear.webp) · [Foam Weaver rear-heal](training/foam-rear.webp).
