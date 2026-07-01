import { mountRigBoneEditor } from "./rigBoneEditor.js";
import { SHOWCASE_ORDER } from "../showcase/showcaseOrder.js";
import { renderMeshBadge } from "../animation-studio/meshBadge.js";
import { renderRegionLegend } from "./rigBoneRegions.js";
import { probeAsset } from "../viewer/gltfLoader.js";
import {
  api,
  fetchBones,
  fetchRigConfig,
  fetchSession,
  pollJob,
  saveBones,
  startJob,
} from "./rigJobClient.js";

const $ = (id) => document.getElementById(id);

const STEPS = [
  { id: "select", label: "Select unit" },
  { id: "prep", label: "Load skeleton" },
  { id: "edit", label: "Place joints" },
  { id: "bind", label: "Bind & export" },
  { id: "review", label: "Review" },
];

let selectedId = null;
let editor = null;
let rigConfig = null;
let sessionInfo = null;
let activeStep = 0;
let rigApiOk = false;

function rigApiErrorMessage() {
  return "Rig API missing — restart: ./scripts/start-studio.sh (then hard-refresh Ctrl+Shift+R)";
}

function setStatus(message, isError = false) {
  const el = $("jobStatus");
  el.textContent = message;
  el.className = isError ? "status status--error" : "status";
}

function setHealth(ok, detail) {
  const health = $("health");
  health.textContent = detail;
  health.className = ok ? "badge badge--ok" : "badge badge--error";
}

function setViewportLoading(show, message = "Loading 3D…") {
  $("viewer3dLoading").hidden = !show;
  $("viewer3dLoading").textContent = message;
  $("viewer3dEmpty").hidden = show || Boolean(selectedId);
  $("viewer3d").hidden = show;
  $("viewer3dControls").hidden = show || !selectedId;
}

function sortCharacters(characters) {
  const order = new Map(SHOWCASE_ORDER.map((id, index) => [id, index]));
  return [...characters].sort((a, b) => {
    const left = order.has(a.id) ? order.get(a.id) : 9999;
    const right = order.has(b.id) ? order.get(b.id) : 9999;
    return left - right || a.id.localeCompare(b.id);
  });
}

function renderSteps() {
  $("stepList").innerHTML = STEPS.map((step, index) => {
    let cls = index === activeStep ? "is-active" : index < activeStep ? "is-done" : "";
    return `<li class="${cls}">${step.label}</li>`;
  }).join("");
}

function showStepPanels() {
  $("stepPrep").hidden = activeStep !== 1;
  $("stepEdit").hidden = activeStep !== 2;
  $("stepBind").hidden = activeStep !== 3;
  $("stepReview").hidden = activeStep !== 4;
}

function setActiveStep(index) {
  activeStep = Math.max(0, Math.min(index, STEPS.length - 1));
  renderSteps();
  showStepPanels();
}

function renderUnitMeta() {
  if (!rigConfig) {
    $("unitMeta").innerHTML = "";
    return;
  }
  const session = sessionInfo?.session;
  const previewReady = session?.preview?.exists;
  $("unitMeta").innerHTML = [
    `<div><strong>${rigConfig.unitId}</strong> · ${rigConfig.bodyClass}</div>`,
    previewReady
      ? `<div>Skeleton loaded — drag numbered joints on the mesh</div>`
      : `<div>Waiting for skeleton prep…</div>`,
    `<div>${editor?.getBoneNames()?.length ?? 0} joints · ${editor?.getLockedCount?.() ?? 0} locked</div>`,
  ].join("");
}

function ensureEditor() {
  if (editor) return editor;
  editor = mountRigBoneEditor($("viewer3d"));
  editor.onLockChange((boneName, locked, lockedTotal, hint) => {
    const el = $("lockStatus");
    if (!el) return;
    const total = editor.getBoneNames().length;
    if (hint === "tap_locked") {
      el.textContent = `${boneName} is locked — double-click to unlock`;
      el.className = "rig-lock-status rig-lock-status--ok";
      renderUnitMeta();
      return;
    }
    el.textContent = locked
      ? `Locked: ${boneName} (${lockedTotal}/${total}) — double-click to unlock`
      : `Unlocked: ${boneName} — drag to move, double-click to lock`;
    el.className = locked ? "rig-lock-status rig-lock-status--ok" : "rig-lock-status";
    renderUnitMeta();
  });
  $("xrayMesh")?.addEventListener("change", (e) => editor.setXrayMesh(e.target.checked));
  $("wireframeMesh")?.addEventListener("change", (e) => editor.setWireframe(e.target.checked));
  $("showTails")?.addEventListener("change", (e) => editor.setShowTails(e.target.checked));
  return editor;
}

