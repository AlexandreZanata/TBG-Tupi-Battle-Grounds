/** Ground placement, camera fit, and disposal helpers for the 3D viewer. */

import * as THREE from "three";

const FIT_PADDING = 1.28;
const FOOT_BONES = ["TBG_Foot.L", "TBG_Foot.R", "TBG_Toe.L", "TBG_Toe.R"];
const _footPos = new THREE.Vector3();

export function placeOnGround(object, rig = null) {
  const box = new THREE.Box3().setFromObject(object);
  const center = box.getCenter(new THREE.Vector3());
  object.position.x -= center.x;
  object.position.z -= center.z;

  let floorLift = box.min.y;
  if (rig) {
    rig.updateMatrixWorld(true);
    let minFootY = Infinity;
    rig.traverse((obj) => {
      if (!obj.isBone || !FOOT_BONES.includes(obj.name)) return;
      minFootY = Math.min(minFootY, obj.getWorldPosition(_footPos).y);
    });
    if (Number.isFinite(minFootY)) floorLift = minFootY;
  }
  object.position.y -= floorLift;
  object.userData.groundY = object.position.y;
}

export function fitCamera(object, camera, width, height) {
  const sphere = new THREE.Box3().setFromObject(object).getBoundingSphere(new THREE.Sphere());
  const radius = Math.max(sphere.radius, 0.001);
  const fovRad = (camera.fov * Math.PI) / 180;
  const aspect = width / Math.max(height, 1);
  const hFov = 2 * Math.tan(Math.tan(fovRad / 2) * aspect);
  const distY = radius / Math.sin(fovRad / 2);
  const distX = radius / Math.sin(hFov / 2);
  const distance = Math.max(distX, distY) * FIT_PADDING;
  camera.position.set(distance * 0.55, distance * 0.42, distance * 0.92);
  camera.lookAt(0, radius * 0.42, 0);
  camera.near = Math.max(distance / 200, 0.001);
  camera.far = distance * 24;
  camera.updateProjectionMatrix();
}

export function disposeObject(object) {
  object.traverse((child) => {
    if (!(child instanceof THREE.Mesh)) return;
    child.geometry?.dispose();
    const mats = Array.isArray(child.material) ? child.material : [child.material];
    mats.forEach((m) => m?.dispose?.());
  });
}
