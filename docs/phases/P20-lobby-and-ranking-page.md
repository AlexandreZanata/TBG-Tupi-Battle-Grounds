# P20 — Medieval lobby and ranking pages

**Depends on:** P19 reviewed.
**Branch:** `phase/p20-lobby`.
**Difficulty:** medium.

## Goal

The first thing players see feels like the game: a medieval lobby over
the living diorama, with clear **Casual** and **Ranked** sections, the
player's banner (name, rating, rank), Stronghold choice, ranking, and
settings — and the public ranking pages in the same visual language.

## Deliverable

- In-game Lobby screen replacing the menu (same actions, new layout).
- Restyled account flows (create, save key, enter key) and HTML overlay.
- Loading screen in `index.html` shown while the WASM downloads.
- `/ranking/` and `/ranking/player.html` restyled via CSS tokens + fonts.

## Read first (only these)

- `AGENTS.md`
- `docs/ART-DIRECTION.md` §3, §7
- `docs/phases/P13-global-ranking.md` (pages, API) and P19 report (kit)
- Client: `src/ui/` (menu, ranking, textform, kit), `index.html`,
  `ranking/` folder

## Do NOT

- Change the API, protocol, or any action's behavior.
- Add a web framework, bundler, or external CDN to the pages (fonts are
  self-hosted from `assets/fonts/`).

## Steps

### 1. Lobby layout
Portrait: top = player banner (heraldic shield with initial, name, rating,
rank `#12`, W-L-D); middle = two big section cards **Ranked** (rating,
"Find match") and **Casual** (Quick Match, Play with a friend, Practice);
bottom bar = Stronghold, Ranking, Settings icons. Landscape/desktop: player
banner left column, sections center, ranking preview (top 5) right column.
Guests: the banner shows `Guest` + **Create account** / **I have a key**;
the Ranked card shows the lock and the call to action.

### 2. Queue and match-found
Queue screen: section name, elapsed timer, animated war-horn icon,
**Cancel**. On `MatchStart`: a 1.2 s "versus" banner (both names, ranked
ratings, both Stronghold crests) before the countdown.

### 3. Account flows
Restyle the save-key screen as a parchment "deed" with the key in a
monospace stamp, Copy/Download as wax-seal buttons; style the HTML overlay
(`index.html` CSS) with the same fonts and colors.

### 4. Loading screen
`index.html`: inline CSS-only screen (parchment background, title, a
banner progress animation, a rotating tip from a small inline list),
removed when Bevy starts (the client hides the element on first frame).
No JS libraries; ≤ 8 KB.

### 5. Ranking pages restyle
`ranking/ranking.css`: swap tokens to the medieval theme; title in the
display font; table rows as parchment strips with rank medals for top 3
(CSS shapes, no images); player page as a heraldic card with faction bars
in faction colors. Keep Lighthouse ≥ 95 accessibility and performance.

### 6. Validation
Screenshots: lobby (guest + logged in) at 390×844, 844×390, 1920×1080;
queue; versus; save-key; loading; both ranking pages (mobile + desktop).
Usability check with one person who never played: from first load to a
casual match, and to creating an account, without help — record time and
confusions.

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass; `trunk build --release` passes
- [ ] All actions from the old menu reachable in ≤ 2 taps
- [ ] Lighthouse targets kept on ranking pages; loading screen ≤ 8 KB
- [ ] Screenshots in `docs/media/p20/`; usability notes; owner approval

## Reviewer checklist

- The lobby reuses the P19 kit (no ad-hoc styles).
- Pages still contain no `innerHTML`.

## Phase report

_Executor fills this in._
