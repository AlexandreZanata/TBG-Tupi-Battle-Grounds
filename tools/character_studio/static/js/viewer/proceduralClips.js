/** Procedural clips for static (non-rigged) mesh3d previews — until Blender rig export. */

import * as THREE from "three";

const CLIP_ALIASES = {
  AttackHeavy: "Attack",
  AttackLight: "Attack",
  Cast: "Attack",
  Shot: "Attack",
  Spawn: "Idle",
};

export function resolveClipName(name) {
  return CLIP_ALIASES[name] ?? name;
}

export function buildProceduralClips() {
  return [idleClip(), attackClip(), deathClip()];
}

function idleClip() {
  const duration = 2.4;
  const t = [0, duration * 0.5, duration];
  return new THREE.AnimationClip("Idle", duration, [
    new THREE.NumberKeyframeTrack(".position[y]", t, [0, 0.028, 0]),
    new THREE.NumberKeyframeTrack(".rotation[y]", t, [0, 0.05, 0]),
  ]);
}

function attackClip() {
  const duration = 0.55;
  const t = [0, 0.12, 0.28, duration];
  return new THREE.AnimationClip("Attack", duration, [
    new THREE.NumberKeyframeTrack(".position[z]", t, [0, 0.14, 0.08, 0]),
    new THREE.NumberKeyframeTrack(".rotation[x]", t, [0, -0.12, 0.04, 0]),
    new THREE.NumberKeyframeTrack(".scale[x]", t, [1, 1.04, 1, 1]),
    new THREE.NumberKeyframeTrack(".scale[y]", t, [1, 1.04, 1, 1]),
    new THREE.NumberKeyframeTrack(".scale[z]", t, [1, 1.04, 1, 1]),
  ]);
}

function deathClip() {
  const duration = 0.9;
  const t = [0, 0.35, duration];
  return new THREE.AnimationClip("Death", duration, [
    new THREE.NumberKeyframeTrack(".rotation[x]", t, [0, -1.35, -1.55]),
    new THREE.NumberKeyframeTrack(".position[y]", t, [0, 0.02, -0.12]),
  ]);
}
