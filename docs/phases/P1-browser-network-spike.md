# P1 — Browser + network spike

**Depends on:** P0. **May run in parallel with:** P2 (different crates).
**Branch:** `phase/p1-browser-network-spike`.
**Difficulty:** medium. The build setup (step 1) is the hardest part.

## Goal

Prove early that (a) the Bevy 0.20 client builds for WebAssembly, renders
with WebGL2, and takes touch input in real desktop and phone browsers, and
(b) the browser reaches the Rust server over WebSocket with a measured
round-trip time. Nothing here is gameplay.

## Deliverable

- `trunk serve` page: a placeholder arena (ground, lane strip, two
  Stronghold blocks); tap/click spawns a capsule that walks the lane; an
  overlay shows FPS, frame time, and RTT to the server.
- `tbg-server` answers `GET /health` and echoes WebSocket binary frames on
  `GET /ws`.
- `.github/workflows/ci.yml`.
- Measured sizes and FPS in the phase report.

## Read first (only these)

- `AGENTS.md`
- `docs/ARCHITECTURE.md` §2, §6, §13
- Bevy 0.20 examples on disk (your memory of Bevy is probably from an older
  version — **the examples are the source of truth**; `PbrBundle`,
  `TextBundle`, etc. no longer exist). Folder:
  `~/.cargo/registry/src/index.crates.io-*/bevy-0.20.0/examples/`
  - `3d/3d_scene.rs` — camera, light, meshes, materials
  - `window/window_settings.rs` — `fit_canvas_to_parent`, window options
  - `window/window_resizing.rs` — reacting to resize
  - `input/touch_input.rs`, `input/mouse_input.rs`
  - `picking/mesh_picking.rs` — clicking meshes
  - `ui/text/text.rs` — on-screen text, FPS display
  - `diagnostics/log_diagnostics.rs` — `FrameTimeDiagnosticsPlugin`
- `ewebsock` 0.8 source: `~/.cargo/registry/src/index.crates.io-*/ewebsock-0.8.0/src/lib.rs`

## Do NOT

- Add gameplay, menus, matchmaking, art, or audio.
- Add crates other than: `ewebsock` (client), axum `ws` feature (server),
  `web-sys` only if step 6 needs the page URL (features `Window`, `Location`).
- Use `HashMap`, threads, or async in the client.

## Steps

### 1. Web build pipeline

Files: `crates/tbg-client/Cargo.toml`, `crates/tbg-client/index.html`,
`crates/tbg-client/Trunk.toml`, `crates/tbg-client/src/main.rs`, root `Cargo.toml`.

1. Install tools if missing: `cargo install --locked trunk`.
2. In `crates/tbg-client/Cargo.toml` change bevy to
   `bevy = { version = "0.20", default-features = false, features = ["3d", "ui"] }`.
3. Root `Cargo.toml`, append:
   ```toml
   [profile.dev.package."*"]
   opt-level = 2

   [profile.release]
   codegen-units = 1
   lto = "thin"
   ```
4. `crates/tbg-client/index.html`:
   ```html
   <!doctype html>
   <html lang="en">
   <head>
     <meta charset="utf-8">
     <meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no">
     <title>Tupi Battle Grounds</title>
     <link data-trunk rel="rust" data-bin="tbg-client" data-wasm-opt="z" />
     <style>
       html, body { margin: 0; height: 100%; overflow: hidden; background: #1d2b36; touch-action: none; }
       canvas#bevy { display: block; width: 100%; height: 100%; outline: none; }
     </style>
   </head>
   <body><canvas id="bevy"></canvas></body>
   </html>
   ```
5. `crates/tbg-client/Trunk.toml`:
   ```toml
   [build]
   target = "index.html"
   dist = "dist"

   [serve]
   addresses = ["0.0.0.0"]
   port = 8000
   ```
   If Trunk rejects a key, run `trunk serve --help` and use the equivalent option.
6. `src/main.rs`: `App::new().add_plugins(DefaultPlugins.set(WindowPlugin { primary_window: Some(Window { title: "Tupi Battle Grounds".into(), canvas: Some("#bevy".into()), fit_canvas_to_parent: true, ..default() }), ..default() }))`,
   a clear color `#A9D8F0`, a `Camera3d`, and nothing else yet. Check field
   names against `window/window_settings.rs`.