async function loadShowcaseMesh(unitId) {
  setViewportLoading(true, "Loading showcase mesh…");
  const meshBase = `/api/characters/${unitId}/mesh?tier=showcase`;
  const probe = await probeAsset(meshBase);
  if (!probe.ok) throw new Error("Showcase mesh not found");
  const url = probe.mtime ? `${meshBase}&v=${probe.mtime}` : meshBase;
  await ensureEditor().loadPreview(url, { edit: false });
  setViewportLoading(false);
  $("viewer3d").hidden = false;
  $("viewer3dControls").hidden = false;
  $("viewer3dEmpty").hidden = true;
}

async function loadRigPreview(unitId) {
  setViewportLoading(true, "Loading skeleton…");
  const session = await fetchSession(unitId);
  if (!session.session?.preview?.exists) {
    throw new Error("Preview not ready");
  }
  const v = session.session.preview.modifiedAt ?? Date.now();
  let bonesSource = null;
  try {
    bonesSource = await fetchBones(unitId);
  } catch {
    bonesSource = null;
  }
  await ensureEditor().loadPreview(`/api/rig/sessions/${unitId}/preview?v=${v}`, {
    edit: true,
    bonesSource,
  });
  editor.setXrayMesh($("xrayMesh")?.checked ?? true);
  editor.setWireframe($("wireframeMesh")?.checked ?? false);
  editor.setShowTails($("showTails")?.checked ?? true);
  setViewportLoading(false);
  $("viewer3d").hidden = false;
  $("viewer3dControls").hidden = false;
  $("viewer3dEmpty").hidden = true;
  $("boneHint").hidden = false;
  renderRegionLegend($("boneRegionLegend"));
  renderUnitMeta();
}

async function refreshMeshInfo(unitId) {
  try {
    renderMeshBadge($("meshBadge"), await api(`/api/characters/${unitId}/mesh-info?tier=showcase`));
  } catch {
    renderMeshBadge($("meshBadge"), null);
  }
}

async function refreshSession() {
  if (!selectedId) return;
  sessionInfo = await fetchSession(selectedId);
  renderUnitMeta();
  if (sessionInfo.session?.preview?.exists && activeStep < 2) {
    setActiveStep(2);
  }
}

async function runPrep() {
  if (!selectedId) return;
  if (!rigApiOk) {
    setStatus(rigApiErrorMessage(), true);
    return;
  }
  $("btnPrep").disabled = true;
  setStatus("Blender: building skeleton (headless)…");
  try {
    const job = await startJob(selectedId, "prep");
    await pollJob(job.id, { onTick: (tick) => setStatus(`prep: ${tick.status}…`) });
    await refreshSession();
    await loadRigPreview(selectedId);
    setActiveStep(2);
    setStatus("Drag the blue joint handles onto shoulders, elbows, knees.");
  } catch (error) {
    setStatus(error.message, true);
  } finally {
    $("btnPrep").disabled = false;
  }
}

async function saveBoneEdits() {
  if (!selectedId || !editor?.isEditable()) return;
  setStatus("Saving bone positions…");
  try {
    await saveBones(selectedId, editor.exportBonesPayload());
    setStatus("Bones saved.");
    setActiveStep(3);
  } catch (error) {
    setStatus(error.message, true);
  }
}

async function runFinalize() {
  if (!selectedId) return;
  if (editor?.isEditable()) {
    await saveBones(selectedId, editor.exportBonesPayload());
  }
  $("btnFinalize").disabled = true;
  setStatus("Blender: binding weights + exporting game GLB…");
  try {
    const job = await startJob(selectedId, "finalize");
    const finished = await pollJob(job.id, {
      onTick: (tick) => setStatus(`finalize: ${tick.status}…`),
    });
    renderWeightReport(finished.report);
    setActiveStep(4);
    setStatus("Game mesh exported.");
    await refreshMeshInfo(selectedId);
  } catch (error) {
    setStatus(error.message, true);
  } finally {
    $("btnFinalize").disabled = false;
  }
}

async function runBakeWalk() {
  if (!selectedId) return;
  if (editor?.isEditable()) {
    await saveBones(selectedId, editor.exportBonesPayload());
  }
  $("btnBakeWalk").disabled = true;
  setStatus("Blender: baking Perry Walk on your skeleton…");
  try {
    const job = await startJob(selectedId, "bake_walk");
    const finished = await pollJob(job.id, {
      onTick: (tick) => setStatus(`bake_walk: ${tick.status}…`),
    });
    const report = finished.report ?? {};
    setStatus(`Walk baked (${report.walkProfile ?? "gait"}) — open Motion Arena to preview.`);
    await refreshMeshInfo(selectedId);
  } catch (error) {
    setStatus(error.message, true);
  } finally {
    $("btnBakeWalk").disabled = false;
  }
}

