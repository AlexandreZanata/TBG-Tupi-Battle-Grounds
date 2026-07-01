/** Apply walk deltas in bone-local space — cached bones, no per-frame traverse. */

import { cycleFrames, HUMANOID_LIGHT_STYLE, sampleWalkPose } from "./walkGait.js";
import { buildBoneCache } from "./walkBoneCache.js";

export const WALK_BONE_ALIASES = {
  "TBG_LowerLeg.R": "calf_R",
  "TBG_LowerLeg.L": "calf_L",
  "TBG_Toe.R": "toes_R",
  "TBG_Toe.L": "toes_L",
};

const SPINE1_SPLIT = ["TBG_Spine1", "TBG_Spine1.001"];

const TWIST_FROM_LOGICAL = [
  { twist: "thigh_twist_R", logical: "TBG_UpperLeg.R", factor: 0.28 },
  { twist: "thigh_twist_L", logical: "TBG_UpperLeg.L", factor: 0.28 },
  { twist: "calf_twist_R", logical: "TBG_LowerLeg.R", factor: 0.38 },
  { twist: "calf_twist_L", logical: "TBG_LowerLeg.L", factor: 0.38 },
  { twist: "upperarm_twist_R", logical: "TBG_UpperArm.R", factor: 0.32 },
  { twist: "upperarm_twist_L", logical: "TBG_UpperArm.L", factor: 0.32 },
  { twist: "lowerarm_twist_R", logical: "TBG_LowerArm.R", factor: 0.35 },
  { twist: "lowerarm_twist_L", logical: "TBG_LowerArm.L", factor: 0.35 },
];

function resolveTargets(logical, boneNames) {
  const alias = WALK_BONE_ALIASES[logical];
  if (logical === "TBG_Spine1") {
    return SPINE1_SPLIT.filter((n) => boneNames.has(n));
  }
  if (boneNames.has(logical)) return [logical];
  if (alias && boneNames.has(alias)) return [alias];
  return [];
}

function applyLocalDelta(bone, snap, entry) {
  bone.quaternion.copy(snap.q);
  bone.position.copy(snap.p);
  const rot = entry?.rot;
  if (rot) {
    if (rot[0]) bone.rotateX(rot[0]);
    if (rot[1]) bone.rotateY(rot[1]);
    if (rot[2]) bone.rotateZ(rot[2]);
  }
  const pos = entry?.pos;
  if (pos) {
    bone.position.x += pos[0] ?? 0;
    bone.position.y += pos[1] ?? 0;
    bone.position.z += pos[2] ?? 0;
  }
}

function applyTwist(cache, rest, pose) {
  for (const { twist, logical, factor } of TWIST_FROM_LOGICAL) {
    const deltaX = pose[logical]?.rot?.[0];
    if (deltaX == null || Math.abs(deltaX) < 1e-6) continue;
    const twistBone = cache.byName.get(twist);
    const snap = rest.get(twist);
    if (!twistBone || !snap) continue;
    twistBone.quaternion.copy(snap.q);
    twistBone.position.copy(snap.p);
    twistBone.rotateX(deltaX * factor);
  }
}

/** Build applier once when walk starts — zero traversals during playback. */
export function createWalkApplier(root, style = HUMANOID_LIGHT_STYLE) {
  const byName = buildBoneCache(root);
  const boneNames = new Set(byName.keys());
  const channels = [];

  for (const logical of Object.keys(sampleWalkPose(0, cycleFrames(style.cadenceSpm), style))) {
    for (const name of resolveTargets(logical, boneNames)) {
      const bone = byName.get(name);
      if (bone) channels.push({ bone, logical });
    }
  }

  return {
    byName,
    channels,
    applyFrame(_frame, rest, pose) {
      for (const ch of channels) {
        const snap = rest.get(ch.bone.name);
        const entry = pose[ch.logical];
        if (!snap || !entry) continue;
        applyLocalDelta(ch.bone, snap, entry);
      }
      applyTwist({ byName }, rest, pose);
    },
  };
}

export function walkCycleMeta(style = HUMANOID_LIGHT_STYLE) {
  const frames = cycleFrames(style.cadenceSpm);
  return { frames, duration: frames / 30, fps: 30 };
}
