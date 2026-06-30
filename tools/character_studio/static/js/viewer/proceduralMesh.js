/** Procedural blockout mesh — mirrors headless card silhouette in Three.js. */

import * as THREE from "three";

function hexToColor(hex) {
  const value = hex.replace("#", "");
  return new THREE.Color(`#${value}`);
}

export function buildProceduralCharacter(definition) {
  const root = new THREE.Group();
  const primary = hexToColor(definition.primaryColor);
  const secondary = hexToColor(definition.secondaryColor);
  const height = definition.heightScale ?? 1;
  const bodyW = definition.bodyWidthScale ?? 1;
  const headS = definition.headScale ?? 1;
  const limbS = definition.limbLengthScale ?? 1;

  const torsoGeo = new THREE.BoxGeometry(0.7 * bodyW, 1.1 * height, 0.45 * bodyW);
  const torso = new THREE.Mesh(
    torsoGeo,
    new THREE.MeshStandardMaterial({ color: primary, roughness: 0.55, metalness: 0.08 }),
  );
  torso.position.y = 1.1 * height * 0.5;
  root.add(torso);

  const headGeo = new THREE.SphereGeometry(0.28 * headS, 20, 20);
  const head = new THREE.Mesh(
    headGeo,
    new THREE.MeshStandardMaterial({ color: primary, roughness: 0.5, metalness: 0.05 }),
  );
  head.position.y = 1.1 * height + 0.35 * headS;
  root.add(head);

  const limbMat = new THREE.MeshStandardMaterial({ color: secondary, roughness: 0.6, metalness: 0.05 });
  const legGeo = new THREE.BoxGeometry(0.22 * bodyW, 0.8 * limbS, 0.22 * bodyW);
  const leftLeg = new THREE.Mesh(legGeo, limbMat);
  leftLeg.position.set(-0.2 * bodyW, 0.4 * limbS, 0);
  root.add(leftLeg);
  const rightLeg = leftLeg.clone();
  rightLeg.position.x = 0.2 * bodyW;
  root.add(rightLeg);

  const armGeo = new THREE.BoxGeometry(0.14, 0.7 * limbS, 0.14);
  const leftArm = new THREE.Mesh(armGeo, limbMat);
  leftArm.position.set(-0.48 * bodyW, 1.0 * height, 0);
  root.add(leftArm);
  const rightArm = leftArm.clone();
  rightArm.position.x = 0.48 * bodyW;
  root.add(rightArm);

  return root;
}
