const ARCHETYPES = ["Tank", "DPS", "Support", "Ranged", "Control"];
const FACTIONS = ["Humans", "Undead", "Machines", "Plants"];

const $ = (id) => document.getElementById(id);

let selectedId = null;
let characterCache = {};

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

function setPreviewVisible(visible) {
  $("cardPreview").hidden = !visible;
  $("previewEmpty").hidden = visible;
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
    accessories: [],
    evolutionStage: parseInt($("evolutionStage").value, 10),
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

  return `
    <button type="button" class="unit-item${active}" data-id="${character.id}" role="listitem">
      <div class="unit-item__thumb">
        <img src="${character.cardUrl}?t=${Date.now()}" alt="" />
      </div>
      <div class="unit-item__meta">
        <span class="unit-item__name">${name}</span>
        <span class="unit-item__id">${character.id}</span>
        <span class="${badgeClass}">${assetMode}</span>
      </div>
    </button>
  `;
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
  fillEditForm(await api(`/api/characters/${id}`));
  $("cardPreview").src = cardUrl(id);
  setPreviewVisible(true);
  await loadList();
  setStatus(`Loaded ${id}`);
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
  characterCache[body.id] = body;
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
  setPreviewVisible(false);
  await loadList();
  setStatus(`Deleted ${id}`);
}

async function init() {
  fillSelect("newArchetype", ARCHETYPES);
  fillSelect("newFaction", FACTIONS);
  fillSelect("archetype", ARCHETYPES);
  fillSelect("faction", FACTIONS);
  setPreviewVisible(false);

  $("btnCreate").addEventListener("click", () => createCharacter().catch((err) => setStatus(err.message, true)));
  $("editForm").addEventListener("submit", (event) => saveCharacter(event).catch((err) => setStatus(err.message, true)));
  $("btnDelete").addEventListener("click", () => deleteCharacter().catch((err) => setStatus(err.message, true)));

  try {
    const health = await api("/api/health");
    setHealth(health.status);

    await loadList();
    const characters = await api("/api/characters");
    if (characters.length) await selectCharacter(characters[0].id);
  } catch (error) {
    setHealth("offline", true);
    setStatus(error.message, true);
  }
}

init();
