import { readFileSync } from "node:fs";
import { fileURLToPath, pathToFileURL } from "node:url";
import { dirname, join } from "node:path";
import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import { FBXLoader } from "three/examples/jsm/loaders/FBXLoader.js";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "../../..");
const glbPath = join(ROOT, "content/characters/imports/mixamo/edit/ybot_skin_01.glb");
const fbxPath = join(ROOT, "content/characters/imports/mixamo/raw/Walking.fbx");
const glbBuf = readFileSync(glbPath);
const glbData = glbBuf.buffer.slice(glbBuf.byteOffset, glbBuf.byteOffset + glbBuf.byteLength);
const fbxBuf = readFileSync(fbxPath);
const mapPath = join(ROOT, "tools/character_studio/static/data/mixamo-to-tbg-bones.json");

function normalizeMixamoBone(name) {
  const base = String(name).split("/").pop() || name;
  return base.replace(/^mixamorig\d*[:_]?/i, "");
}

function buildLookup(boneMap) {
  const lookup = new Map();
  for (const [tbg, canonical] of Object.entries(boneMap.mapping || {})) {
    lookup.set(canonical, tbg);
    lookup.set(normalizeMixamoBone(canonical), tbg);
    for (const alias of boneMap.name_aliases?.[canonical] || []) {
      lookup.set(alias, tbg);
      lookup.set(normalizeMixamoBone(alias), tbg);
    }
  }
  return lookup;
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

function buildBonePathMap(root) {
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

function bindClipsToRoot(root, clips) {
  const paths = buildBonePathMap(root);
  return clips.map((clip) => {
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
  }).filter((c) => c.tracks.length > 0);
}

function sampleBoneDelta(root, bonePath, t0, t1) {
  const mixer0 = new THREE.AnimationMixer(root);
  const mixer1 = new THREE.AnimationMixer(root);
  return { mixer0, mixer1 };
}

const boneMap = JSON.parse(readFileSync(mapPath, "utf8"));
const lookup = buildLookup(boneMap);

const gltf = await new GLTFLoader().parseAsync(glbData, glbPath);
const walkFbx = new FBXLoader().parse(fbxBuf, fbxPath);
const root = gltf.scene;

console.log("root name:", root.name, "children:", root.children.map((c) => c.name));

const paths = buildBonePathMap(root);
console.log("TBG bones found:", [...paths.keys()].filter((k) => k.startsWith("TBG")).length);
console.log("sample path TBG_Spine:", paths.get("TBG_Spine"));
console.log("sample path TBG_UpperLeg.L:", paths.get("TBG_UpperLeg.L"));

const walkRaw = walkFbx.animations[0];
console.log("\nWalking raw:", walkRaw?.name, "tracks", walkRaw?.tracks.length);
console.log("sample raw tracks:", walkRaw?.tracks.slice(0, 5).map((t) => t.name));

const retargeted = retargetClip(walkRaw, lookup);
console.log("\nRetargeted tracks:", retargeted.tracks.length);
console.log("sample retarget:", retargeted.tracks.slice(0, 5).map((t) => t.name));

const bound = bindClipsToRoot(root, [retargeted]);
console.log("\nBound tracks:", bound[0]?.tracks.length);
console.log("sample bound:", bound[0]?.tracks.slice(0, 5).map((t) => t.name));

// Test PropertyBinding
let bindOk = 0;
let bindFail = 0;
for (const track of bound[0]?.tracks || []) {
  const pb = new THREE.PropertyBinding(root, track.name);
  if (pb.node) bindOk++;
  else bindFail++;
}
console.log("\nPropertyBinding ok/fail:", bindOk, bindFail);

// Simulate animation - sample quaternion change on leg bone
const clip = bound[0];
clip.name = "Walking";
const mixer = new THREE.AnimationMixer(root);
const action = mixer.clipAction(clip);
action.play();

const legPath = paths.get("TBG_UpperLeg.L");
const legBone = root.getObjectByName("TBG_UpperLeg.L");
const q0 = legBone?.quaternion.clone();

mixer.update(0.5);
const q1 = legBone?.quaternion.clone();

console.log("\nLeg bone quat changed after 0.5s:", q0 && q1 ? !q0.equals(q1) : "bone not found");
if (q0 && q1) {
  console.log(" q0:", q0.toArray().map((v) => v.toFixed(3)));
  console.log(" q1:", q1.toArray().map((v) => v.toFixed(3)));
}

// Compare embedded GLB clip track format
const embedded = gltf.animations[0];
console.log("\nEmbedded GLB clip:", embedded?.name, "tracks", embedded?.tracks.length);
console.log("sample embedded tracks:", embedded?.tracks.slice(0, 5).map((t) => t.name));

// Test embedded clip binding
let embOk = 0;
for (const track of embedded?.tracks || []) {
  const pb = new THREE.PropertyBinding(root, track.name);
  if (pb.node) embOk++;
}
console.log("Embedded PropertyBinding ok:", embOk, "/", embedded?.tracks.length);