7. (Reviewer pre-check on 2026-10-27: Bevy 0.20 with `["3d", "ui"]` plus
   `ewebsock` 0.8 passed `cargo check --target wasm32-unknown-unknown`
   without extra crates.) If the WASM build still fails mentioning `getrandom`, add to the client
   `Cargo.toml` exactly what the error message recommends for the
   `wasm32-unknown-unknown` target, scoped with
   `[target.'cfg(target_arch = "wasm32")'.dependencies]`, and record it as a
   deviation.

Check:
```bash
cargo check -p tbg-client --target wasm32-unknown-unknown
cd crates/tbg-client && trunk build --release
```
*Done when* `trunk serve` shows a sky-blue full-window canvas in Chrome
with no errors in the browser console.

### 2. Camera and placeholder arena

Files: `src/main.rs`, new `src/camera.rs`, new `src/arena.rs`.

- World axes: the lane runs along world **X** from `x = 0.0` (own
  Stronghold) to `x = 16.0` (enemy Stronghold); `y` is up; lane width 3.0
  along **Z** (`z ∈ [-1.5, 1.5]`). 1 world unit = 1 tile.
- `arena.rs` spawns: ground plane 24 × 10 centered at `(8, 0, 0)`, color
  `#7BB661`; lane strip 16 × 3 at `y = 0.01`, color `#C9A46A`; two
  Stronghold cuboids 1.6 × 2.0 × 2.4 at `x = -0.8` (color `#3FA9F5`) and
  `x = 16.8` (color `#F2545B`); one `DirectionalLight` with shadows,
  pointing down at about 50°; ambient light at a low brightness.
- `camera.rs`: perspective camera, vertical FOV 35°, looking at the lane
  center `(8, 0, 0)` from above at about 55° pitch.
  - **Landscape** (window width ≥ height): the lane is horizontal on
    screen, own Stronghold on the left. Camera sits on the −Z side looking
    toward +Z, tilted down.
  - **Portrait** (height > width): rotate the camera 90° around the Y axis
    through the lane center so the lane is vertical, own Stronghold at the
    bottom.
  - Choose the camera distance so the whole lane (x from −2 to 18) fits
    with about 10% margin. Recompute on every `WindowResized` event.

*Done when* in Chrome devtools device mode both 360×640 (portrait) and
1920×1080 (landscape) show the whole lane, own Stronghold bottom/left.

### 3. Tap/click to spawn walkers

Files: new `src/walkers.rs`.

- Any left click or touch release anywhere spawns a capsule (radius 0.3,
  height 0.8, random-ish color from a fixed list of 4) at `(0.5, 0.7, z)`
  where `z` cycles through `-0.8, 0.0, 0.8`.
- Each capsule moves +X at 1.0 tile/s and despawns at `x ≥ 15.5`.
- Use `ButtonInput<MouseButton>` and `Touches` resources (see the input examples).

*Done when* mouse clicks and real touch on a phone both spawn walkers.

### 4. Diagnostics overlay

Files: new `src/overlay.rs`.

- Add `FrameTimeDiagnosticsPlugin`. A `Text` node in the top-left corner,
  updated twice per second: `FPS 60 | 16.7 ms | RTT -- ms | walkers 3`.
- URL parameter `?stress=N` spawns N walkers at startup at staggered x
  positions, and keeps the count at N by respawning at `x = 0.5`.
- Read URL parameters through one helper `fn url_param(name: &str) ->
  Option<String>` in new `src/config.rs`: on `wasm32` use
  `web_sys::window().location().search()`; on native read the environment
  variable `TBG_<NAME uppercased>`.

*Done when* `?stress=30` shows 30 walkers and the FPS line updates.

### 5. Server: health and echo WebSocket

Files: `crates/tbg-server/Cargo.toml`, `crates/tbg-server/src/main.rs`.

- axum dependency becomes `axum = { version = "0.8", features = ["ws"] }`.
- Routes: `GET /health` → `200 ok` (plain text); `GET /ws` → WebSocket
  upgrade with `.max_message_size(1024).max_frame_size(1024)`; every
  received `Message::Binary(bytes)` is sent back unchanged; other message
  types are ignored; the task ends when the socket closes.
- Listen address from env `TBG_ADDR`, default `0.0.0.0:8080`. Print
  `tbg-server listening on <addr>` at startup.
