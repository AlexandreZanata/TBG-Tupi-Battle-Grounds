/** MB-Lab Studio — morphs, look, accessories, optional Blender HQ export. */

import { fetchLibrary } from "../assembly-studio/assemblyApi.js";
import { mountAssemblyViewer } from "../assembly-studio/assemblyViewer.js";
import { downloadMorphGlb, exportAssembledGlb } from "./mblabExport.js";
import { loadStudioMorphConfig, readHeadMorphs, renderHeadSliders } from "./mblabHeadMorphs.js";
import { LIVE_UNIT_ID, runLiveMorph, scheduleLiveMorph } from "./mblabLive.js";
import { applyMaterialPreset, readLook, renderLookPanel } from "./mblabLook.js";
import { applyLook } from "./mblabMaterials.js";
import { applyStudioMorphs, collectMorphMeshes } from "./mblabMorphDriver.js";
import {
  createPartsState,
  renderPartsPanel,
  resolveSelectedParts,
  selectedPartIds,
} from "./mblabParts.js";
import { ensureMblabPipeline } from "./mblabSetupClient.js";
import { applyQuickMorph } from "./mblabQuickPreview.js";

const STATUS_URL = "/api/assembly/mblab/status";
const SETUP_URL = "/api/assembly/mblab/setup";

let viewer = null;
let library = null;
let studioConfig = null;
let lastStatus = null;
let lastMorphUrl = null;
let hasMorphTargets = false;
let blenderSync = false;
let materialId = "default";
const parts = createPartsState();

function setStatus(msg) {
  document.getElementById("mblabStatus").textContent = msg;
}

function readBodyMorphs() {
  const pick = (id) => Number(document.getElementById(id)?.value ?? 50) / 100;
  return { age: pick("age"), body_mass: pick("bodyMass"), muscles: pick("muscles"), stature: pick("stature") };
}

function readMorphs() {
  return { ...readBodyMorphs(), ...readHeadMorphs(studioConfig) };
}

function selectedBaseId() {
  return document.getElementById("baseId")?.value || "male";
}

function exportUnitId() {
  return document.getElementById("unitId")?.value?.trim() || "mblab_preview_01";
}

function bindSlider(rangeId, outId, onChange, { fireInitial = false } = {}) {
  const range = document.getElementById(rangeId);
  const out = document.getElementById(outId);
  if (!range || !out) return;
  const sync = (fire) => {
    out.textContent = (Number(range.value) / 100).toFixed(2);
    if (fire) onChange();
  };
  range.addEventListener("input", () => sync(true));
  sync(fireInitial);
}

async function resolvePreviewUrl(baseId) {
  const preview = `/api/assembly/bases/${baseId}/preview`;
  const res = await fetch(preview, { method: "HEAD" });
  return res.ok ? preview : `/api/assembly/bases/${baseId}/mesh`;
}

function syncLookAndMorphs() {
  const morphs = readMorphs();
  hasMorphTargets = applyStudioMorphs(collectMorphMeshes(viewer.getRoot?.()), morphs, studioConfig);
  applyQuickMorph(viewer.getRoot?.(), morphs);
  applyLook(viewer.getRoot?.(), readLook());
}

async function loadPreview(baseUrl) {
  await viewer.loadPreview({ baseUrl, parts: resolveSelectedParts(library, parts) });
  syncLookAndMorphs();
  hasMorphTargets = collectMorphMeshes(viewer.getRoot?.()).length > 0;
}

async function refreshBasePreview() {
  const baseId = selectedBaseId();
  const url = lastMorphUrl || `${await resolvePreviewUrl(baseId)}?t=${Date.now()}`;
  setStatus(`Loading ${baseId}…`);
  try {
    await loadPreview(url);
    setStatus(hasMorphTargets ? "Ready" : "Ready");
  } catch (err) {
    console.error("MB-Lab preview failed", err);
    setStatus(err?.message || "Preview failed — check console");
  }
}

