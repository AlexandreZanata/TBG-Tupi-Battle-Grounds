/** Clip playback buttons and loop toggle for Animation Studio. */

import { playBehaviorAction, shouldLoopAction } from "../studio/behaviorPlayback.js";

const CLIP_ACTIONS = ["idle", "walk", "attack", "attack2", "ultimate", "death"];

export function createClipControls(container, getContext) {
  container.innerHTML = `
    <div class="clip-controls" role="group" aria-label="Animation clips">
      ${CLIP_ACTIONS.map(
        (action) =>
          `<button type="button" class="btn btn--ghost btn--sm clip-controls__btn" data-action="${action}">${labelFor(action)}</button>`,
      ).join("")}
    </div>
    <label class="viewer3d-controls__toggle">
      <input type="checkbox" id="clipLoopToggle" checked />
      Loop idle / walk
    </label>
    <label class="viewer3d-controls__toggle">
      <input type="checkbox" id="walkRootMotionToggle" />
      Root motion walk (WalkRoot)
    </label>
  `;

  let activeAction = "idle";
  const walkRootToggle = container.querySelector("#walkRootMotionToggle");

  container.querySelectorAll("[data-action]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const { viewer, character } = getContext();
      if (!viewer || !character) return;
      activeAction = btn.dataset.action;
      highlightActive(container, activeAction);
      await playBehaviorAction(viewer, character, activeAction, {
        walkRootMotion: walkRootToggle?.checked ?? false,
      });
    });
  });

  return {
    setActive(action) {
      activeAction = action;
      highlightActive(container, action);
    },
    getActive: () => activeAction,
    shouldLoop: (action) => shouldLoopAction(action),
    useWalkRootMotion: () => walkRootToggle?.checked ?? false,
  };
}

function labelFor(action) {
  const labels = {
    idle: "Idle",
    walk: "Walk",
    attack: "Attack",
    attack2: "Attack2",
    ultimate: "Ultimate",
    death: "Death",
  };
  return labels[action] ?? action;
}

function highlightActive(container, action) {
  container.querySelectorAll("[data-action]").forEach((btn) => {
    btn.classList.toggle("is-active", btn.dataset.action === action);
  });
}
