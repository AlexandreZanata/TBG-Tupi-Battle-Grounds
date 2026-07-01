import { mountCharacterViewer } from "../viewer/viewer3d.js";
import { createArenaLocomotion, DEFAULT_WALK_MPS } from "./arenaLocomotion.js";
import { buildMotionArena } from "./arenaScene.js";
import { createArenaController } from "./arenaController.js";
import { SHOWCASE_ORDER } from "../showcase/showcaseOrder.js";
import { MOTION_ARENA_PILOT_IDS } from "./arenaPilotConfig.js";

const $ = (id) => document.getElementById(id);
const keys = { forward: false, back: false, left: false, right: false };

async function api(path) {
  const response = await fetch(path);
  const data = await response.json().catch(() => ({}));
  if (!response.ok) throw new Error(data.detail ?? response.statusText);
  return data;
}

function bindKeyboard() {
  window.addEventListener("keydown", (e) => {
    if (e.code === "KeyW") keys.forward = true;
    if (e.code === "KeyS") keys.back = true;
    if (e.code === "KeyA") keys.left = true;
    if (e.code === "KeyD") keys.right = true;
  });
  window.addEventListener("keyup", (e) => {
    if (e.code === "KeyW") keys.forward = false;
    if (e.code === "KeyS") keys.back = false;
    if (e.code === "KeyA") keys.left = false;
    if (e.code === "KeyD") keys.right = false;
  });
}

function filterArenaPilots(characters) {
  if (!MOTION_ARENA_PILOT_IDS.length) return characters;
  const allowed = new Set(MOTION_ARENA_PILOT_IDS);
  return characters.filter((c) => allowed.has(c.id));
}

function sortCharacters(characters) {
  const order = new Map(SHOWCASE_ORDER.map((id, index) => [id, index]));
  return [...characters].sort((a, b) => (order.get(a.id) ?? 9999) - (order.get(b.id) ?? 9999));
}

async function init() {
  bindKeyboard();
  let pendingReady = null;
  const viewer = mountCharacterViewer($("viewer3d"), {
    stageVariant: "arena",
    skipInitialClip: true,
    onReady: () => {
      pendingReady?.();
    },
  });
  buildMotionArena(viewer.getScene());

  let arenaLocomotion = null;
  let controller = null;
  let raf = 0;
  let last = performance.now();

  const tick = (now) => {
    raf = requestAnimationFrame(tick);
    const delta = Math.min(0.05, (now - last) / 1000);
    last = now;
    if (controller && arenaLocomotion) {
      const hasInput = keys.forward || keys.back || keys.left || keys.right;
      arenaLocomotion.updateInput(delta, hasInput);
      const speed = controller.update(delta);
      arenaLocomotion.updateSpeed(speed);
    }
  };
  requestAnimationFrame(tick);

  const characters = sortCharacters(
    filterArenaPilots((await api("/api/characters")).filter((c) => c.assetMode === "mesh3d")),
  );
  $("unitCount").textContent = `${characters.length} mesh3d`;
  const list = $("charList");
  list.innerHTML = characters
    .map(
      (c, i) =>
        `<button type="button" class="unit-item unit-item--chip" data-id="${c.id}"><span class="unit-item__index">${i + 1}</span><span class="unit-item__name">${c.characterName ?? c.id}</span></button>`,
    )
    .join("");

  async function loadUnit(id) {
    const data = await api(`/api/characters/${id}`);
    arenaLocomotion?.dispose();
    arenaLocomotion = null;
    controller = null;
    await new Promise((resolve) => {
      pendingReady = resolve;
      viewer.loadCharacter({ ...data, meshTier: "game" });
    });
    pendingReady = null;
    const root = viewer.getDisplayRoot();
    if (root) {
      arenaLocomotion = createArenaLocomotion(viewer, {
        useRootMotion: $("useRootMotion")?.checked ?? false,
        walkMps: DEFAULT_WALK_MPS,
      });
      controller = createArenaController(root, () => keys, {
        allowMove: () => arenaLocomotion.isWalkEngaged(),
      });
      viewer.setFootIkDebugEnabled($("footIkToggle")?.checked ?? false);
    }
    $("status").textContent =
      `${id} — WASD · Perry walk ~${DEFAULT_WALK_MPS} m/s · alternating foot contacts`;
    list.querySelectorAll(".unit-item").forEach((btn) => btn.classList.toggle("is-active", btn.dataset.id === id));
  }

  list.querySelectorAll(".unit-item").forEach((btn) => btn.addEventListener("click", () => loadUnit(btn.dataset.id)));
  $("footIkToggle")?.addEventListener("change", (e) => viewer.setFootIkDebugEnabled(e.target.checked));
  $("useRootMotion")?.addEventListener("change", () => {
    const active = list.querySelector(".unit-item.is-active");
    if (active) loadUnit(active.dataset.id);
  });

  const defaultId =
    MOTION_ARENA_PILOT_IDS.find((id) => characters.some((c) => c.id === id)) ??
    characters.find((c) => c.id === "demo_blocky_walk_01")?.id ??
    characters.find((c) => c.id === "demo_guard_01")?.id ??
    characters[0]?.id;
  if (defaultId) await loadUnit(defaultId);

  const health = await api("/api/health");
  $("health").textContent = `${health.status} · build ${health.studioBuild ?? "?"}`;
}

init().catch((error) => {
  $("status").textContent = error.message;
  $("status").className = "status status--error";
});
