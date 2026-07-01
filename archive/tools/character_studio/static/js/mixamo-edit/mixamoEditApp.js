/** Mixamo Edit — align Mixamo rig inside ghost skin, then join via Blender. */

import { mountAnimViewer } from "../assembly-studio/animViewer.js";
import {
  BUILD_URL,
  MANIFEST_URL,
  STATUS_URL,
  TEST_WALK_FBX,
  fbxUrl,
  formatApiDetail,
  pollJob,
  renderClips,
  renderFbxSelect,
  renderProjects,
  uploadSkin,
} from "./mixamoEditApi.js";
import { mountMixamoAlignViewer } from "./mixamoAlignViewer.js";
import { previewJoinedWithWalking } from "./mixamoResultLoader.js";
import {
  readBoneProportions,
  resetBoneProportionSliders,
  syncBoneProportions,
} from "./mixamoEditSliders.js";
import { bindMixamoEditControls } from "./mixamoEditControls.js";

let align = null;
let result = null;
let manifest = { projects: [] };
let selectedId = null;
let skinObjectUrl = null;

function setStatus(msg) {
  document.getElementById("editStatus").textContent = msg;
}

function unitId() {
  return document.getElementById("unitId")?.value?.trim() || "ybot_skin_01";
}

function selectedProject() {
  return manifest.projects.find((p) => p.id === selectedId) ?? null;
}

function vp() {
  return document.getElementById("viewport");
}

function resetViewport() {
  vp().replaceChildren();
}

function ensureAlign() {
  if (align) return align;
  result?.dispose();
  result = null;
  resetViewport();
  align = mountMixamoAlignViewer(vp());
  align.setOnTransformChange(syncRigSliders);
  return align;
}

function ensureResult() {
  if (result) return result;
  align?.dispose();
  align = null;
  resetViewport();
  result = mountAnimViewer(vp());
  return result;
}

function syncRigSliders() {
  if (!align?.hasPair()) return;
  const { rigPosition, skeletonSize, boneProportions } = align.getAlignment();
  document.getElementById("rigX").value = rigPosition[0].toFixed(3);
  document.getElementById("rigY").value = rigPosition[1].toFixed(3);
  document.getElementById("rigZ").value = rigPosition[2].toFixed(3);
  const pct = Math.round(skeletonSize * 100);
  document.getElementById("skeletonSize").value = String(pct);
  document.getElementById("skeletonSizeOut").textContent = `${skeletonSize.toFixed(2)}×`;
  syncBoneProportions(boneProportions);
}

function applyRigSliders() {
  if (!align?.hasPair()) return;
  align.setRigPosition(
    Number(document.getElementById("rigX").value),
    Number(document.getElementById("rigY").value),
    Number(document.getElementById("rigZ").value),
  );
}

async function loadResult(project) {
  selectedId = project.id;
  renderProjects(manifest.projects, selectedId, loadResult);
  setStatus(`Loading ${project.label}…`);
  try {
    const { names, pick } = await previewJoinedWithWalking(ensureResult(), project, fbxUrl(TEST_WALK_FBX));
    renderClips(names);
    setStatus(pick ? `${project.label || project.id} · ${pick}` : `${project.label || project.id} · T-pose`);
  } catch (err) {
    console.error(err);
    setStatus(err.message || "Preview load failed");
  }
}

async function loadAlignPreview() {
  const file = document.getElementById("customMeshFile")?.files?.[0];
  const fbx = document.getElementById("mixamoFbx")?.value;
  if (!file || !fbx) return setStatus("Select Mixamo FBX + skin GLB");
  if (skinObjectUrl) URL.revokeObjectURL(skinObjectUrl);
  skinObjectUrl = URL.createObjectURL(file);
  setStatus("Loading align preview…");
  try {
    await ensureAlign().loadAlignPair({ mixamoUrl: fbxUrl(fbx), skinUrl: skinObjectUrl });
    align.setSkinOpacity(Number(document.getElementById("skinOpacity")?.value ?? 38) / 100);
    align.setSkeletonSize(1);
    resetBoneProportionSliders();
    align.setBoneProportions(readBoneProportions());
    syncRigSliders();
    setStatus("Align rig inside ghost skin · then Join");
  } catch (err) {
    console.error(err);
    setStatus(err.message || "Align load failed");
  }
}

async function onJoin(ev) {
  ev.preventDefault();
  if (!ensureAlign().hasPair()) return setStatus("Load align preview first");
  const id = unitId();
  try {
    const mesh = await uploadSkin(id, document.getElementById("customMeshFile").files[0]);
    const alignData = ensureAlign().getAlignment();
    const res = await fetch(BUILD_URL, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        unitId: id,
        label: document.getElementById("label")?.value?.trim() || id,
        mixamoFbx: document.getElementById("mixamoFbx")?.value,
        customMesh: mesh,
        heightScale: Number(document.getElementById("heightScale")?.value ?? 100) / 100,
        rigPosition: alignData.rigPosition,
        rigScale: alignData.rigScale,
        manualAlign: true,
        boneProportions: alignData.boneProportions,
      }),
    });
    if (!res.ok) {
      const body = await res.json().catch(() => ({}));
      throw new Error(formatApiDetail(body.detail) || "Build failed");
    }
    setStatus("Starting Blender…");
    const report = await pollJob((await res.json()).id, setStatus);
    manifest = await fetch(MANIFEST_URL).then((r) => r.json());
    renderProjects(manifest.projects, selectedId, loadResult);
    const built = manifest.projects.find((p) => p.id === id);
    if (built) await loadResult(built);
    else setStatus("Joined · select project to preview Walking");
  } catch (err) {
    console.error(err);
    setStatus(err.message || "Join failed");
  }
}

function bindControls() {
  bindMixamoEditControls({
    loadAlignPreview,
    onJoin,
    alignRef: () => align,
    syncRigSliders,
    applyRigSliders,
    resultRef: () => result,
    selectedProject,
    setStatus,
    onProjectRemoved(project, next) {
      if (project && selectedId === project.id) {
        selectedId = null;
        result?.dispose();
        result = null;
        renderClips([]);
        ensureAlign();
      }
      manifest = next;
      renderProjects(manifest.projects, selectedId, loadResult);
    },
  });
}

async function init() {
  bindControls();
  ensureAlign();
  const status = await fetch(STATUS_URL).then((r) => r.json());
  renderFbxSelect(status.rawFbx || []);
  manifest = await fetch(MANIFEST_URL).then((r) => r.json()).catch(() => ({ projects: [] }));
  renderProjects(manifest.projects, selectedId, loadResult);
  setStatus(status.rawFbx?.includes("Y Bot.fbx") ? "Upload skin GLB · Load align" : "Add Y Bot.fbx to raw/");
}

document.addEventListener("DOMContentLoaded", () => void init());
