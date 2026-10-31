# Illustrated lobby and account presentation

## Result and scope

Owner-requested art follow-up on `codex/ranking-ui`, after completing the Faction molds. Seven original generated illustrations replace the lobby feature art, logo and neutral player crest and add a quiet player-status background. The four feature cards retain their labels, actions and locations. The player/status surfaces receive a restrained navy/gold finish and the existing account form receives a coherent visual presentation. The live 3D lobby diorama, game modes, navigation, News panel, locked Troops feature and competitive rules are unchanged.

The supplied screenshots are composition/style references, not instructions or screenshots to copy. The first reference defines the layout to retain; the second informs a richer painted medieval material treatment. The production layout is still rendered HTML over the existing 3D scene. No full-screen painted background or fabricated player record was introduced.

## Artwork

- `arcanas-v2`: crossed silver swords, gold strategy sigil and burgundy cloth.
- `strongholds-v2`: limestone keep with cobalt banners, oak gate and green approaches.
- `troops-v2`: silver/gold closed helmet against blue cloth.
- `ranking-v2`: crowned gold trophy and oak laurels.
- `crest-v2`: neutral knight/shield identity shared by Guest, the status panel and account entry; it does not award a ranked tier.
- `profile-v2`: low-contrast blue-hour camp, with detail concentrated away from the live statistics.
- `logo-v2`: exact TUPI / BATTLE GROUNDS wordmark, aged gold, blue pennants and a compact forged plaque.

Built-in image generation tool, 2026-10-30. [Exact production prompt set](prompts/lobby-v2.json). Selected outputs are saved in `crates/tbg-client/assets/lobby/`; the production mapping is `art.json`, and `presentation.json` records dimensions, fidelity inspection sizes and the transport budget. Private originals/normalized masters remain in `.local/art-masters/lobby-v2/`. The prior compact SVG lobby assets remain as the existing fallback set. No new dependency, external image API or large PNG master is committed.

## Normalization and optimization

`scripts/lobby_art.py` uses the established `art_pipeline.py` vtracer settings and alpha-weighted SSIM/alpha-MAE measurements. Feature masters are 768x640, crest 768x1024, quiet background 1200x640 and logo 1500x600. Transparent assets use contain without distortion/cropping, with a 2% safety margin. Opaque art uses a centered fit.

Actual vector derivatives contain paths, not embedded rasters. As with the approved Arcana/Augment pipeline, their texture/alpha fidelity fails the full-size SSIM >= .95 and display >= .97 / alpha MAE <= 1 gates. The manifest selects WebP fallbacks: Q90, method 6, twice the inspection size for opaque art; lossless at four times the inspection size for the logo and player crest. All seven served assets pass the display gates. Versions: Pillow 12.2.0, vtracer 0.6.15, numpy 2.4.6, scipy 1.18.0. [Full measurements and SHA-256](../media/lobby-v2/quality.json).

- `arcanas-v2`: 55,108 WebP bytes; SSIM 0.976243, alpha MAE 0.000000; SVG gzip 37,522 bytes.
- `strongholds-v2`: 81,064 WebP bytes; SSIM 0.973544, alpha MAE 0.000000; SVG gzip 39,193 bytes.
- `troops-v2`: 47,128 WebP bytes; SSIM 0.973482, alpha MAE 0.000000; SVG gzip 32,488 bytes.
- `ranking-v2`: 52,782 WebP bytes; SSIM 0.973738, alpha MAE 0.000000; SVG gzip 37,595 bytes.
- `crest-v2`: 416,652 WebP bytes; SSIM 0.997730, alpha MAE 0.027706; SVG gzip 39,837 bytes.
- `profile-v2`: 42,578 WebP bytes; SSIM 0.981578, alpha MAE 0.000000; SVG gzip 34,302 bytes.
- `logo-v2`: 831,090 WebP bytes; SSIM 0.996395, alpha MAE 0.042409; SVG gzip 34,413 bytes.

