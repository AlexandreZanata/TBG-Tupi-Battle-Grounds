/** Map behavior profile actions to viewer clip names. */

const FALLBACK = { idle: "Idle", attack: "Attack", death: "Death", spawn: "Idle" };

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

export async function playBehaviorAction(viewer, character, action) {
  const profile = await fetchBehaviorProfile(character.behaviorProfile);
  const primary = clipNameForAction(profile, action);
  if (viewer.playClip(primary, action === "idle")) return true;
  const fallback = FALLBACK[action];
  if (fallback && fallback !== primary) {
    return viewer.playClip(fallback, action === "idle");
  }
  return false;
}
