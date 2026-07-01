/** Cyan bone wire overlay in rig viewport. */

import * as THREE from "three";

export function isRigParent(node, prefix) {
  return node?.name?.startsWith(prefix);
}

export function createBoneLines(scene, rigNodes, prefix) {
  const points = [];
  rigNodes.forEach((node) => {
    if (!isRigParent(node.parent, prefix)) return;
    points.push(new THREE.Vector3(), new THREE.Vector3());
  });
  if (!points.length) return null;
  const lines = new THREE.LineSegments(
    new THREE.BufferGeometry().setFromPoints(points),
    new THREE.LineBasicMaterial({ color: 0x38bdf8, depthTest: false, transparent: true, opacity: 0.32 }),
  );
  lines.renderOrder = 2;
  lines.frustumCulled = false;
  scene.add(lines);
  return lines;
}

export function syncBoneLines(boneLines, rigNodes, prefix) {
  if (!boneLines) return;
  const positions = boneLines.geometry.attributes.position;
  let i = 0;
  const parentPos = new THREE.Vector3();
  const nodePos = new THREE.Vector3();
  rigNodes.forEach((node) => {
    if (!isRigParent(node.parent, prefix)) return;
    node.parent.getWorldPosition(parentPos);
    node.getWorldPosition(nodePos);
    positions.setXYZ(i++, parentPos.x, parentPos.y, parentPos.z);
    positions.setXYZ(i++, nodePos.x, nodePos.y, nodePos.z);
  });
  positions.needsUpdate = true;
}

export function disposeBoneLines(scene, boneLines) {
  if (!boneLines) return null;
  scene.remove(boneLines);
  boneLines.geometry.dispose();
  boneLines.material.dispose();
  return null;
}