Total new SVG gzip + WebP: **1,781,752 bytes**, budget **3,000,000 bytes**. Only selected WebPs load in normal use; repeated crest instances share the same URL.

## Layout and account behavior

All eight measured outer anchors have exactly identical x/y/width/height before and after at 1920x1080: header, brand, Guest, navigation, modes, profile, feature row and News. [Before](../media/lobby-v2/layout-before-1920.json) / [after](../media/lobby-v2/layout-after-1920.json). Desktop inspection at 1366x768 shows the full lobby and account forms without clipping.

The account form remains centered. The six existing Rust binding IDs and the single input remain intact; Rust still owns open/close, values, submission, errors, account keys and account state. `account.js` changes presentation only: Name/Key headings, password masking with a Show/Hide key control, and Tab containment. It never reads the input value, stores a key, fetches an account API or sends an action. Each open resets key visibility. Escape continues through the existing lobby/Rust cancellation path. The literal Rust key prompt selects the sign-in presentation; a future localization of that prompt must update this presentation mapping.

## Verification

- `node --test --test-isolation=none --test-reporter=tap crates/tbg-client/tests/*.test.js`: **27 passed, 0 failed**. New account scenarios guard the one-input Rust bindings, masked/revealed/reopened key states and absence of credential reads/API calls. The lobby inventory checks all seven integrated assets and budget.
- `node --check` for both account and lobby scripts: pass. `git diff --check`: pass.
- `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` (1.17 s) and WASM target check: passed during the Faction follow-up; Rust source is unchanged throughout this task.
- `trunk build --release`: pass with the new assets and account form. `cargo build --release -p tbg-server`: pass (11.84 s), using the same source revision for the isolated local server.
- Actual release browser at 1920x1080 and 1366x768: all seven art URLs load, nine lobby image instances decode, no error-level console entries. Profile statistics and selected Stronghold remain live DOM text.
- Actual local sign-in submission with an intentionally fictitious key returns the existing server error `That key didn't work` inside the restyled form. No real credential or production database was accessed, and no account was created. Validation uses the existing raw `?server=ws://127.0.0.1:18795/ws` parameter; the development default is port 8080, and this client does not decode an encoded server query value.
- Show/Hide key changes only input masking; reopening returns to masked/empty input. Name entry uses the original 3–16-character instruction. Tab/Shift+Tab remain inside the form; Escape and Back to lobby close it. The error, creation and sign-in panels fit entirely inside 1366x768.
- Feature actions: the new Arcanas card opens the existing illustrated editor, Strongholds opens the four-kind picker, Ranking opens the existing illustrated rankings with the local empty-record state; Troops remains locked.
- Faction purchase behavior and all four colored molds were observed in a real local release Practice before this lobby follow-up; see [Faction report](FACTIONS.md).

Gallery: [lobby 1920](../media/lobby-v2/lobby-v2-1920.jpg), [lobby 1366](../media/lobby-v2/lobby-v2-1366.jpg), [sign in 1920](../media/lobby-v2/login-v2-1920.jpg), [sign in 1366](../media/lobby-v2/login-v2-1366.jpg), [create 1920](../media/lobby-v2/register-v2-1920.jpg), [create 1366](../media/lobby-v2/register-v2-1366.jpg), [server refusal](../media/lobby-v2/login-error-v2-1366.jpg).

## Delivery and limits

Owner requested commits and GitHub push only; no merge and no VPS deployment. New art is ready for owner visual review on the branch. The base game's existing small diagnostics overlay remains visible behind the header, as in the supplied first reference; this task does not modify Rust diagnostics.

The earlier unrelated native `records_insert_and_count` repetition is still stopped under AGENTS.md; see [the exact prior test evidence](../RANKING-UI.md#online-verification-blocker). No local third attempt or full-workspace rerun including that test is claimed. The draft PR retains that review note.