async function runAuto() {
  if (!selectedId) return;
  setStatus("Running experimental auto-rig…");
  try {
    const job = await startJob(selectedId, "auto");
    const finished = await pollJob(job.id, { onTick: (tick) => setStatus(`auto: ${tick.status}…`) });
    renderWeightReport(finished.report);
    setActiveStep(4);
    setStatus("Auto-rig finished.");
  } catch (error) {
    setStatus(error.message, true);
  }
}

function renderWeightReport(report) {
  const weight = report?.weight;
  if (!weight) {
    $("weightReport").innerHTML = "<p>No weight metrics.</p>";
    return;
  }
  const cls = weight.passesGate ? "rig-weight--pass" : "rig-weight--fail";
  $("weightReport").innerHTML = `
    <p class="${cls}"><strong>${weight.passesGate ? "Weight gate PASS" : "Weight gate FAIL"}</strong></p>
    <ul>
      <li>Dominant: ${weight.dominantBone} (${(weight.dominantShare * 100).toFixed(1)}%)</li>
      <li>Unweighted verts: ${weight.unweightedVertices}</li>
    </ul>
  `;
}

async function loadUnit(unitId) {
  selectedId = unitId;
  $("editId").textContent = unitId;
  $("rigPanel").hidden = false;
  $("boneHint").hidden = true;
  setActiveStep(1);
  setStatus("");
  window.location.hash = unitId;

  $("linkAnimation").href = `/animation-studio.html#${unitId}`;
  $("linkArena").href = `/motion-arena.html#${unitId}`;
  document.querySelectorAll(".unit-item").forEach((btn) => {
    btn.classList.toggle("is-active", btn.dataset.id === unitId);
  });

  await loadShowcaseMesh(unitId);
  await refreshMeshInfo(unitId);

  if (!rigApiOk) {
    setStatus(rigApiErrorMessage(), true);
    return;
  }

  try {
    rigConfig = await fetchRigConfig(unitId);
  } catch (error) {
    setStatus(error.message, true);
    rigConfig = { unitId, bodyClass: "HumanoidLight" };
  }

  await refreshSession();
  renderUnitMeta();

  try {
    if (sessionInfo?.session?.preview?.exists) {
      await loadRigPreview(unitId);
      setActiveStep(2);
    } else {
      setStatus("Click Load skeleton to attach TBG bones (Blender runs in background).");
    }
  } catch (error) {
    setStatus(error.message, true);
  }
}

async function init() {
  let healthOk = false;
  try {
    const health = await api("/api/health");
    healthOk = true;
    try {
      const rig = await api("/api/rig/status");
      rigApiOk = true;
      const blender = rig.blenderAvailable ? "Blender OK" : "No Blender";
      setHealth(true, `${health.studioBuild} · ${blender}`);
    } catch {
      rigApiOk = false;
      setHealth(false, `${health.studioBuild} · restart server`);
      setStatus(rigApiErrorMessage(), true);
    }
  } catch {
    setHealth(false, "offline — run ./scripts/start-studio.sh");
  }

  renderSteps();
  showStepPanels();

  if (!healthOk) {
    setStatus("Start the studio server: ./scripts/start-studio.sh");
    return;
  }

  let characters = [];
  try {
    characters = sortCharacters((await api("/api/characters")).filter((c) => c.mesh));
  } catch (error) {
    setStatus(error.message, true);
    return;
  }

  $("unitCount").textContent = `${characters.length} mesh3d`;
  $("charList").innerHTML = characters
    .map((character, index) => {
      const active = character.id === selectedId ? " is-active" : "";
      return `
        <button type="button" class="unit-item unit-item--chip${active}" data-id="${character.id}" role="listitem">
          <span class="unit-item__index">${index}</span>
          <span class="unit-item__name">${character.id.replace("demo_", "").replace("_01", "")}</span>
        </button>`;
    })
    .join("");

  $("charList").addEventListener("click", (event) => {
    const btn = event.target.closest("[data-id]");
    if (btn) loadUnit(btn.dataset.id);
  });

  $("btnPrep").addEventListener("click", () => runPrep());
  $("btnSaveBones").addEventListener("click", () => saveBoneEdits());
  $("btnFinalize").addEventListener("click", () => runFinalize());
  $("btnBakeWalk")?.addEventListener("click", () => runBakeWalk());
  $("btnAuto").addEventListener("click", () => runAuto());

  const hash = window.location.hash.replace("#", "");
  if (hash) {
    loadUnit(hash);
  }
}

init();
