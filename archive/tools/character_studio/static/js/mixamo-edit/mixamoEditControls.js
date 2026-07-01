/** Mixamo Edit UI control bindings. */

import { bindBonePropSliders } from "./mixamoEditSliders.js";
import { bindProjectActions } from "./mixamoEditProjects.js";

export function bindMixamoEditControls(deps) {
  const {
    loadAlignPreview,
    onJoin,
    alignRef,
    syncRigSliders,
    applyRigSliders,
    resultRef,
    selectedProject,
    setStatus,
    onProjectRemoved,
  } = deps;

  document.getElementById("btnLoadAlign")?.addEventListener("click", () => void loadAlignPreview());
  document.getElementById("customMeshFile")?.addEventListener("change", () => void loadAlignPreview());
  document.getElementById("editForm")?.addEventListener("submit", (e) => void onJoin(e));
  document.getElementById("btnModeMove")?.addEventListener("click", () => alignRef()?.setTransformMode("translate"));
  document.getElementById("btnResetRig")?.addEventListener("click", () => {
    alignRef()?.resetRigTransform();
    syncRigSliders();
  });
  bindBonePropSliders(alignRef);
  document.getElementById("skeletonSize")?.addEventListener("input", (e) => {
    const mult = Number(e.target.value) / 100;
    document.getElementById("skeletonSizeOut").textContent = `${mult.toFixed(2)}×`;
    alignRef()?.setSkeletonSize(mult);
    syncRigSliders();
  });
  document.getElementById("skinOpacity")?.addEventListener("input", (e) => {
    document.getElementById("skinOpacityOut").textContent = (Number(e.target.value) / 100).toFixed(2);
    alignRef()?.setSkinOpacity(Number(e.target.value) / 100);
  });
  for (const id of ["rigX", "rigY", "rigZ"]) {
    document.getElementById(id)?.addEventListener("change", applyRigSliders);
  }
  document.getElementById("heightScale")?.addEventListener("input", (e) => {
    document.getElementById("heightScaleOut").textContent = (Number(e.target.value) / 100).toFixed(2);
  });
  document.getElementById("btnPlayClip")?.addEventListener("click", () => {
    const n = document.getElementById("clipSelect")?.value;
    if (n) resultRef()?.playClip(n, true);
  });
  document.getElementById("btnStopClip")?.addEventListener("click", () => resultRef()?.stopClip());
  bindProjectActions({ getSelected: selectedProject, setStatus, onRemoved: onProjectRemoved });
}
