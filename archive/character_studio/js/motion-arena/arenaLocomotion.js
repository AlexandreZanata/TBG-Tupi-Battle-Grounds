/**
 * Arena locomotion — uses viewer.playClip (same path as Animation Studio).
 * In-place Walk + root translation: match speed to stride/cadence to avoid foot slide.
 */

import * as THREE from "three";

const IDLE = "Idle";
const WALK = "Walk";
const WALK_ROOT = "WalkRoot";
const SPEED_OFF = 0.03;
/** Hold idle (neutral feet) briefly after input before Walk clip + translation. */
const WALK_ARM_DELAY_SEC = 0.18;
/** HumanoidHeavy ~1.4 m stride @ 95 spm → ~1.1 m/s (see walk/style.py). */
export const DEFAULT_WALK_MPS = 1.12;

export function createArenaLocomotion(viewer, { useRootMotion = false, walkMps = DEFAULT_WALK_MPS } = {}) {
  const walkName = useRootMotion ? WALK_ROOT : WALK;
  let mode = "idle";
  let walkEngaged = false;
  let armTimer = 0;

  viewer.playClip(IDLE, true);

  return {
    updateInput(delta, hasInput) {
      armTimer = hasInput ? armTimer + delta : 0;
      const wantWalk = hasInput && armTimer >= WALK_ARM_DELAY_SEC;

      if (wantWalk && mode !== "walk") {
        viewer.playClip(walkName, true);
        mode = "walk";
      } else if (!hasInput && mode !== "idle") {
        viewer.playClip(IDLE, true);
        mode = "idle";
      }
      walkEngaged = mode === "walk";
    },
    updateSpeed(speed) {
      if (mode !== "walk") {
        viewer.setClipTimeScale(1);
        return;
      }
      const moving = speed > SPEED_OFF;
      if (!moving) {
        viewer.setClipTimeScale(0.35);
        return;
      }
      const scale = THREE.MathUtils.clamp(speed / walkMps, 0.85, 1.25);
      viewer.setClipTimeScale(scale);
    },
    isWalkEngaged() {
      return walkEngaged;
    },
    dispose() {},
  };
}
