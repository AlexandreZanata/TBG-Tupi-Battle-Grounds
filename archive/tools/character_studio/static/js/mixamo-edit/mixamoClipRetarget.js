/** Retarget Mixamo FBX animation clips → TBG bone names on joined GLB. */

const MAP_URL = "/data/mixamo-to-tbg-bones.json";
let cachedMap = null;

function normalizeMixamoBone(name) {
  const base = String(name).split("/").pop() || name;
  return base.replace(/^mixamorig\d*[:_]?/i, "");
}

export function buildMixamoToTbgLookup(boneMap) {
  const lookup = new Map();
  const aliases = boneMap.name_aliases || {};
  for (const [tbg, canonical] of Object.entries(boneMap.mapping || {})) {
    lookup.set(canonical, tbg);
    lookup.set(normalizeMixamoBone(canonical), tbg);
    for (const alias of aliases[canonical] || []) {
      lookup.set(alias, tbg);
      lookup.set(normalizeMixamoBone(alias), tbg);
    }
  }
  return lookup;
}

export async function loadMixamoBoneMap() {
  if (cachedMap) return cachedMap;
  const res = await fetch(MAP_URL);
  if (!res.ok) throw new Error("Mixamo bone map not found");
  cachedMap = await res.json();
  return cachedMap;
}

export function retargetMixamoClips(clips, boneMap) {
  const lookup = buildMixamoToTbgLookup(boneMap);
  return clips
    .map((clip) => {
      const next = retargetClip(clip, lookup);
      if (next.tracks.length) next.name = "Walking";
      return next;
    })
    .filter((c) => c.tracks.length > 0);
}

function retargetClip(clip, lookup) {
  const next = clip.clone();
  next.tracks = [];
  for (const track of clip.tracks) {
    const dot = track.name.indexOf(".");
    if (dot < 0) continue;
    const boneName = track.name.slice(0, dot);
    const prop = track.name.slice(dot);
    const tbg = lookup.get(boneName) || lookup.get(normalizeMixamoBone(boneName));
    if (!tbg) continue;
    const renamed = track.clone();
    renamed.name = `${tbg}${prop}`;
    next.tracks.push(renamed);
  }
  return next;
}