- axum 0.8 notes: `Message::Binary` holds `bytes::Bytes` (use
  `axum::body::Bytes`); serve with
  `axum::serve(listener, app).await` where
  `listener = tokio::net::TcpListener::bind(addr).await`.

Check: `cargo run -p tbg-server` then `curl -s localhost:8080/health` → `ok`.

### 6. Client ping over WebSocket

Files: new `src/net.rs`, `src/overlay.rs`, client `Cargo.toml` (add `ewebsock = "0.8"`).

- Server URL rule (put it in `config.rs` as `fn server_url() -> String`):
  1. `?server=<url>` if present;
  2. on wasm, if the page protocol is `https:` → `wss://<page host>/ws`;
  3. on wasm otherwise → `ws://<page hostname>:8080/ws`;
  4. native → `ws://127.0.0.1:8080/ws`.
- `ewebsock::connect(url, ewebsock::Options::default())` returns
  `(WsSender, WsReceiver)`. These are **not** `Send + Sync` on the web, so
  store them in a struct inserted with `app.insert_non_send_resource(...)`
  and access it with `NonSendMut<T>` in systems.
- Every second send `WsMessage::Binary` containing the current time in
  milliseconds as 8 little-endian bytes (`f64` from `Time::elapsed_secs_f64() * 1000.0`).
  On receiving a binary echo, RTT = now − decoded time. Show the latest
  RTT in the overlay. On `WsEvent::Closed`/`Error`, show `RTT offline` and
  try to reconnect every 2 seconds.

*Done when* the overlay shows a stable RTT in the desktop browser and on a
phone on the same Wi-Fi (page served from the PC's LAN IP).

### 7. CI workflow

File: `.github/workflows/ci.yml`. On `push` and `pull_request`: checkout,
`dtolnay/rust-toolchain@stable` with `wasm32-unknown-unknown`,
`Swatinem/rust-cache@v2`, then the four commands from `AGENTS.md`
"Verification". On Ubuntu, Bevy needs system packages before `cargo check`:
`sudo apt-get update && sudo apt-get install -y libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev`.
Run the same commands locally and fix every clippy finding.

### 8. Device matrix and measurements

1. `cd crates/tbg-client && trunk build --release`.
2. Sizes: `ls -l dist/*.wasm`, `gzip -9 -c dist/*.wasm | wc -c`, and
   `brotli -c dist/*.wasm | wc -c` if `brotli` is installed.
3. Repeat the build with `opt-level = "s"` added under `[profile.release]`;
   record both; keep the smaller one unless its FPS is more than 15% lower.
4. Serve on the LAN: `trunk serve --release`; run `tbg-server` too.
5. The owner opens `http://<PC LAN IP>:8000/?stress=30` on: Chrome desktop,
   Firefox desktop, Safari (if a Mac is available), Android Chrome, iOS
   Safari. Record FPS after 20 s, time until the arena appears, RTT.

## Tests to write

None. This phase is proven by builds and devices.

## Acceptance checklist

- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`,
      `cargo test --workspace`, WASM check — all pass
- [ ] `trunk build --release` passes
- [ ] Renders and accepts input on every available target in step 8
- [ ] ≥ 30 FPS with `?stress=30` on both phones
- [ ] Compressed WASM ≤ 6 MB (if larger: report it; do not try to fix it here)
- [ ] RTT visible on a phone over the LAN

## Risks and what to do

| Problem | Action |
|---------|--------|
| Black screen on iPhone | Check Safari console via a Mac if possible; try disabling shadows; report exact errors |
| WASM much larger than 6 MB | Report the numbers; the reviewer decides on feature trimming |
| Phone cannot reach `:8080` | Check the PC firewall allows ports 8000 and 8080 on the LAN |
| `ws://` blocked | Only happens on `https` pages; use plain `http` on the LAN in this phase |

## Reviewer checklist

- Only the allowed dependencies were added; versions are pinned to minor.
- `NonSend` used for the socket; no `unwrap` on received network data.
- Camera framing is correct in both orientations (check screenshots).
- Measurements are real (sizes match a fresh `trunk build --release`).
- If any target failed to render: decide on D-01 (DECISIONS.md) before P3.

## Phase report

*(Executor fills this in at the end.)*

- Commands run and results:
- WASM size raw / gzip / brotli (opt 3 vs "s"):
- Device table (target · renders? · FPS @30 · load time · RTT):
- Deviations:
- Blocked items:
