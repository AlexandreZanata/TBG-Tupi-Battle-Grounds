/**
 * Bone spring secondary motion — ossos-inspired lag on spine/arms (experimental).
 */

import * as THREE from "three";

const SPRING_BONES = new Set([
  "TBG_Spine1",
  "TBG_Neck",
  "TBG_UpperArm.L",
  "TBG_UpperArm.R",
]);

export function createBoneSpring(rig) {
  const bones = new Map();
  const state = new Map();

  rig.traverse((obj) => {
    if (obj.isBone && SPRING_BONES.has(obj.name)) {
      bones.set(obj.name, obj);
      state.set(obj.name, obj.quaternion.clone());
    }
  });

  let enabled = false;
  const strength = 0.12;

  return {
    setEnabled(value) {
      enabled = Boolean(value);
      if (enabled) {
        bones.forEach((bone, name) => state.get(name).copy(bone.quaternion));
      }
    },
    afterUprightUpdate(delta) {
      if (!enabled || delta <= 0) return;
      const t = 1 - Math.exp(-10 * strength * delta);
      bones.forEach((bone, name) => {
        const spring = state.get(name);
        spring.slerp(bone.quaternion, t);
        bone.quaternion.copy(spring);
      });
    },
  };
}
