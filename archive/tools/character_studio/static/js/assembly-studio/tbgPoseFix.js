/** TBG GLB preview — auto-correct inverted skin bind + ground on mesh bbox. */

import * as THREE from "three";

const _v = new THREE.Vector3();

export function refreshSkinnedMeshes(root) {
  root?.traverse((obj) => {
    if (obj.isSkinnedMesh?.skeleton) obj.skeleton.update();
  });
}

function dominantBoneY(root) {
  let headY = -Infinity;
  let footY = Infinity;
  root.traverse((obj) => {
    if (!obj.isSkinnedMesh) return;
    const pos = obj.geometry.attributes.position;
    const skinIndex = obj.geometry.attributes.skinIndex;
    const skinWeight = obj.geometry.attributes.skinWeight;
    const bones = obj.skeleton.bones;
    const step = Math.max(1, Math.floor(pos.count / 8000));
    for (let i = 0; i < pos.count; i += step) {
      _v.fromBufferAttribute(pos, i);
      obj.localToWorld(_v);
      const weights = [skinWeight.getX(i), skinWeight.getY(i), skinWeight.getZ(i), skinWeight.getW(i)];
      const indices = [skinIndex.getX(i), skinIndex.getY(i), skinIndex.getZ(i), skinIndex.getW(i)];
      let best = 0;
      let boneIdx = indices[0];
      for (let j = 0; j < 4; j += 1) {
        if (weights[j] > best) {
          best = weights[j];
          boneIdx = indices[j];
        }
      }
      const name = bones[boneIdx]?.name || "";
      if (name.includes("Head")) headY = Math.max(headY, _v.y);
      if (name.includes("Foot")) footY = Math.min(footY, _v.y);
    }
  });
  return { headY, footY };
}

function isMeshInvertedY(root) {
  const { headY, footY } = dominantBoneY(root);
  return Number.isFinite(headY) && Number.isFinite(footY) && headY < footY;
}

function placeOnGroundMesh(object) {
  const box = new THREE.Box3().setFromObject(object);
  const center = box.getCenter(new THREE.Vector3());
  object.position.x -= center.x;
  object.position.z -= center.z;
  object.position.y -= box.min.y;
}

/** Stand character on grid; flip 180° when template bind puts head below feet. */
export function stabilizeTbgCharacter(root) {
  root.updateMatrixWorld(true);
  refreshSkinnedMeshes(root);
  if (isMeshInvertedY(root)) {
    root.rotation.z = Math.PI;
    root.updateMatrixWorld(true);
    refreshSkinnedMeshes(root);
  }
  placeOnGroundMesh(root);
  root.updateMatrixWorld(true);
}
