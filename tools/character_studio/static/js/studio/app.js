import { mountCharacterViewer } from "../viewer/viewer3d.js";
import { playBehaviorAction } from "./behaviorPlayback.js";

const ARCHETYPES = ["Tank", "DPS", "Support", "Ranged", "Control"];
const FACTIONS = ["Humans", "Undead", "Machines", "Plants"];

const $ = (id) => document.getElementById(id);

let selectedId = null;
let characterCache = {};
let viewer = null;
let activePreviewTab = "viewport3d";

function fillSelect(id, values) {
  $(id).innerHTML = values.map((v) => `<option value="${v}">${v}</option>`).join("");
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

function setCardPreviewVisible(visible) {
  $("cardPreview").hidden = !visible;
  $("previewEmpty").hidden = visible;
}

function setViewerVisible(visible) {
  $("viewer3d").hidden = !visible;
  $("viewer3dControls").hidden = !visible;
  $("viewer3dEmpty").hidden = visible;
  $("viewer3dLoading").hidden = true;
}

async function api(path, options = {}) {
  const response = await fetch(path, {
    headers: { "Content-Type": "application/json" },
    ...options,
  });

  if (response.status === 204) return null;

  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(data.detail ? JSON.stringify(data.detail) : response.statusText);
  }

  return data;
}

function colorInputFromHex(hex) {
  return hex.startsWith("#") ? hex : `#${hex}`;
}

function readEditForm() {
  const cached = characterCache[$("id").value] || {};
  return {
    id: $("id").value,
    characterName: $("characterName").value,
    archetype: $("archetype").value,
    faction: $("faction").value,
    heightScale: parseFloat($("heightScale").value),
    bodyWidthScale: parseFloat($("bodyWidthScale").value),
    headScale: parseFloat($("headScale").value),
    limbLengthScale: parseFloat($("limbLengthScale").value),
    primaryColor: $("primaryColor").value,
    secondaryColor: $("secondaryColor").value,
    accessories: cached.accessories || [],
    evolutionStage: parseInt($("evolutionStage").value, 10),
    assetMode: cached.assetMode || "procedural",
    mesh: cached.mesh || null,
    textures: cached.textures || null,
    materialProfile: cached.materialProfile || null,
    behaviorProfile: cached.behaviorProfile || null,
  };
}

function fillEditForm(data) {
  characterCache[data.id] = data;
  $("id").value = data.id;
  $("editId").textContent = data.id;
  $("characterName").value = data.characterName;
  $("archetype").value = data.archetype;
  $("faction").value = data.faction;
  $("heightScale").value = data.heightScale;
  $("bodyWidthScale").value = data.bodyWidthScale;
  $("headScale").value = data.headScale;
  $("limbLengthScale").value = data.limbLengthScale;
  $("primaryColor").value = colorInputFromHex(data.primaryColor);
  $("secondaryColor").value = colorInputFromHex(data.secondaryColor);
  $("evolutionStage").value = data.evolutionStage;
  updateMesh3dPanel(data);
  refreshViewer(data);
}

function updateMesh3dPanel(data) {
  const assetMode = data?.assetMode || "procedural";
  const panel = $("mesh3dPanel");
  const badge = $("assetModeBadge");
  const link = $("meshPathLink");

  badge.textContent = assetMode;
  badge.className =
    assetMode === "mesh3d" ? "unit-item__badge unit-item__badge--mesh3d" : "unit-item__badge";

  panel.hidden = assetMode !== "mesh3d";
  if (assetMode === "mesh3d" && data?.mesh) {
    link.textContent = data.mesh;
    link.href = `/api/characters/${data.id}/mesh`;
    link.title = data.mesh;
  } else {
    link.textContent = "—";
    link.removeAttribute("href");
    link.removeAttribute("title");
  }
}

function cardUrl(id) {
  return `/api/characters/${id}/card?t=${Date.now()}`;
}

function renderUnitItem(character) {
  const data = characterCache[character.id];
  const name = data?.characterName || character.id;
  const active = character.id === selectedId ? " is-active" : "";
  const assetMode = data?.assetMode || character.assetMode || "procedural";
  const badgeClass =
    assetMode === "mesh3d" ? "unit-item__badge unit-item__badge--mesh3d" : "unit-item__badge";
  const meshHint =
    assetMode === "mesh3d" && (data?.mesh || character.mesh)
      ? `<span class="unit-item__mesh">${data?.mesh || character.mesh}</span>`
      : "";

  return `
    <button type="button" class="unit-item${active}" data-id="${character.id}" role="listitem">
      <div class="unit-item__thumb">
        <img src="${character.cardUrl}?t=${Date.now()}" alt="" />
      </div>
      <div class="unit-item__meta">
        <span class="unit-item__name">${name}</span>
        <span class="unit-item__id">${character.id}</span>
        <span class="${badgeClass}">${assetMode}</span>
        ${meshHint}
      </div>
    </button>
  `;
}

function switchPreviewTab(tab) {
  activePreviewTab = tab;
  document.querySelectorAll(".preview-tabs__btn").forEach((btn) => {
    btn.classList.toggle("is-active", btn.dataset.tab === tab);
  });
  $("viewport3dPanel").hidden = tab !== "viewport3d";
  $("cardPanel").hidden = tab !== "card";
}