async function bootstrapPipeline() {
  try {
    lastStatus = await ensureMblabPipeline(STATUS_URL, SETUP_URL, setStatus);
    if (!hasMorphTargets) await refreshBasePreview();
  } catch (err) {
    console.error("MB-Lab setup failed", err);
    setStatus(err?.message || "Setup failed");
  }
}

function onLookChanged() {
  applyLook(viewer.getRoot?.(), readLook());
  setStatus("Live");
}

function onMorphChanged() {
  syncLookAndMorphs();
  setStatus(hasMorphTargets ? "Live" : "Adjusting…");
  if (!blenderSync) return;
  scheduleLiveMorph({
    baseId: selectedBaseId(),
    morphs: readMorphs(),
    onStatus: setStatus,
    onReady: async (url) => {
      lastMorphUrl = url;
      await loadPreview(url);
    },
  });
}

function onPartPicked(category, partId) {
  parts[category] = partId;
  renderPartsPanel(library, selectedBaseId(), parts, onPartPicked);
  void viewer.updateParts(resolveSelectedParts(library, parts)).then(() => {
    applyLook(viewer.getRoot?.(), readLook());
    setStatus("Live");
  });
}

async function onSubmit(ev) {
  ev.preventDefault();
  if (lastStatus && !lastStatus.ready) lastStatus = await ensureMblabPipeline(STATUS_URL, SETUP_URL, setStatus);
  await runLiveMorph({
    baseId: selectedBaseId(),
    morphs: readMorphs(),
    onStatus: setStatus,
    onReady: async (u) => {
      lastMorphUrl = u;
      await loadPreview(u);
    },
  });
  setStatus("Done");
}

async function init() {
  try {
    viewer = mountAssemblyViewer(document.getElementById("viewport"));
    const onSlider = () => onMorphChanged();
    bindSlider("age", "ageOut", onSlider);
    bindSlider("bodyMass", "bodyMassOut", onSlider);
    bindSlider("muscles", "musclesOut", onSlider);
    bindSlider("stature", "statureOut", onSlider);
    studioConfig = await loadStudioMorphConfig();
    renderHeadSliders(studioConfig, onSlider);
    library = await fetchLibrary();
    function pickMaterial(mat) {
      materialId = mat.id;
      applyMaterialPreset(mat);
      renderLookPanel(library, { materialId, onChange: onLookChanged, onMaterial: pickMaterial });
      onLookChanged();
    }
    renderLookPanel(library, { materialId, onChange: onLookChanged, onMaterial: pickMaterial });
    renderPartsPanel(library, selectedBaseId(), parts, onPartPicked);
    document.getElementById("blenderSync")?.addEventListener("change", (e) => {
      blenderSync = e.target.checked;
      if (blenderSync) onMorphChanged();
    });
    document.getElementById("mblabForm")?.addEventListener("submit", (e) => void onSubmit(e));
    document.getElementById("baseId")?.addEventListener("change", () => {
      lastMorphUrl = null;
      renderPartsPanel(library, selectedBaseId(), parts, onPartPicked);
      void refreshBasePreview();
    });
    document.getElementById("btnDownloadBody")?.addEventListener("click", () => {
      void downloadMorphGlb(LIVE_UNIT_ID, setStatus).catch((e) => setStatus(e.message));
    });
    document.getElementById("btnExportAssembled")?.addEventListener("click", () => {
      void exportAssembledGlb({
        unitId: exportUnitId(),
        baseId: selectedBaseId(),
        partIds: selectedPartIds(parts),
        materialId,
        onStatus: setStatus,
      }).catch((e) => setStatus(e.message));
    });
    const statusRes = await fetch(STATUS_URL);
    lastStatus = await statusRes.json();
    await refreshBasePreview();
    void bootstrapPipeline();
  } catch (err) {
    console.error("MB-Lab Studio init failed", err);
    setStatus(err?.message || "Init failed");
  }
}

document.addEventListener("DOMContentLoaded", () => void init());
