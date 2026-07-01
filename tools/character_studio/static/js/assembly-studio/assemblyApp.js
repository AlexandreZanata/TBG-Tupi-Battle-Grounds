/** Assembly Studio — modular character builder (Phase 05 §4). */

import { fetchLibrary, pollJob, queueExport, recipeFromState } from "./assemblyApi.js";
import { createPanelRenderer } from "./assemblyPanels.js";
import { mountAssemblyViewer } from "./assemblyViewer.js";

const STEPS = [
  { id: "body", label: "Body", categories: [] },
  { id: "head", label: "Head", categories: ["heads"] },
  { id: "parts", label: "Parts", categories: ["hair", "armor", "weapons", "hats"] },
  { id: "materials", label: "Materials", categories: [] },
  { id: "review", label: "Review", categories: [] },
  { id: "export", label: "Export", categories: [] },
];

const state = {
  step: 0,
  baseId: null,
  parts: {},
  materialId: "default",
  unitId: "",
  library: null,
  viewer: null,
};

function selectedPartIds() {
  return Object.values(state.parts).filter(Boolean);
}

function resolvePart(partId) {
  if (!state.library?.parts) return null;
  for (const items of Object.values(state.library.parts)) {
    const hit = items.find((p) => p.id === partId);
    if (hit) return hit;
  }
  return null;
}

function setStatus(msg) {
  const el = document.getElementById("assemblyStatus");
  if (el) el.textContent = msg;
}

function goStep(index) {
  state.step = Math.max(0, Math.min(index, STEPS.length - 1));
  panels.renderSteps();
  panels.renderPanel();
}

const panels = createPanelRenderer({
  STEPS,
  state,
  setStatus,
  goStep,
  refreshPreview,
  runExport,
});

async function refreshPreview() {
  if (!state.viewer || !state.baseId || !state.library) return;
  const base = state.library.bases.find((b) => b.id === state.baseId);
  if (!base?.meshUrl) return;
  const parts = selectedPartIds()
    .map(resolvePart)
    .filter(Boolean)
    .map((p) => ({ socket: p.socket, meshUrl: p.meshUrl }));
  try {
    await state.viewer.loadPreview({ baseUrl: base.meshUrl, parts });
  } catch (err) {
    setStatus(`Preview error: ${err.message}`);
  }
}

async function runExport() {
  const status = document.getElementById("exportStatus");
  if (!state.baseId) {
    setStatus("Select a base body first");
    return;
  }
  if (status) status.textContent = "Queuing…";
  try {
    const job = await queueExport(recipeFromState(state));
    if (status) status.textContent = `Job ${job.id} running…`;
    const finalJob = await waitForJob(job.id);
    if (finalJob.status === "completed") {
      const url = finalJob.report?.exportUrl;
      if (status) status.innerHTML = `Done — <a href="${url}">download GLB</a>`;
      setStatus("Export complete");
    } else if (status) {
      status.textContent = finalJob.error || "Export failed";
    }
  } catch (err) {
    if (status) status.textContent = err.message;
  }
}

async function waitForJob(jobId) {
  for (let i = 0; i < 60; i += 1) {
    const job = await pollJob(jobId);
    if (job.status === "completed" || job.status === "failed") return job;
    await new Promise((r) => setTimeout(r, 500));
  }
  throw new Error("export timed out");
}

async function checkHealth() {
  const badge = document.getElementById("health");
  try {
    const res = await fetch("/api/health");
    const data = await res.json();
    if (badge) {
      badge.textContent = data.status === "ok" ? "online" : "degraded";
      badge.classList.toggle("badge--ok", data.status === "ok");
    }
  } catch {
    if (badge) badge.textContent = "offline";
  }
}

export async function initAssemblyStudio() {
  const host = document.getElementById("viewport");
  if (host) state.viewer = mountAssemblyViewer(host);
  try {
    state.library = await fetchLibrary();
  } catch (err) {
    setStatus(`Library load failed: ${err.message}`);
  }
  panels.renderSteps();
  panels.renderPanel();
  checkHealth();
  document.getElementById("btnNext")?.addEventListener("click", () => goStep(state.step + 1));
  document.getElementById("btnPrev")?.addEventListener("click", () => goStep(state.step - 1));
}

document.addEventListener("DOMContentLoaded", () => {
  void initAssemblyStudio();
});
