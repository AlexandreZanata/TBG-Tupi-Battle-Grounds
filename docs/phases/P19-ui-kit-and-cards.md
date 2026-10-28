# P19 — Medieval UI kit and standardized cards

**Depends on:** P18 reviewed.
**Branch:** `phase/p19-ui-kit`.
**Difficulty:** medium.

## Goal

One consistent, accessible medieval interface: a single **card**
component used everywhere a unit appears (shop, evolution, unit info,
stronghold choice), a themed HUD, and shared panels, buttons, and icons —
faster to read and nicer to touch on phones and desktops.

## Deliverable

- `ui/kit/`: theme tokens, panel, button, banner header, icon, and the
  `Card` component with its states.
- Fonts (OFL) and an icon atlas generated from SVG sources in the repo.
- Unit portraits rendered at runtime from the base forms, cached.
- HUD, evolution panel, unit info, toasts, and result screen rebuilt with
  the kit; a Settings panel (quality, sound, reduced motion).

## Read first (only these)

- `AGENTS.md`
- `docs/ART-DIRECTION.md` §3, §7
- Client: `src/ui/` (all), `src/overlay.rs`, `src/audio.rs`
- Bevy examples: `ui/styling/`, `ui/images/`, `ui/text/`, `ui/ui_scaling.rs`,
  `3d/render_to_texture.rs`

## Do NOT

- Change what the HUD shows or the intentions it sends (same information,
  better presentation).
- Use the display font for body text or numbers.
- Add a UI framework crate.

## Dependencies you may add

| Item | Where | Why |
|------|-------|-----|
| Fonts: Cinzel (display) and Alegreya Sans (body), OFL, Latin subset | `assets/fonts/` | themed, legible text; record in `THIRD_PARTY_NOTICES.md` |
| `resvg` CLI (tool, not a crate dependency) | dev machine | render `assets-src/icons/*.svg` to the PNG atlas |
| `web-sys` feature `Navigator` vibrate (already has `Navigator`) | client | haptic tap on Android |

Download fonts only from the official Google Fonts GitHub repository; subset
with `pyftsubset` (fonttools) to Basic Latin + Latin-1; ≤ 120 KB total.

## Steps

### 1. Theme tokens
`ui/kit/theme.rs`: colors from ART-DIRECTION §3 (UI row), font handles,
sizes in viewport units with a minimum (`body = max(2.2vmin, 16px-equiv)`),
radii, spacing scale (4/8/12/16/24), shadows (darker frame offset).
Every UI file uses tokens only — no literal colors after this phase.

### 2. Icons
`assets-src/icons/*.svg` (hand-written, two-tone, 64×64 grid): energy gem,
four faction crests (crown, skull, gear, leaf), five archetype icons
(shield, swords, bow, staff-orb, heart-leaf), slot pip, sound,
settings, back, ranking, lock, check. `scripts/icons.sh` renders them with
`resvg` into `assets/ui/icons.png` (one atlas, ≤ 60 KB) + a Rust table of
atlas indices (`ui/kit/icons.rs`).

### 3. Portraits
`ui/kit/portraits.rs`: at startup render each unit's base form (idle pose,
3/4 view, neutral team color) to a 256×256 image (192×192 on Low) with an
off-screen camera (`3d/render_to_texture.rs`), then despawn the camera.
Re-render when a unit's art changes (P22–P29 call it).

### 4. Card component
`ui/kit/card.rs`: `spawn_card(kind, size, state) -> Entity` and an update
system. Layout per ART-DIRECTION §7 (3:4; cost gem top-left; crest
top-right; portrait; name banner; archetype icon + slot pips). States:
`Affordable`, `Unaffordable` (desaturated, cost red), `Pending` (pulsing
gold outline), `Disabled`, `Selected`. Press: scale 0.95 for 80 ms, click
sound, `navigator.vibrate(10)` where supported. Long-press (existing) opens
unit info.
- Sizes: `Shop` (bottom bar; three cards fill ≤ 92 % of the short screen
  side on portrait), `Info`, `Evolution`, `Choice`.

### 5. Rebuild screens with the kit
HUD (Energy bar as a segmented gem bar with the next-gem timer, Stronghold
HP bars as banner bars, clock on a wooden plaque, synergy chips with crests
and counts), evolution panel (two `Evolution` cards with branch name and
effect text from `tbg-core::content`), unit info sheet, toasts and banners,
result screen, Settings panel (Quality Low/Medium/High/Auto, Sound,
Reduced motion — persisted in `localStorage`).

### 6. Accessibility pass
Check at 390×844 and 1920×1080: every text ≥ minimum size, contrast ≥ 4.5:1
(measure with a contrast checker on screenshots), every tap target ≥ 48 px,
team identity readable in grayscale screenshots, reduced motion disables
pulses, shakes, and the menu orbit.

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass; `trunk build --release` passes
- [ ] No literal color left in `src/ui/` (grep for `Color::srgb` outside `theme.rs`)
- [ ] First-load asset budget still ≤ 1.5 MB (fonts + icons + audio)
- [ ] Screenshots (portrait + desktop) of shop, evolution, info, result,
      settings in `docs/media/p19/`; grayscale check images included
- [ ] A full match on a phone with the new HUD; owner approves (report)

## Reviewer checklist

- One `Card` implementation; no screen builds its own card layout.
- Fonts licensed OFL and listed in `THIRD_PARTY_NOTICES.md`.

## Phase report

_Executor fills this in._
