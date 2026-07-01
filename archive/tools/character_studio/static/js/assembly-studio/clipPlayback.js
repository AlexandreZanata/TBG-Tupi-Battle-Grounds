/** Skeletal clip playback — crossfade + loop (Three.js AnimationMixer). */

import * as THREE from "three";

const CROSSFADE_SEC = 0.22;

export function prepareClipsForPlayback(clips) {
  return clips.map((clip) => {
    const next = clip.clone();
    next.optimize();
    return next;
  });
}

export function playClipOnMixer(mixer, clips, name, loop, { previousAction = null } = {}) {
  const resolved = clips.find((c) => c.name === name);
  if (!mixer || !resolved) return { action: null, played: false };

  const next = mixer.clipAction(resolved);
  next.reset();
  next.setLoop(loop ? THREE.LoopRepeat : THREE.LoopOnce, loop ? Infinity : 1);
  next.clampWhenFinished = !loop;
  next.setEffectiveWeight(1);
  next.play();

  if (previousAction && previousAction !== next) {
    previousAction.crossFadeTo(next, CROSSFADE_SEC, false);
  } else {
    next.fadeIn(CROSSFADE_SEC);
  }

  mixer.update(0);
  return { action: next, played: true };
}
