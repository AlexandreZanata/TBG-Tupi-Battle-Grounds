# PORTRAITS — Shop card portraits for every buyable unit

**Status:** implemented on `phase/unit-portraits` (from `phase/battle-hud-cards`
`32adcf3`, the live release). Client assets, one capture tool and one test; no
rule, server, protocol or card CSS change.
**Owner approval needed for:** the look of the portraits (the framing follows the
original set; the figures are the current 3D models).

## What and why

Before this phase only 10 of the 25 buyable units had a Shop portrait, all from
the pre-kit models; the other 15 showed an empty frame and the server answered
404 for their picture. Now every buyable unit (25: the `FULL_POOL` kinds, never the
summons Beetling and Crate) has `assets/battle/portraits/<slug>.webp`, shot from
the current 3D models in the gallery portrait mode (`?gallery=<Kind>&portrait`,
D-29), 400 x 400 WebP on the flat `#15212a` background of the existing set. The
contract is unchanged: the card loads `<kind lowercase, no separators>.webp` and
crops it with `object-fit: cover; object-position: center 43%`.

## How to regenerate

```bash
cd crates/tbg-client && trunk build --release && cd ../..
python3 -m http.server 8765 --directory crates/tbg-client/dist &
TBG_PLAYWRIGHT=<path to playwright> node scripts/unit-portraits.cjs [Kind ...]
```

No kinds = all 25. The tool needs only Playwright and Chrome (the analysis and
the WebP encoding run in the browser; no Python or Pillow). It fits each figure
automatically: it widens the camera until nothing touches the zones the card crop
hides (at least 4 px left and right, 26 px top, 31 px bottom) and re-centres a
figure more than 25 px off centre. `TBG_UNITS_URL` points Archer and Cargo Crab
at a different build (see below).

## Framing: calibration, before and after

Measured with the tool's own rule (pixels that differ from the background):

| Set | Figure height | Top margin (min) | Bottom margin (min) | Centre offset (max) |
|---|---|---|---|---|
| Before (10 old portraits) | 57-80 % | 42 px | 34 px | 48 px (Siege Walker) |
| After (all 25) | 49-80 % | 27 px | 33 px | 21 px |

Per unit, after (figure height as a share of the image, horizontal centre offset
in px):

| Unit | Height | Offset |
|---|---|---|
| antidotebrewer | 65 % | +4 |
| archer | 80 % | +19 |
| bloodfurnace | 67 % | -5 |
| bloodweaver | 72 % | +0 |
| bloomkeeper | 64 % | +5 |
| bombard | 67 % | -7 |
| bonebrute | 58 % | +14 |
| cargocrab | 49 % | +21 |
| drumguard | 72 % | +6 |
| farwatch | 68 % | +20 |
| hexer | 63 % | +15 |
| leechroot | 64 % | -3 |
| longaxe | 67 % | -1 |
| nester | 64 % | -10 |
| oilpress | 59 % | +1 |
| oldfinn | 71 % | +1 |
| puppetmaster | 78 % | +2 |
| rampart | 70 % | +8 |
| rootcaller | 74 % | +14 |
| rotcap | 65 % | +17 |
| sawbot | 68 % | -11 |
| shatterback | 68 % | -2 |
| shieldbearer | 51 % | -4 |
| siegewalker | 69 % | -18 |
| smuggler | 66 % | -3 |

## Camera change (capture only)

`?gallery=<Kind>&portrait` accepts two optional capture parameters,
`portrait_zoom` (1 to 4, widens the view) and `portrait_x` (-2 to 2, shifts it).
Defaults reproduce the old framing exactly, so the lobby, the gallery and every
existing capture are unchanged. The fixed "large" list (Bone Brute, Siege Walker)
is kept; the tool's fit loop handles wide kits such as Oilpress, Shatterback,
Rampart and Cargo Crab.

## Archer and Cargo Crab

Both come from the **current live build** (`32adcf3`), i.e. the models that fight
today. The reworked kits on `phase/units-archer-crab-selection` were not used:
the Archer's bow is visibly broken there and the Cargo Crab rework is not
validated yet. Regenerate the two once their fixes land:

```bash
TBG_UNITS_URL=http://127.0.0.1:<port> node scripts/unit-portraits.cjs Archer CargoCrab
```

## Files and checks

- 25 files, 151 184 bytes in total, largest 8 238 bytes (budget 40 960 each), all
  400 x 400, opaque.
- `crates/tbg-client/tests/portraits.test.js` fails if any buyable unit lacks a
  portrait, or one is not 400 x 400 or exceeds 40 KB, so a new unit can never
  ship with an empty card again.
- Evidence: `docs/media/portraits/contact-sheet.jpg` (the 25 portraits) and
  `docs/media/portraits/shop-1920.jpg` (Practice at 1920 x 1080, Energy available,
  colour portraits). In a Practice match at 1920 x 1080 and 1280 x 800, 14 different
  kinds rotated through the Shop and every portrait request answered 200.

## Limitations

- Portraits are single idle poses of the base form; they do not follow Evolution.
- The two special kits will need a regeneration (above).
- Unaffordable cards show the portrait in grey by design (`shop.css`).
- Desktop only; no phone check.
