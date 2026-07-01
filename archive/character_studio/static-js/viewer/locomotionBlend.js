/**
 * Idle ↔ Walk — discrete crossfade (never blend both clips at once).
 * Simultaneous weights tear the mesh at the hips (idle torso + walk legs).
 */

import * as THREE from "three";

const IDLE = "Idle";
const WALK = "Walk";
const WALK_ROOT = "WalkRoot";
const CROSSFADE_SEC = 0.22;
const SPEED_ON = 0.12;
const SPEED_OFF = 0.05;

export function createLocomotionBlend(mixer, clips, { useRootMotion = false } = {}) {
  const walkName = useRootMotion ? WALK_ROOT : WALK;
  const idleClip = clips.find((c) => c.name === IDLE);
  const walkClip = clips.find((c) => c.name === walkName) ?? clips.find((c) => c.name === WALK);
  if (!mixer || !idleClip || !walkClip) {
    return { update() {}, dispose() {} };
  }

  const idle = mixer.clipAction(idleClip);
  const walk = mixer.clipAction(walkClip);
  let mode = "idle";

  idle.setLoop(THREE.LoopRepeat, Infinity);
  walk.setLoop(THREE.LoopRepeat, Infinity);
  idle.play();
  idle.setEffectiveWeight(1);
  walk.setEffectiveWeight(0);

  function enterWalk() {
    if (mode === "walk") return;
    walk.reset().play();
    idle.crossFadeTo(walk, CROSSFADE_SEC, false);
    mode = "walk";
  }

  function enterIdle() {
    if (mode === "idle") return;
    idle.reset().play();
    walk.crossFadeTo(idle, CROSSFADE_SEC, false);
    mode = "idle";
  }

  return {
    update(speed, maxSpeed = 2.4) {
      const moving = mode === "walk" ? speed > SPEED_OFF : speed > SPEED_ON;
      if (moving) {
        enterWalk();
        const t = THREE.MathUtils.clamp(speed / maxSpeed, 0, 1);
        walk.timeScale = THREE.MathUtils.lerp(0.9, 1.15, t);
      } else {
        enterIdle();
        walk.timeScale = 1;
      }
    },
    dispose() {
      idle.stop();
      walk.stop();
    },
  };
}
