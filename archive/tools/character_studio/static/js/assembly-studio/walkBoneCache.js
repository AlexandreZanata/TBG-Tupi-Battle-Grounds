/** Bone lookup cache — built once per character load (avoid per-frame traverse). */

export function buildBoneCache(root) {
  const byName = new Map();
  root?.traverse((obj) => {
    if (obj.isBone) byName.set(obj.name, obj);
  });
  return byName;
}

export function captureRestPose(byName) {
  const rest = new Map();
  for (const [name, bone] of byName) {
    rest.set(name, {
      q: bone.quaternion.clone(),
      p: bone.position.clone(),
      rot: bone.rotation.clone(),
    });
  }
  return rest;
}

export function restoreAllBones(byName, rest) {
  for (const [name, snap] of rest) {
    const bone = byName.get(name);
    if (!bone) continue;
    bone.quaternion.copy(snap.q);
    bone.position.copy(snap.p);
    bone.rotation.copy(snap.rot);
  }
}
