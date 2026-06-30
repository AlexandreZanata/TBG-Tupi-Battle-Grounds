/**
 * Post-mixer rotation damping — softens stiff LINEAR glTF quaternion blends.
 */

import * as THREE from "three";

const SMOOTH_BONES = new Set([
  "TBG_Spine",
  "TBG_Spine1",
  "TBG_UpperLeg.L",
  "TBG_UpperLeg.R",
  "TBG_LowerLeg.L",
  "TBG_LowerLeg.R",
  "TBG_Foot.L",
  "TBG_Foot.R",
  "TBG_Shoulder.R",
  "TBG_UpperArm.R",
  "TBG_LowerArm.R",
]);

export function createMotionSmoothing(rig) {
  const bones = new Map();
  const smoothed = new Map();

  rig.traverse((obj) => {
    if (obj.isBone && SMOOTH_BONES.has(obj.name)) {
      bones.set(obj.name, obj);
      smoothed.set(obj.name, obj.quaternion.clone());
    }
  });

  let enabled = false;
  let strength = 0.42;

  return {
    setEnabled(value) {
      enabled = Boolean(value);
      if (enabled) {
        bones.forEach((bone, name) => smoothed.get(name).copy(bone.quaternion));
      }
    },
    afterMixerUpdate(delta) {
      if (!enabled || delta <= 0) return;
      const t = 1 - Math.exp(-14 * strength * delta);
      bones.forEach((bone, name) => {
        const state = smoothed.get(name);
        state.slerp(bone.quaternion, t);
        bone.quaternion.copy(state);
      });
    },
  };
}
