/** Shared Three.js helpers for Mixamo align/result viewers. */

import * as THREE from "three";

export const TARGET_HEIGHT = 1.8;

export function worldBox(obj) {
  obj.updateMatrixWorld(true);
  return new THREE.Box3().setFromObject(obj);
}

export function placeOnGround(root) {
  const box = worldBox(root);
  if (box.isEmpty()) return;
  root.position.y -= box.min.y;
}

/** Force X/Y/Z scale equal — never stretch one axis alone. */
export function enforceUniformScale(obj) {
  const { x, y, z } = obj.scale;
  const u = (x + y + z) / 3;
  if (Math.abs(x - u) > 1e-6 || Math.abs(y - u) > 1e-6 || Math.abs(z - u) > 1e-6) {
    obj.scale.set(u, u, u);
  }
  return u;
}

export function normalizeHeight(root, targetH = TARGET_HEIGHT) {
  enforceUniformScale(root);
  const box = worldBox(root);
  const h = box.getSize(new THREE.Vector3()).y;
  if (h < 1e-4) return;
  const s = targetH / h;
  if (s < 0.2 || s > 5) objScaleUniform(root, root.scale.x * s);
}

export function objScaleUniform(obj, factor) {
  obj.scale.set(factor, factor, factor);
}

export function brightenMeshes(root, { skin = false, opacity = 1 } = {}) {
  root.traverse((obj) => {
    if (!obj.isMesh) return;
    obj.frustumCulled = false;
    const mats = Array.isArray(obj.material) ? obj.material : [obj.material];
    for (const mat of mats) {
      if (!mat) continue;
      mat.transparent = opacity < 0.99;
      mat.opacity = opacity;
      mat.depthWrite = opacity > 0.45;
      mat.side = THREE.DoubleSide;
      if (skin) mat.color?.set?.(0xd4a574);
      if (!mat.color || mat.color.getHex() === 0) mat.color = new THREE.Color(skin ? 0xd4a574 : 0x9ab4d4);
      mat.needsUpdate = true;
    }
  });
}

export function fitCamera(target, camera, controls) {
  const box = worldBox(target);
  if (box.isEmpty()) return;
  const size = box.getSize(new THREE.Vector3());
  const center = box.getCenter(new THREE.Vector3());
  const span = Math.max(size.x, size.y, size.z, 0.5);
  const dist = span * 2.8;
  camera.near = Math.max(0.001, span / 500);
  camera.far = Math.max(500, dist * 8);
  camera.updateProjectionMatrix();
  camera.position.set(center.x + dist * 0.55, center.y + span * 0.45, center.z + dist * 0.85);
  controls.target.copy(center);
  controls.update();
}
