/**
 * Foot IK debug overlay — Animation Studio only, not shipped in game build.
 */

import * as THREE from "three";
import { solveTwoBoneIk } from "./twoBoneIk.js";

const LEGS = [
  { upper: "TBG_UpperLeg.L", lower: "TBG_LowerLeg.L", foot: "TBG_Foot.L", side: -1 },
  { upper: "TBG_UpperLeg.R", lower: "TBG_LowerLeg.R", foot: "TBG_Foot.R", side: 1 },
];

export function createFootIkDebug(rig) {
  const bones = mapLegBones(rig);
  const markers = new THREE.Group();
  markers.name = "FootIkDebug";
  rig.add(markers);

  const rayMaterial = new THREE.LineBasicMaterial({ color: 0x44ff88 });
  const lines = LEGS.map(() => {
    const geometry = new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(), new THREE.Vector3()]);
    const line = new THREE.Line(geometry, rayMaterial);
    markers.add(line);
    return line;
  });

  let enabled = false;
  let activeWalk = false;

  return {
    setEnabled(value) {
      enabled = Boolean(value);
      markers.visible = enabled;
    },
    setWalkActive(value) {
      activeWalk = Boolean(value);
    },
    update() {
      if (!enabled || !activeWalk) return;
      const groundY = estimateGroundY(rig);
      LEGS.forEach((spec, index) => {
        const chain = bones[index];
        if (!chain.foot) return;
        const footPos = chain.foot.getWorldPosition(new THREE.Vector3());
        const hit = new THREE.Vector3(footPos.x, groundY, footPos.z);
        const pole = chain.upper.getWorldPosition(new THREE.Vector3()).add(
          new THREE.Vector3(spec.side * 0.25, 0, 0),
        );
        solveTwoBoneIk(chain.upper, chain.lower, chain.foot, hit, pole);
        updateRay(lines[index], footPos, hit);
      });
    },
    dispose() {
      markers.removeFromParent();
      lines.forEach((line) => {
        line.geometry.dispose();
      });
      rayMaterial.dispose();
    },
  };
}

function mapLegBones(rig) {
  const byName = new Map();
  rig.traverse((obj) => {
    if (obj.isBone) byName.set(obj.name, obj);
  });
  return LEGS.map((spec) => ({
    upper: byName.get(spec.upper) ?? null,
    lower: byName.get(spec.lower) ?? null,
    foot: byName.get(spec.foot) ?? null,
  }));
}

function estimateGroundY(rig) {
  const box = new THREE.Box3().setFromObject(rig);
  return box.min.y;
}

function updateRay(line, from, to) {
  line.geometry.setFromPoints([from.clone(), to.clone()]);
  line.geometry.attributes.position.needsUpdate = true;
}
