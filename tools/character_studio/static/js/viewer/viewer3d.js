/**
 * Character Studio 3D viewer — GLTFLoader + studio lighting + procedural fallback animations.
 */

import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { loadGltfScene, probeAsset } from "./gltfLoader.js";
import { buildProceduralCharacter } from "./proceduralMesh.js";
import { buildProceduralClips, resolveClipName } from "./proceduralClips.js";
import { applyCharacterMaterials, normalizeMeshes } from "./viewerMaterials.js";
import { applyStudioLighting, configureRenderer } from "./viewerLighting.js";
import { bindViewerResize, prepareViewerRoot, readViewerSize, applyRendererSize } from "./viewerResize.js";

const FIT_PADDING = 1.28;

export function mountCharacterViewer(container, options = {}) {
  const mount = prepareViewerRoot(container);
  const scene = new THREE.Scene();
  applyStudioLighting(scene, options.stageVariant ?? "dark");

  const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: false });
  configureRenderer(renderer);
  mount.appendChild(renderer.domElement);

  const { width, height } = readViewerSize(container);
  applyRendererSize(renderer, width, height, options.maxPixelRatio ?? 2);

  const camera = new THREE.PerspectiveCamera(36, width / Math.max(height, 1), 0.01, 200);
  const controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;
  controls.minDistance = 0.5;
  controls.maxDistance = 12;
  controls.target.set(0, 0.9, 0);

  let modelRoot = null;
  let mixer = null;
  let animationClips = [];
  let activeAction = null;
  let autoRotate = false;
  let cancelled = false;
  let loadToken = 0;
  let lastFrameMs = performance.now();
  let frameId = 0;
  let viewportW = width;
  let viewportH = height;

  const resizeObserver = bindViewerResize(container, renderer, camera, (w, h) => {
    viewportW = w;
    viewportH = h;
    if (modelRoot) fitCamera(modelRoot, camera, w, h);
  });

  const animate = (now) => {
    frameId = requestAnimationFrame(animate);
    const delta = (now - lastFrameMs) / 1000;
    lastFrameMs = now;
    if (autoRotate && modelRoot) modelRoot.rotation.y += 0.28 * delta;
    if (mixer) mixer.update(delta);
    controls.update();
    renderer.render(scene, camera);
  };
  requestAnimationFrame(animate);

  function removeModelRoot() {
    if (mixer) {
      mixer.stopAllAction();
      mixer = null;
    }
    activeAction = null;
    animationClips = [];
    if (!modelRoot) return;
    scene.remove(modelRoot);
    disposeObject(modelRoot);
    modelRoot = null;
  }

  async function loadCharacter(character) {
    const token = ++loadToken;
    const keepPrevious = Boolean(modelRoot);
    if (!keepPrevious) {
      removeModelRoot();
    }
    options.onLoading?.(character, { cached: false });
    try {
      const built = await buildCharacterRoot(character, token);
      if (token !== loadToken || cancelled || !built) return;
      if (keepPrevious) removeModelRoot();
      modelRoot = built.root;
      scene.add(modelRoot);
      fitCamera(modelRoot, camera, viewportW, viewportH);
      animationClips = built.animationClips;
      mixer = new THREE.AnimationMixer(modelRoot);
      playClip("Idle", true);
      options.onReady?.(character);
    } catch (error) {
      if (token !== loadToken || cancelled) return;
      options.onError?.(error);
    }
  }

  async function buildCharacterRoot(character, token) {
    if (character.assetMode === "mesh3d" && character.id) {
      return buildMesh3dRoot(character, token);
    }
    return buildProceduralRoot(character, token);
  }

  async function buildMesh3dRoot(character, token) {
    const meshUrl = `/api/characters/${character.id}/mesh`;
    const probe = await probeAsset(meshUrl);
    if (!probe.ok) {
      throw new Error(probe.reason === "too_large" ? "Mesh exceeds 150 MB preview limit" : "Mesh not found");
    }
    options.onLoading?.(character, { cached: probe.cached });
    if (!probe.cached) {
      options.onProgress?.(`Decoding mesh (${((probe.size || 0) / (1024 * 1024)).toFixed(0)} MB)…`);
    }

    const { scene: gltfScene, animations } = await loadGltfScene(meshUrl);
    if (token !== loadToken) return null;

    normalizeMeshes(gltfScene);
    await applyCharacterMaterials(gltfScene, character);
    if (token !== loadToken) return null;

    const display = options.displayScale ?? 1;
    gltfScene.scale.set(
      (character.bodyWidthScale ?? 1) * display,
      (character.heightScale ?? 1) * display,
      (character.bodyWidthScale ?? 1) * display,
    );
    placeOnGround(gltfScene);
    return {
      root: gltfScene,
      animationClips: animations.length > 0 ? animations : buildProceduralClips(),
    };
  }

  async function buildProceduralRoot(character, token) {
    const object = buildProceduralCharacter(character);
    if (token !== loadToken) return null;
    placeOnGround(object);
    return { root: object, animationClips: buildProceduralClips() };
  }

  function playClip(name, loop = false) {
    if (!mixer) return false;
    const resolved = resolveClipName(name);
    const clip = animationClips.find((c) => c.name === resolved) ?? animationClips.find((c) => c.name === name);
    if (!clip) return false;
    if (activeAction) activeAction.fadeOut(0.15);
    activeAction = mixer.clipAction(clip);
    activeAction.reset().setLoop(loop ? THREE.LoopRepeat : THREE.LoopOnce, loop ? Infinity : 1);
    activeAction.clampWhenFinished = !loop;
    activeAction.fadeIn(0.12).play();
    return true;
  }

  return {
    loadCharacter,
    setAutoRotate(value) {
      autoRotate = value;
    },
    playClip,
    dispose() {
      cancelled = true;
      cancelAnimationFrame(frameId);
      resizeObserver.disconnect();
      removeModelRoot();
      renderer.dispose();
      mount.remove();
    },
  };
}

function placeOnGround(object) {
  const box = new THREE.Box3().setFromObject(object);
  const center = box.getCenter(new THREE.Vector3());
  object.position.x -= center.x;
  object.position.z -= center.z;
  object.position.y -= box.min.y;
}

function fitCamera(object, camera, width, height) {
  const sphere = new THREE.Box3().setFromObject(object).getBoundingSphere(new THREE.Sphere());
  const radius = Math.max(sphere.radius, 0.001);
  const fovRad = (camera.fov * Math.PI) / 180;
  const aspect = width / Math.max(height, 1);
  const hFov = 2 * Math.atan(Math.tan(fovRad / 2) * aspect);
  const distY = radius / Math.sin(fovRad / 2);
  const distX = radius / Math.sin(hFov / 2);
  const distance = Math.max(distX, distY) * FIT_PADDING;
  camera.position.set(distance * 0.55, distance * 0.42, distance * 0.92);
  camera.lookAt(0, radius * 0.42, 0);
  camera.near = Math.max(distance / 200, 0.001);
  camera.far = distance * 24;
  camera.updateProjectionMatrix();
}

function disposeObject(object) {
  object.traverse((child) => {
    if (!(child instanceof THREE.Mesh)) return;
    child.geometry?.dispose();
    const mats = Array.isArray(child.material) ? child.material : [child.material];
    mats.forEach((m) => m?.dispose?.());
  });
}
