/**
 * Two-bone IK solver — debug overlay only (Animation Studio).
 * Pattern from ossos/FABRIK; no external THREE.IK dependency.
 */

import * as THREE from "three";

export function clampReach(root, target, maxReach) {
  const offset = new THREE.Vector3().subVectors(target, root);
  const distSq = offset.lengthSq();
  if (distSq <= maxReach * maxReach) {
    return target.clone();
  }
  return root.clone().add(offset.normalize().multiplyScalar(maxReach));
}

export function solveTwoBoneIk(upper, lower, end, target, poleHint) {
  if (!upper || !lower || !end) return false;

  const root = upper.getWorldPosition(new THREE.Vector3());
  const mid = lower.getWorldPosition(new THREE.Vector3());
  const upperLen = root.distanceTo(mid);
  const lowerLen = mid.distanceTo(end.getWorldPosition(new THREE.Vector3()));
  if (upperLen < 1e-4 || lowerLen < 1e-4) return false;

  const clamped = clampReach(root, target, upperLen + lowerLen - 1e-3);
  const toTarget = new THREE.Vector3().subVectors(clamped, root);
  const dist = toTarget.length();
  if (dist < 1e-4) return false;

  const cosUpper = (upperLen * upperLen + dist * dist - lowerLen * lowerLen) / (2 * upperLen * dist);
  const upperAngle = Math.acos(THREE.MathUtils.clamp(cosUpper, -1, 1));
  const bendNormal = new THREE.Vector3().crossVectors(toTarget, new THREE.Vector3().subVectors(poleHint, root));
  if (bendNormal.lengthSq() < 1e-6) bendNormal.crossVectors(toTarget, new THREE.Vector3(0, 1, 0));
  bendNormal.normalize();

  const upperDir = toTarget
    .clone()
    .normalize()
    .applyAxisAngle(bendNormal, upperAngle);
  const desiredMid = root.clone().add(upperDir.multiplyScalar(upperLen));
  rotateBoneToward(upper, mid, desiredMid);
  rotateBoneToward(lower, end.getWorldPosition(new THREE.Vector3()), clamped);
  upper.updateMatrixWorld(true);
  lower.updateMatrixWorld(true);
  end.updateMatrixWorld(true);
  return true;
}

function rotateBoneToward(bone, fromWorld, toWorld) {
  const parent = bone.parent;
  const fromLocal = parent.worldToLocal(fromWorld.clone());
  const toLocal = parent.worldToLocal(toWorld.clone());
  const delta = new THREE.Quaternion().setFromUnitVectors(
    fromLocal.sub(bone.position).normalize(),
    toLocal.sub(bone.position).normalize(),
  );
  bone.quaternion.premultiply(delta);
}
