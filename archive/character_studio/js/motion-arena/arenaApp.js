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

function hasMovementInput() {
  return keys.forward || keys.back || keys.left || keys.right;
}

async function init() {
  bindKeyboard();
  let pendingReady = null;
  let activeUnitId = null;
  let walkPreviewPinned = false;
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
  let last = performance.now();

  function walkClipName() {
    return $("useRootMotion")?.checked ? "WalkRoot" : "Walk";
  }

  function startWalkPreview() {
    walkPreviewPinned = true;
    const clip = walkClipName();
    const ok = viewer.playClip(clip, true);
    viewer.setLocomotionPreview(true);
    viewer.setClipTimeScale(1);
    const clips = viewer.listClipNames().join(", ") || "none";
    $("status").textContent = ok
      ? `${activeUnitId} — ${clip} loop · WASD to move · clips: ${clips}`
      : `${activeUnitId} — Walk clip missing (bind & export first). Found: ${clips}`;
    $("status").className = ok ? "status" : "status status--error";
  }

  function attachLocomotion(root) {
    arenaLocomotion = createArenaLocomotion(viewer, {
      useRootMotion: $("useRootMotion")?.checked ?? false,
      walkMps: DEFAULT_WALK_MPS,
    });
    controller = createArenaController(root, () => keys, {
      allowMove: () => arenaLocomotion.isWalkEngaged(),
    });
    viewer.setFootIkDebugEnabled($("footIkToggle")?.checked ?? false);
  }

  const tick = (now) => {
    requestAnimationFrame(tick);
    const delta = Math.min(0.05, (now - last) / 1000);
    last = now;
    if (!controller || !arenaLocomotion) return;

    const hasInput = hasMovementInput();
    if (hasInput) walkPreviewPinned = false;

    if (walkPreviewPinned && !hasInput) {
      viewer.setClipTimeScale(1);
      return;
    }

    arenaLocomotion.updateInput(delta, hasInput);
    const speed = controller.update(delta);
    arenaLocomotion.updateSpeed(speed);
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

  async function loadUnit(id, { autoWalk = false } = {}) {
    activeUnitId = id;
    walkPreviewPinned = false;
    window.location.hash = id;
    arenaLocomotion?.dispose();
    arenaLocomotion = null;
    controller = null;
    const data = await api(`/api/characters/${id}`);
    await new Promise((resolve) => {
      pendingReady = resolve;
      viewer.loadCharacter({ ...data, meshTier: "game" });
    });
    pendingReady = null;
    const root = viewer.getDisplayRoot();
    if (root) attachLocomotion(root);

    list.querySelectorAll(".unit-item").forEach((btn) => btn.classList.toggle("is-active", btn.dataset.id === id));
    $("status").textContent = `${id} — click ▶ Walk preview or hold WASD to walk`;
    $("status").className = "status";
    if (autoWalk) startWalkPreview();
  }

  list.querySelectorAll(".unit-item").forEach((btn) => btn.addEventListener("click", () => loadUnit(btn.dataset.id)));
  $("btnWalkPreview")?.addEventListener("click", () => startWalkPreview());
  $("footIkToggle")?.addEventListener("change", (e) => viewer.setFootIkDebugEnabled(e.target.checked));
  $("useRootMotion")?.addEventListener("change", () => {
    if (activeUnitId) loadUnit(activeUnitId, { autoWalk: walkPreviewPinned });
  });

  const hashId = window.location.hash.replace("#", "");
  const defaultId =
    (hashId && characters.some((c) => c.id === hashId) ? hashId : null) ??
    characters.find((c) => c.id === "demo_guard_01")?.id ??
    MOTION_ARENA_PILOT_IDS.find((id) => characters.some((c) => c.id === id)) ??
    characters.find((c) => c.id === "demo_blocky_walk_01")?.id ??
    characters[0]?.id;

  if (defaultId) {
    const autoWalk = hashId === defaultId;
    await loadUnit(defaultId, { autoWalk });
  }

  const health = await api("/api/health");
  $("health").textContent = `${health.status} · build ${health.studioBuild ?? "?"}`;
}

init().catch((error) => {
  $("status").textContent = error.message;
  $("status").className = "status status--error";
});
