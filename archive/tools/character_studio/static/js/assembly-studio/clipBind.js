/** Rewrite animation tracks to full bone paths under a loaded scene root. */

export function buildBonePathMap(root) {
  const map = new Map();
  root.traverse((obj) => {
    if (!obj.isBone) return;
    const parts = [];
    let cur = obj;
    while (cur && cur !== root) {
      parts.unshift(cur.name);
      cur = cur.parent;
    }
    map.set(obj.name, parts.join("/"));
  });
  return map;
}

export function bindClipsToRoot(root, clips) {
  const paths = buildBonePathMap(root);
  return clips
    .map((clip) => {
      const next = clip.clone();
      next.tracks = [];
      for (const track of clip.tracks) {
        const dot = track.name.indexOf(".");
        if (dot < 0) continue;
        const boneKey = track.name.slice(0, dot).split("/").pop();
        const rel = paths.get(boneKey);
        if (!rel) continue;
        const renamed = track.clone();
        renamed.name = `${rel}${track.name.slice(dot)}`;
        next.tracks.push(renamed);
      }
      return next;
    })
    .filter((c) => c.tracks.length > 0);
}
