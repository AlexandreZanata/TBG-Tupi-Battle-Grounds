/** Rest-pose bone proportion tweaks — spread hips/shoulders, flatten torso depth. */

import { buildBoneCache, captureRestPose, restoreAllBones } from "../assembly-studio/walkBoneCache.js";

const DEFAULTS = { hipWidth: 1, shoulderWidth: 1, torsoDepth: 1 };

const RULES = [
  { key: "hipWidth", bones: ["LeftUpLeg", "RightUpLeg", "LeftLeg", "RightLeg", "LeftFoot", "RightFoot"], width: true },
  { key: "shoulderWidth", bones: ["LeftShoulder", "RightShoulder", "LeftArm", "RightArm"], width: true },
  { key: "torsoDepth", bones: ["Hips", "Spine", "Spine1", "Spine2"], width: false },
];

function shortName(name) {
  return name.replace(/^mixamorig[:_]?/i, "");
}

function buildMixamoBoneCache(root) {
  const byName = buildBoneCache(root);
  root.traverse((obj) => {
    if (!obj.isSkinnedMesh?.skeleton) return;
    for (const bone of obj.skeleton.bones) {
      if (!byName.has(bone.name)) byName.set(bone.name, bone);
    }
  });
  return byName;
}

function findBone(byName, canonical) {
  for (const [name, bone] of byName) {
    if (shortName(name) === canonical) return bone;
  }
  return null;
}

function clampFactor(v) {
  return Math.max(0.5, Math.min(2, Number(v) || 1));
}

function pickWidthAxis(snap) {
  return Math.abs(snap.p.x) >= Math.abs(snap.p.z) ? "x" : "z";
}

function scaleAxis(bone, snap, factor, width) {
  bone.position.copy(snap.p);
  const axis = width ? pickWidthAxis(snap) : "z";
  if (axis === "x") bone.position.x *= factor;
  else bone.position.z *= factor;
}

/** Recompute bind pose after rest-pose bone edits — required for mesh to deform. */
function rebindSkinnedMeshes(root) {
  root.updateMatrixWorld(true);
  root.traverse((obj) => {
    if (!obj.isSkinnedMesh?.skeleton) return;
    const skel = obj.skeleton;
    for (const bone of skel.bones) bone.updateMatrixWorld(true);
    skel.calculateInverses();
    skel.update();
  });
}

export function createBoneProportions(root) {
  const byName = buildMixamoBoneCache(root);
  const rest = captureRestPose(byName);
  const state = { ...DEFAULTS };

  function apply(next = state) {
    restoreAllBones(byName, rest);
    for (const rule of RULES) {
      const factor = clampFactor(next[rule.key] ?? 1);
      state[rule.key] = factor;
      if (Math.abs(factor - 1) < 1e-4) continue;
      for (const canonical of rule.bones) {
        const bone = findBone(byName, canonical);
        const snap = rest.get(bone?.name);
        if (!bone || !snap) continue;
        scaleAxis(bone, snap, factor, rule.width);
      }
    }
    rebindSkinnedMeshes(root);
    return { ...state };
  }

  return {
    apply,
    get: () => ({ ...state }),
    reset: () => apply({ ...DEFAULTS }),
    boneCount: byName.size,
  };
}
