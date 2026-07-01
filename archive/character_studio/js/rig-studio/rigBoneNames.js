/** glTF node names drop dots before L/R; Blender uses TBG_Shoulder.L. */

export function toBlenderBoneName(name) {
  if (!name) return name;
  if (name.includes(".L") || name.includes(".R")) return name;
  const m = name.match(/^(TBG_.+)(L|R)$/);
  return m ? `${m[1]}.${m[2]}` : name;
}

export function toGltfBoneName(name) {
  if (!name) return name;
  return name.replace(/\.(L|R)$/, "$1");
}
