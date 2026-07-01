/**
 * Foot IK preview — Animation Studio walk planting (mirrors Unity LocomotionFootIk).
 */

import * as THREE from "three";
import { solveTwoBoneIk } from "./twoBoneIk.js";
import { refreshSkinnedMeshes } from "./skinRefresh.js";

const RAY_HEIGHT = 0.45;
const RAY_DISTANCE = 1.2;
const POLE_OFFSET = 0.25;
const ARENA_FLOOR_Y = 0;

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
  let floorY = ARENA_FLOOR_Y;

  return {
    setFloorY(y) {
      floorY = y;
    },
    setEnabled(value) {
      enabled = Boolean(value);
      markers.visible = enabled;
    },
    setWalkActive(value) {
      activeWalk = Boolean(value);
    },
    update() {
      if (!enabled || !activeWalk) return;
      LEGS.forEach((spec, index) => {
        const chain = bones[index];
        if (!chain.foot) return;
        const footPos = chain.foot.getWorldPosition(new THREE.Vector3());
        const hit = raycastGround(footPos, floorY);
        const pole = chain.upper.getWorldPosition(new THREE.Vector3()).add(
          new THREE.Vector3(spec.side * POLE_OFFSET, 0, 0),
        );
        solveTwoBoneIk(chain.upper, chain.lower, chain.foot, hit, pole);
        updateRay(lines[index], footPos, hit);
      });
      refreshSkinnedMeshes(rig);
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

function raycastGround(footPos, floorY) {
  const originY = footPos.y + RAY_HEIGHT;
  const hitY = Math.max(floorY, originY - RAY_DISTANCE);
  return new THREE.Vector3(footPos.x, hitY, footPos.z);
}

function updateRay(line, from, to) {
  line.geometry.setFromPoints([from.clone(), to.clone()]);
  line.geometry.attributes.position.needsUpdate = true;
}
