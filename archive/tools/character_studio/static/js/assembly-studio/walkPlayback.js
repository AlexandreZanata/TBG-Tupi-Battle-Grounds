/** Procedural walk loop — 30 Hz pose tick, cached bones (no per-frame traverse). */

import { refreshSkinnedMeshes } from "./tbgPoseFix.js";
import { captureRestPose, restoreAllBones } from "./walkBoneCache.js";
import { sampleWalkPose } from "./walkGait.js";
import { createWalkApplier, walkCycleMeta } from "./walkPose.js";

export function createWalkPlayback(root) {
  let playing = false;
  let time = 0;
  let tickAccum = 0;
  const meta = walkCycleMeta();
  const step = 1 / meta.fps;
  let rest = null;
  let applier = null;
  let boneByName = null;

  function bindRoot(nextRoot) {
    rest = null;
    applier = null;
    boneByName = null;
    if (!nextRoot) return;
    applier = createWalkApplier(nextRoot);
    boneByName = applier.byName;
  }

  function captureRest() {
    if (!boneByName) return;
    rest = captureRestPose(boneByName);
  }

  function restoreRestPose() {
    if (!boneByName || !rest) return;
    restoreAllBones(boneByName, rest);
    root?.updateMatrixWorld(true);
    refreshSkinnedMeshes(root);
  }

  function update(delta) {
    if (!playing || !root || !applier || !rest) return;
    tickAccum += delta;
    if (tickAccum < step) return;
    tickAccum %= step;
    time = (time + step) % meta.duration;
    const frame = Math.floor((time / meta.duration) * meta.frames) % meta.frames;
    const pose = sampleWalkPose(frame, meta.frames);
    applier.applyFrame(frame, rest, pose);
    root.updateMatrixWorld(true);
    refreshSkinnedMeshes(root);
  }

  function setPlaying(on) {
    if (!root) return false;
    if (on) {
      if (!applier) bindRoot(root);
      if (!rest?.size) captureRest();
    }
    playing = on;
    tickAccum = 0;
    if (!on) {
      time = 0;
      restoreRestPose();
    }
    return playing;
  }

  function toggle() {
    return setPlaying(!playing);
  }

  function dispose() {
    playing = false;
    time = 0;
    tickAccum = 0;
    rest = null;
    applier = null;
    boneByName = null;
  }

  function resetRoot(nextRoot) {
    dispose();
    root = nextRoot;
    if (root) bindRoot(root);
  }

  return { update, setPlaying, toggle, isPlaying: () => playing, dispose, resetRoot };
}
