/** Singleton GLTFLoader — Draco + Meshopt; caches decoded scenes per mesh URL. */

import { DRACOLoader } from "three/addons/loaders/DRACOLoader.js";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
import { MeshoptDecoder } from "three/addons/libs/meshopt_decoder.module.js";
import { MAX_PREVIEW_ASSET_BYTES } from "../config/previewLimits.js";
import { cloneGltfEntry, getCachedGltf, storeGltfScene } from "./gltfCache.js";

const DRACO_DECODER_PATH = "https://www.gstatic.com/draco/versioned/decoders/1.5.7/";

let loaderPromise = null;
const inflight = new Map();

export async function loadGltfScene(url) {
  const cached = getCachedGltf(url);
  if (cached) {
    return cloneGltfEntry(cached.scene, cached.animations);
  }

  if (inflight.has(url)) {
    const entry = await inflight.get(url);
    return cloneGltfEntry(entry.scene, entry.animations);
  }

  const loadPromise = (async () => {
    const loader = await getLoader();
    const gltf = await loader.loadAsync(url);
    const entry = { scene: gltf.scene, animations: gltf.animations ?? [] };
    storeGltfScene(url, entry.scene, entry.animations);
    return entry;
  })();

  inflight.set(url, loadPromise);
  try {
    const entry = await loadPromise;
    return cloneGltfEntry(entry.scene, entry.animations);
  } finally {
    inflight.delete(url);
  }
}

async function getLoader() {
  loaderPromise ??= createLoader();
  return loaderPromise;
}

async function createLoader() {
  const dracoLoader = new DRACOLoader();
  dracoLoader.setDecoderPath(DRACO_DECODER_PATH);

  const loader = new GLTFLoader();
  loader.setDRACOLoader(dracoLoader);

  await MeshoptDecoder.ready;
  loader.setMeshoptDecoder(MeshoptDecoder);

  return loader;
}

export async function probeAsset(url, maxBytes = MAX_PREVIEW_ASSET_BYTES) {
  const infoUrl = toMeshInfoUrl(url);
  const response = await fetch(infoUrl);
  if (!response.ok) {
    return { ok: false, reason: "missing" };
  }
  const data = await response.json();
  const length = Number(data.size || 0);
  if (length > maxBytes) {
    return { ok: false, reason: "too_large" };
  }
  return { ok: true, size: length, cached: Boolean(getCachedGltf(url)) };
}

/** Map `/mesh` asset URL to `/mesh-info`, preserving query string (e.g. tier=showcase). */
export function toMeshInfoUrl(meshUrl) {
  const [path, query = ""] = meshUrl.split("?");
  const infoPath = path.replace(/\/mesh$/, "/mesh-info");
  return query ? `${infoPath}?${query}` : infoPath;
}
