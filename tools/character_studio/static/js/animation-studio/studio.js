import { mountCharacterViewer } from "../viewer/viewer3d.js";
import { createMeshTierToggle } from "./meshTierToggle.js";
import { createClipControls } from "./clipControls.js";
import { renderMeshBadge } from "./meshBadge.js";
import { bindTimelineSync } from "./timelineSync.js";
import { loadBehaviorTimings } from "./behaviorTimings.js";

const $ = (id) => document.getElementById(id);

let selectedId = null;
let meshTier = "game";
let characterCache = {};
let viewer = null;
let clipControls = null;
let stopTimeline = null;

async function api(path) {
  const response = await fetch(path, { headers: { "Content-Type": "application/json" } });
  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(data.detail ? JSON.stringify(data.detail) : response.statusText);
  }
  return data;
}

function setStatus(message, isError = false) {
  const status = $("status");
  status.textContent = message;
  status.className = isError ? "status status--error" : "status";
}

function setHealth(state, isError = false) {
  const health = $("health");
  health.textContent = state;
  health.className = isError ? "badge badge--error" : "badge badge--ok";
}

function setViewerVisible(visible) {
  $("viewer3d").hidden = !visible;
  $("viewer3dControls").hidden = !visible;
  $("viewer3dEmpty").hidden = visible;
  $("viewer3dLoading").hidden = true;
}

function renderUnitItem(character) {
  const data = characterCache[character.id];
  const name = data?.characterName || character.id;
  const active = character.id === selectedId ? " is-active" : "";
  const badge = data?.animationSet ? "unit-item__badge unit-item__badge--mesh3d" : "unit-item__badge";

  return `
    <button type="button" class="unit-item${active}" data-id="${character.id}" role="listitem">
      <div class="unit-item__thumb">
        <img src="${character.cardUrl}?t=${Date.now()}" alt="" />
      </div>
      <div class="unit-item__meta">
        <span class="unit-item__name">${name}</span>
        <span class="unit-item__id">${character.id}</span>
        <span class="${badge}">${data?.animationSet ?? "mesh3d"}</span>
      </div>
    </button>
  `;
}

async function refreshMeshInfo(unitId) {
  try {
    renderMeshBadge($("meshBadge"), await api(`/api/characters/${unitId}/mesh-info?tier=${meshTier}`));
  } catch {
    renderMeshBadge($("meshBadge"), null);
  }
}

function ensureViewer() {
  if (viewer) return viewer;
  viewer = mountCharacterViewer($("viewer3d"), {
    onLoading: () => {
      $("viewer3dLoading").hidden = false;
      $("viewer3dEmpty").hidden = true;
    },
    onProgress: (message) => {
      $("viewer3dLoading").hidden = false;
      $("viewer3dLoading").textContent = message;
    },
    onReady: () => {
      $("viewer3dLoading").hidden = true;
      setViewerVisible(true);
      stopTimeline?.();
      stopTimeline = bindTimelineSync(viewer, $("timelineSlider"), $("timelineTime"));
    },
    onError: (error) => {
      $("viewer3dLoading").hidden = true;
      setViewerVisible(false);
      $("viewer3dEmpty").hidden = false;
      $("viewer3dEmpty").textContent = error?.message || "3D preview failed";
    },
  });
  $("autoRotate").addEventListener("change", (e) => viewer.setAutoRotate(e.target.checked));
  return viewer;
}

function getContext() {
  const data = characterCache[selectedId];
  if (!data) return { viewer, character: null };
  return { viewer, character: { ...data, meshTier } };
}

async function loadCharacterIntoViewer() {
  const data = characterCache[selectedId];
  if (!data) return;
  ensureViewer().loadCharacter({ ...data, meshTier });
  await refreshMeshInfo(selectedId);
  await loadBehaviorTimings($("behaviorTimings"), data.behaviorProfile);
  clipControls?.setActive("idle");
}

async function loadList() {
  const characters = (await api("/api/characters")).filter((c) => c.assetMode === "mesh3d");
  $("unitCount").textContent = `${characters.length} mesh3d`;

  await Promise.all(
    characters.map(async (character) => {
      if (!characterCache[character.id]) {
        characterCache[character.id] = await api(`/api/characters/${character.id}`);
      }
    }),
  );

  const list = $("charList");
  list.innerHTML = characters.map(renderUnitItem).join("");
  list.querySelectorAll(".unit-item").forEach((item) => {
    item.addEventListener("click", () => selectCharacter(item.dataset.id));
  });
}

async function selectCharacter(id) {
  selectedId = id;
  characterCache[id] = await api(`/api/characters/${id}`);
  $("editId").textContent = id;
  await loadList();
  await loadCharacterIntoViewer();
  setStatus(`Loaded ${id} (${meshTier} tier) — drag to orbit`);
}

async function init() {
  setViewerVisible(false);
  clipControls = createClipControls($("clipControls"), getContext);
  createMeshTierToggle($("meshTierToggle"), async (tier) => {
    meshTier = tier;
    if (selectedId) await loadCharacterIntoViewer();
  });

  try {
    setHealth((await api("/api/health")).status);
    await loadList();
    const ids = Object.keys(characterCache);
    const defaultId = ids.includes("demo_sentinel_01") ? "demo_sentinel_01" : ids[0];
    if (defaultId) await selectCharacter(defaultId);
  } catch (error) {
    setHealth("offline", true);
    setStatus(error.message, true);
  }
}

init();
