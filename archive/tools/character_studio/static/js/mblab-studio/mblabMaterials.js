/** Tint MB-Lab skin + attached gear in the browser (instant). */

import * as THREE from "three";

const SKIP_MAT = /eyelash|pupil|teeth|sclera|lacrimal|tongue|nail|cornea/i;
const SKIN_MAT = /human|skin|body|generic|dermal/i;
const GEAR_MAT = /hair|gear|armor|weapon|hat|leather|plate|tbg_hair/i;

export function isMblabSkinMaterial(name) {
  const n = String(name || "");
  if (!n || SKIP_MAT.test(n)) return false;
  return SKIN_MAT.test(n);
}

export function applyLook(root, { skinHex, gearHex }) {
  if (!root) return;
  const skin = new THREE.Color(skinHex || "#b8a99e");
  const gear = new THREE.Color(gearHex || "#8c8578");
  root.traverse((obj) => {
    if (!obj.isMesh || !obj.material) return;
    const mats = Array.isArray(obj.material) ? obj.material : [obj.material];
    const useGear = obj.userData.tbgGear === true;
    for (const mat of mats) {
      if (!mat.color) continue;
      const name = mat.name || "";
      if (useGear || GEAR_MAT.test(name)) mat.color.copy(gear);
      else if (isMblabSkinMaterial(name)) mat.color.copy(skin);
    }
  });
}
