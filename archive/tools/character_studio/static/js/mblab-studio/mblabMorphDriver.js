/** Real-time morph target blending for MB-Lab Studio preview GLB. */

const NEUTRAL = 0.5;

export function morphInfluences(value, neutral = NEUTRAL) {
  const v = Number(value);
  return {
    max: Math.max(0, (v - neutral) * 2),
    min: Math.max(0, (neutral - v) * 2),
  };
}

export function collectMorphMeshes(root) {
  const meshes = [];
  if (!root) return meshes;
  root.traverse((obj) => {
    if (obj.isMesh && obj.morphTargetInfluences?.length) meshes.push(obj);
  });
  return meshes;
}

export function applyStudioMorphs(meshes, morphs, config) {
  if (!meshes.length || !config?.sliders) return false;
  let applied = false;
  for (const mesh of meshes) {
    const dict = mesh.morphTargetDictionary || {};
    for (const slider of config.sliders) {
      if (slider.browserOnly) continue;
      const { max, min } = morphInfluences(morphs[slider.id] ?? NEUTRAL);
      const maxKey = `tbg_${slider.id}_max`;
      const minKey = `tbg_${slider.id}_min`;
      if (maxKey in dict) {
        mesh.morphTargetInfluences[dict[maxKey]] = max;
        applied = true;
      }
      if (minKey in dict) {
        mesh.morphTargetInfluences[dict[minKey]] = min;
        applied = true;
      }
    }
  }
  return applied;
}

export function applyStatureScale(root, stature) {
  if (!root) return;
  const s = 0.82 + Number(stature ?? NEUTRAL) * 0.36;
  root.scale.setScalar(s);
}
