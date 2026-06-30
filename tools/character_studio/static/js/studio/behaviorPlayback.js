/** Map behavior profile actions to viewer clip names. */

const FALLBACK = {
  idle: "Idle",
  walk: "Walk",
  attack: "Attack",
  attack2: "Attack2",
  death: "Death",
  spawn: "Spawn",
  ultimate: "Ultimate",
};

const LOOP_ACTIONS = new Set(["idle", "walk"]);

export async function fetchBehaviorProfile(behaviorProfileId) {
  if (!behaviorProfileId) return null;
  const url = `/api/content/characters/behaviors/${behaviorProfileId}.json`;
  const response = await fetch(url);
  if (!response.ok) return null;
  return response.json();
}

export function clipNameForAction(profile, action) {
  if (profile?.actions?.[action]) return profile.actions[action];
  return FALLBACK[action] ?? "Idle";
}

export function shouldLoopAction(action) {
  return LOOP_ACTIONS.has(action);
}

export async function playBehaviorAction(viewer, character, action) {
  const profile = await fetchBehaviorProfile(character.behaviorProfile);
  const primary = clipNameForAction(profile, action);
  const loop = shouldLoopAction(action);
  if (viewer.playClip(primary, loop)) return true;
  const fallback = FALLBACK[action];
  if (fallback && fallback !== primary) {
    return viewer.playClip(fallback, loop);
  }
  return false;
}
