/** Joint handles — numbered sprite, region color, lock ring. */

import * as THREE from "three";
import { bonePlacementNumber, regionHex } from "./rigBoneRegions.js";
import { createBoneLabelSprite } from "./rigBoneLabels.js";

function hex(color) {
  return new THREE.Color(color);
}

const IDLE_HALO_OPACITY = 0.1;

export function createJointHandle(boneName) {
  const group = new THREE.Group();
  const color = regionHex(boneName);
  const number = bonePlacementNumber(boneName);
  group.userData.boneName = boneName;
  group.userData.placementNumber = number;
  group.userData.locked = false;

  const label = createBoneLabelSprite(number || "?", color);
  label.position.set(0, 0.032, 0);

  const core = new THREE.Mesh(
    new THREE.SphereGeometry(0.008, 8, 8),
    new THREE.MeshBasicMaterial({
      color: hex(color),
      transparent: true,
      opacity: 0.45,
      depthTest: true,
      depthWrite: false,
    }),
  );
  core.renderOrder = 2;

  const halo = new THREE.Mesh(
    new THREE.SphereGeometry(0.022, 10, 10),
    new THREE.MeshBasicMaterial({
      color: hex(color),
      transparent: true,
      opacity: IDLE_HALO_OPACITY,
      depthTest: false,
      depthWrite: false,
    }),
  );
  halo.renderOrder = 3;

  const lockRing = new THREE.Mesh(
    new THREE.RingGeometry(0.034, 0.041, 24),
    new THREE.MeshBasicMaterial({
      color: 0x4ade80,
      transparent: true,
      opacity: 0.75,
      depthTest: false,
      side: THREE.DoubleSide,
    }),
  );
  lockRing.renderOrder = 4;
  lockRing.visible = false;
  lockRing.rotation.x = Math.PI / 2;

  const tail = new THREE.Mesh(
    new THREE.SphereGeometry(0.006, 6, 6),
    new THREE.MeshBasicMaterial({
      color: hex(color),
      transparent: true,
      opacity: 0.5,
      depthTest: false,
      depthWrite: false,
    }),
  );
  tail.renderOrder = 3;
  tail.userData.isTail = true;

  group.add(core, halo, lockRing, tail, label);
  applyHandleVisual(group, "idle");
  return group;
}

export function applyHandleVisual(group, state) {
  const core = group.children[0];
  const halo = group.children[1];
  const lockRing = group.children[2];
  const label = group.children[4];
  const color = regionHex(group.userData.boneName);
  const locked = Boolean(group.userData.locked);

  lockRing.visible = locked;
  if (locked) {
    halo.material.opacity = 0.06;
    core.material.opacity = 0.35;
    label.material.opacity = 0.92;
    group.scale.setScalar(1);
    return;
  }
  group.scale.setScalar(1);
  if (state === "active") {
    halo.material.color.setHex(0xef4444);
    halo.material.opacity = 0.35;
    core.material.opacity = 0.7;
    label.material.opacity = 0.95;
    return;
  }
  if (state === "hover") {
    halo.material.color.copy(hex(color));
    halo.material.opacity = 0.22;
    core.material.opacity = 0.6;
    label.material.opacity = 0.88;
    return;
  }
  halo.material.color.copy(hex(color));
  halo.material.opacity = IDLE_HALO_OPACITY;
  core.material.opacity = 0.45;
  label.material.opacity = 0.72;
}

export function pickHandleMeshes(handles) {
  const meshes = [];
  handles.forEach((group) => {
    group.traverse((child) => {
      if (child.userData?.isLabel || child.userData?.isTail) return;
      if (child === group.children[2]) return;
      if (child.isMesh || child.isSprite) meshes.push(child);
    });
  });
  return meshes;
}
