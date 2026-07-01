/** Manual Assembly Studio — skeleton, materials, transform per part. */

import { fetchLibrary, pollJob, queueExport } from "./assemblyApi.js";
import { mountManualScene } from "./manualScene.js";

const state = {
  library: null,
  baseId: null,
  materialId: "default",
  selectedBone: "TBG_Head",
  scene: null,
};

function $(id) {
  return document.getElementById(id);
}

function setStatus(msg) {
  const el = $("manualStatus");
  if (el) el.textContent = msg;
}

function allParts() {
  const out = [];
  for (const [cat, items] of Object.entries(state.library?.parts || {})) {
    for (const item of items) out.push({ ...item, category: cat });
  }
  return out;
}

function renderBones() {
  const list = $("boneList");
  if (!list || !state.scene) return;
  list.innerHTML = "";
  for (const name of state.scene.listBones()) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "manual-list__item";
    if (name === state.selectedBone) btn.classList.add("manual-list__item--active");
    btn.textContent = name;
    btn.addEventListener("click", () => {
      state.selectedBone = name;
      state.scene.selectBone(name);
      renderBones();
      setStatus(`Osso: ${name}`);
    });
    list.appendChild(btn);
  }
}

function renderParts() {
  const list = $("partList");
  if (!list) return;
  list.innerHTML = "";
  for (const part of allParts()) {
    if (state.baseId && !part.baseCompat.includes(state.baseId)) continue;
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "manual-list__item";
    btn.textContent = `${part.id} → ${part.socket}`;
    btn.addEventListener("click", () => void attachPart(part));
    list.appendChild(btn);
  }
}

function renderMaterials() {
  const list = $("materialList");
  if (!list) return;
  list.innerHTML = "";
  for (const mat of state.library?.materials || []) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "manual-list__item";
    if (mat.id === state.materialId) btn.classList.add("manual-list__item--active");
    btn.textContent = mat.label || mat.id;
    btn.addEventListener("click", () => {
      state.materialId = mat.id;
      state.scene?.applyMaterialProfile(mat.profile);
      renderMaterials();
      setStatus(`Material: ${mat.id}`);
    });
    list.appendChild(btn);
  }
}

function renderBases() {
  const list = $("baseList");
  if (!list) return;
  list.innerHTML = "";
  for (const base of state.library?.bases || []) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "manual-list__item";
    if (base.id === state.baseId) btn.classList.add("manual-list__item--active");
    btn.textContent = base.label;
    btn.addEventListener("click", () => void loadBase(base));
    list.appendChild(btn);
  }
}

async function loadBase(base) {
  state.baseId = base.id;
  renderBases();
  setStatus(`Carregando ${base.label}…`);
  await state.scene.loadBase(base.meshUrl);
  renderBones();
  renderParts();
  const mat = state.library.materials.find((m) => m.id === state.materialId);
  if (mat) state.scene.applyMaterialProfile(mat.profile);
  setStatus(`Base: ${base.label} — escolha osso + peça`);
}

async function attachPart(part) {
  if (!state.baseId) {
    setStatus("Escolha uma base primeiro");
    return;
  }
  const socket = state.selectedBone || part.socket;
  await state.scene.attachPart({ partId: part.id, meshUrl: part.meshUrl, socket });
  setStatus(`Anexado: ${part.id} em ${socket} — use G/R/S e arraste`);
}

async function runExport() {
  const unitId = $("unitId")?.value?.trim();
  if (!unitId || !state.baseId) {
    setStatus("Preencha unit id e base");
    return;
  }
  const attachments = state.scene.getExportPayload();
  const parts = attachments.map((a) => a.partId);
  setStatus("Exportando…");
  const job = await queueExport({
    unitId,
    baseId: state.baseId,
    parts,
    materialId: state.materialId,
    attachments,
  });
  for (let i = 0; i < 60; i += 1) {
    const status = await pollJob(job.id);
    if (status.status === "completed") {
      const url = status.report?.exportUrl;
      $("exportLink").innerHTML = url ? `<a href="${url}" target="_blank">Baixar GLB</a>` : "OK";
      setStatus("Export pronto");
      return;
    }
    if (status.status === "failed") {
      setStatus(status.error || "Falhou");
      return;
    }
    await new Promise((r) => setTimeout(r, 400));
  }
}

async function init() {
  state.scene = mountManualScene($("viewport"));
  state.library = await fetchLibrary();
  renderBases();
  renderMaterials();
  $("btnMove")?.addEventListener("click", () => state.scene.setMode("translate"));
  $("btnRotate")?.addEventListener("click", () => state.scene.setMode("rotate"));
  $("btnScale")?.addEventListener("click", () => state.scene.setMode("scale"));
  $("btnSkeleton")?.addEventListener("change", (e) => state.scene.setSkeletonVisible(e.target.checked));
  $("btnExport")?.addEventListener("click", () => void runExport());
  const male = state.library.bases.find((b) => b.id === "male");
  if (male) await loadBase(male);
}

document.addEventListener("DOMContentLoaded", () => {
  void init().catch((err) => setStatus(err.message));
});
