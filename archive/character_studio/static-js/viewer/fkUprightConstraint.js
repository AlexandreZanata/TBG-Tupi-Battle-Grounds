/**
 * Character-space upright — remove world yaw/roll after mixer.
 * Local Euler Y/Z clamp fails when TBG_Root bind is -90° X (Blender → glTF).
 */

import * as THREE from "three";

const TORSO_BONES = ["TBG_Spine", "TBG_Spine1", "TBG_Neck", "TBG_Head"];
const STABILIZE_ROOT = "TBG_Root";
const WEAPON_ARM = ["TBG_Shoulder.R", "TBG_UpperArm.R", "TBG_LowerArm.R", "TBG_Hand.R"];

const _wq = new THREE.Quaternion();
const _pq = new THREE.Quaternion();
const _lq = new THREE.Quaternion();
const _euler = new THREE.Euler(0, 0, 0, "YXZ");

let skipDuringLocomotion = false;

/** Skip torso clamp while Walk plays — avoids mesh smear with FK walk + foot IK. */
export function setFkUprightLocomotionPreview(enabled) {
  skipDuringLocomotion = Boolean(enabled);
}

function applyWorldPitchOnly(bone) {
  bone.getWorldQuaternion(_wq);
  _euler.setFromQuaternion(_wq, "YXZ");
  _euler.y = 0;
  _euler.z = 0;
  _lq.setFromEuler(_euler);
  if (bone.parent) {
    bone.parent.getWorldQuaternion(_pq);
    bone.quaternion.copy(_pq.invert().multiply(_lq));
  } else {
    bone.quaternion.copy(_lq);
  }
  bone.updateWorldMatrix(true, false);
}

function applyWorldYawOnly(bone) {
  bone.getWorldQuaternion(_wq);
  _euler.setFromQuaternion(_wq, "YXZ");
  _euler.y = 0;
  _lq.setFromEuler(_euler);
  if (bone.parent) {
    bone.parent.getWorldQuaternion(_pq);
    bone.quaternion.copy(_pq.invert().multiply(_lq));
  } else {
    bone.quaternion.copy(_lq);
  }
}

export function enforceFkUpright(rig) {
  if (!rig || skipDuringLocomotion) return;
  const byName = new Map();
  rig.traverse((obj) => {
    if (obj.isBone) byName.set(obj.name, obj);
  });

  const root = byName.get(STABILIZE_ROOT);
  if (root) applyWorldPitchOnly(root);

  for (const name of TORSO_BONES) {
    const bone = byName.get(name);
    if (bone) applyWorldPitchOnly(bone);
  }

  for (const name of WEAPON_ARM) {
    const bone = byName.get(name);
    if (bone) applyWorldYawOnly(bone);
  }
}
