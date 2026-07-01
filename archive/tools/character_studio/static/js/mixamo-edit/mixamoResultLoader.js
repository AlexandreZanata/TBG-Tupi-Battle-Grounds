/** Load retargeted Walking (or other Mixamo) clips for joined TBG GLB preview. */

import { FBXLoader } from "three/addons/loaders/FBXLoader.js";
import { loadMixamoBoneMap, retargetMixamoClips } from "./mixamoClipRetarget.js";

export async function loadMixamoAnimClips(fbxUrl) {
  const fbx = await new FBXLoader().loadAsync(fbxUrl);
  const boneMap = await loadMixamoBoneMap();
  return retargetMixamoClips(fbx.animations ?? [], boneMap);
}

export function pickWalkClipName(names) {
  return names.find((n) => n === "Walking") || names.find((n) => /walk/i.test(n)) || null;
}

export async function previewJoinedWithWalking(viewer, project, walkFbxUrl) {
  let extraClips = [];
  try {
    extraClips = await loadMixamoAnimClips(walkFbxUrl);
  } catch (err) {
    console.warn("Walking.fbx test clip unavailable", err);
  }
  const info = await viewer.loadAsset(`${project.meshUrl}?t=${Date.now()}`, { extraClips });
  if (!info) throw new Error("Model preview was interrupted — try again");
  const boundWalk = extraClips.some((clip) => clip.tracks.length > 0);
  const names = info?.clipNames?.length ? info.clipNames : project.clipNames || [];
  const pick = pickWalkClipName(names);
  if (pick && (pick === "Walking" || boundWalk)) viewer.playClip(pick, true);
  else viewer.stopClip?.();
  return { names, pick: pick && (pick === "Walking" || boundWalk) ? pick : null };
}