function ensureViewer() {
  if (viewer) return viewer;
  viewer = mountCharacterViewer($("viewer3d"), {
    onLoading: () => {
      $("viewer3dLoading").hidden = false;
      $("viewer3dLoading").textContent = "Loading 3D…";
      $("viewer3dEmpty").hidden = true;
    },
    onProgress: (message) => {
      $("viewer3dLoading").hidden = false;
      $("viewer3dLoading").textContent = message;
    },
    onReady: () => {
      $("viewer3dLoading").hidden = true;
      setViewerVisible(true);
    },
    onError: (error) => {
      $("viewer3dLoading").hidden = true;
      setViewerVisible(false);
      $("viewer3dEmpty").hidden = false;
      $("viewer3dEmpty").textContent = error?.message || "3D preview failed";
    },
  });
  $("autoRotate").addEventListener("change", (e) => viewer.setAutoRotate(e.target.checked));
  $("btnPlayIdle").addEventListener("click", () => playAction("idle"));
  $("btnPlayAttack").addEventListener("click", () => playAction("attack"));
  $("btnPlayDeath").addEventListener("click", () => playAction("death"));
  return viewer;
}

async function playAction(action) {
  if (!viewer || !selectedId) return;
  const character = { ...characterCache[selectedId], ...readEditForm() };
  await playBehaviorAction(viewer, character, action);
}

function refreshViewer(data) {
  if (!data) {
    setViewerVisible(false);
    $("viewer3dEmpty").hidden = false;
    return;
  }
  ensureViewer().loadCharacter(data);
}

async function loadList() {
  const characters = await api("/api/characters");
  $("unitCount").textContent = `${characters.length} total`;

  await Promise.all(
    characters.map(async (character) => {
      if (!characterCache[character.id]) {
        characterCache[character.id] = await api(`/api/characters/${character.id}`);
      }
    })
  );

  const list = $("charList");
  list.innerHTML = characters.map(renderUnitItem).join("");

  list.querySelectorAll(".unit-item").forEach((item) => {
    item.addEventListener("click", () => selectCharacter(item.dataset.id));
  });
}

async function selectCharacter(id) {
  selectedId = id;
  const data = await api(`/api/characters/${id}`);
  fillEditForm(data);
  $("cardPreview").src = cardUrl(id);
  setCardPreviewVisible(true);
  await loadList();
  setStatus(`Loaded ${id} — drag to orbit, scroll to zoom`);
}

async function createCharacter() {
  const body = {
    id: $("newId").value.trim(),
    archetype: $("newArchetype").value,
    faction: $("newFaction").value,
    characterName: $("newName").value.trim() || null,
  };

  await api("/api/characters", { method: "POST", body: JSON.stringify(body) });
  $("newId").value = "";
  $("newName").value = "";
  await loadList();
  await selectCharacter(body.id);
  setStatus(`Created ${body.id}`);
}

async function saveCharacter(event) {
  event.preventDefault();
  const body = readEditForm();
  await api(`/api/characters/${body.id}`, { method: "PUT", body: JSON.stringify(body) });
  const reloaded = await api(`/api/characters/${body.id}`);
  characterCache[body.id] = reloaded;
  await selectCharacter(body.id);
  setStatus(`Saved ${body.id}`);
}

async function deleteCharacter() {
  const id = $("id").value;
  if (!id || !confirm(`Delete ${id}?`)) return;

  await api(`/api/characters/${id}`, { method: "DELETE" });
  delete characterCache[id];
  selectedId = null;
  $("editForm").reset();
  $("editId").textContent = "—";
  $("cardPreview").removeAttribute("src");
  setCardPreviewVisible(false);
  setViewerVisible(false);
  $("viewer3dEmpty").hidden = false;
  await loadList();
  setStatus(`Deleted ${id}`);
}

async function init() {
  fillSelect("newArchetype", ARCHETYPES);
  fillSelect("newFaction", FACTIONS);
  fillSelect("archetype", ARCHETYPES);
  fillSelect("faction", FACTIONS);
  setCardPreviewVisible(false);
  setViewerVisible(false);

  document.querySelectorAll(".preview-tabs__btn").forEach((btn) => {
    btn.addEventListener("click", () => switchPreviewTab(btn.dataset.tab));
  });

  $("btnCreate").addEventListener("click", () => createCharacter().catch((err) => setStatus(err.message, true)));
  $("editForm").addEventListener("submit", (event) => saveCharacter(event).catch((err) => setStatus(err.message, true)));
  $("btnDelete").addEventListener("click", () => deleteCharacter().catch((err) => setStatus(err.message, true)));

  ["heightScale", "bodyWidthScale", "headScale", "limbLengthScale", "primaryColor", "secondaryColor"].forEach((id) => {
    $(id).addEventListener("change", () => {
      if (!selectedId) return;
      const draft = { ...characterCache[selectedId], ...readEditForm() };
      refreshViewer(draft);
    });
  });

  try {
    const health = await api("/api/health");
    setHealth(health.status);

    await loadList();
    await selectCharacter("demo_scout_01");
  } catch (error) {
    setHealth("offline", true);
    setStatus(error.message, true);
  }
}

init();
