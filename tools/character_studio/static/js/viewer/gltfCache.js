/** In-memory GLB cache — decode once per URL, clone for each viewer instance. */

import * as THREE from "three";
import * as SkeletonUtils from "three/addons/utils/SkeletonUtils.js";

const sceneCache = new Map();

export function getGltfCacheSize() {
  return sceneCache.size;
}

export function storeGltfScene(url, scene, animations = []) {
  sceneCache.set(url, { scene, animations });
}

export function getCachedGltf(url) {
  return sceneCache.get(url) ?? null;
}

export function cloneGltfEntry(scene, animations = []) {
  const clonedScene = SkeletonUtils.clone(scene);
  clonedScene.traverse((obj) => {
    if (!(obj instanceof THREE.Mesh) || !obj.material) return;
    obj.material = Array.isArray(obj.material)
      ? obj.material.map((mat) => mat.clone())
      : obj.material.clone();
  });
  return {
    scene: clonedScene,
    animations: animations.map((clip) => clip.clone()),
  };
}
