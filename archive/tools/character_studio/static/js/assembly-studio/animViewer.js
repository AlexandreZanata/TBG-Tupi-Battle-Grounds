/** GLB/FBX preview with baked Mixamo (or other) skeletal clips — no procedural walk. */

import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
import { FBXLoader } from "three/addons/loaders/FBXLoader.js";
import { playClipOnMixer, prepareClipsForPlayback } from "./clipPlayback.js";
import { bindClipsToRoot } from "./clipBind.js";
import { refreshSkinnedMeshes, stabilizeTbgCharacter } from "./tbgPoseFix.js";

function fitCamera(root, camera, controls) {
  root.updateMatrixWorld(true);
  const box = new THREE.Box3().setFromObject(root);
  if (box.isEmpty()) return;
  const size = box.getSize(new THREE.Vector3());
  const center = box.getCenter(new THREE.Vector3());
  const span = Math.max(size.x, size.y, size.z, 0.5);
  const dist = span * 2.8;
  camera.near = Math.max(0.001, span / 500);
  camera.far = Math.max(500, dist * 8);
  camera.updateProjectionMatrix();
  camera.position.set(center.x + dist * 0.55, center.y + span * 0.45, center.z + dist * 0.85);
  controls.target.copy(center);
  controls.update();
}

function disposeObject(obj) {
  obj.traverse((node) => {
    if (node.geometry) node.geometry.dispose();
    if (node.material) {
      const mats = Array.isArray(node.material) ? node.material : [node.material];
      mats.forEach((m) => m.dispose?.());
    }
  });
}

export function mountAnimViewer(container) {
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0xe8e6e1);

  const renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  container.replaceChildren(renderer.domElement);

  const camera = new THREE.PerspectiveCamera(36, 1, 0.001, 5000);
  camera.position.set(1.6, 1.4, 2.2);

  const controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;

  const hemi = new THREE.HemisphereLight(0xffffff, 0x8a8278, 1.1);
  const key = new THREE.DirectionalLight(0xffffff, 1.2);
  key.position.set(2, 4, 3);
  scene.add(hemi, key);

  const gltfLoader = new GLTFLoader();
  const fbxLoader = new FBXLoader();

  let root = null;
  let mixer = null;
  let clips = [];
  let activeAction = null;
  let token = 0;
  let frameId = 0;
  let lastFrameMs = performance.now();

  const resize = () => {
    const w = container.clientWidth || 640;
    const h = container.clientHeight || 420;
    renderer.setSize(w, h, false);
    camera.aspect = w / Math.max(h, 1);
    camera.updateProjectionMatrix();
  };
  const observer = new ResizeObserver(resize);
  observer.observe(container);
  resize();

  const animate = (now) => {
    frameId = requestAnimationFrame(animate);
    const delta = Math.min(0.05, (now - lastFrameMs) / 1000);
    lastFrameMs = now;
    mixer?.update(delta);
    controls.update();
    renderer.render(scene, camera);
  };
  animate();

  function clearRoot() {
    if (mixer) {
      mixer.stopAllAction();
      mixer = null;
    }
    activeAction = null;
    clips = [];
    if (!root) return;
    scene.remove(root);
    disposeObject(root);
    root = null;
  }

  async function loadFromGltf(url) {
    const gltf = await gltfLoader.loadAsync(url);
    return { scene: gltf.scene, animations: gltf.animations ?? [] };
  }

  async function loadFromFbx(url) {
    const fbx = await fbxLoader.loadAsync(url);
    return { scene: fbx, animations: fbx.animations ?? [] };
  }

  async function loadAsset(url, { extraClips = [] } = {}) {
    const current = ++token;
    clearRoot();
    const lower = url.split("?")[0].toLowerCase();
    const loaded = lower.endsWith(".fbx") ? await loadFromFbx(url) : await loadFromGltf(url);
    if (current !== token) return null;

    root = loaded.scene;
    root.traverse((obj) => {
      if (obj.isSkinnedMesh) obj.frustumCulled = false;
    });
    scene.add(root);
    root.updateMatrixWorld(true);
    refreshSkinnedMeshes(root);
    stabilizeTbgCharacter(root);
    root.updateMatrixWorld(true);
    refreshSkinnedMeshes(root);
    fitCamera(root, camera, controls);

    const boundExtra = extraClips.length ? bindClipsToRoot(root, extraClips) : [];
    if (extraClips.length && !boundExtra.length) {
      console.warn("Mixamo Edit: Walking clip had no matching bones on joined model");
    }
    clips = prepareClipsForPlayback([...(loaded.animations ?? []), ...boundExtra]);
    mixer = clips.length ? new THREE.AnimationMixer(root) : null;
    return { clipNames: clips.map((c) => c.name), walkTracks: boundExtra[0]?.tracks?.length ?? 0 };
  }

  function playClip(name, loop = true) {
    if (!mixer || !clips.length) return false;
    const resolved = clips.find((c) => c.name === name) ?? clips[0];
    if (!resolved) return false;
    const { action, played } = playClipOnMixer(mixer, clips, resolved.name, loop, {
      previousAction: activeAction,
    });
    if (!played) return false;
    activeAction = action;
    return true;
  }

  function stopClip() {
    if (!mixer) return;
    mixer.stopAllAction();
    activeAction = null;
  }

  function getClipNames() {
    return clips.map((c) => c.name);
  }

  return {
    loadAsset,
    playClip,
    stopClip,
    getClipNames,
    isPlaying: () => Boolean(activeAction?.isRunning?.()),
    dispose() {
      token += 1;
      cancelAnimationFrame(frameId);
      observer.disconnect();
      clearRoot();
      renderer.dispose();
    },
  };
}
