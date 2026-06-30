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
import { placeOnGround, fitCamera, disposeObject } from "./viewerSceneUtils.js";

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
    const tier = character.meshTier === "game" ? "game" : "showcase";
    const meshUrl = `/api/characters/${character.id}/mesh?tier=${tier}`;
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
    const embedded = animations.length > 0;
    const allowProcedural = tier !== "game";
    return {
      root: gltfScene,
      animationClips: embedded ? animations : allowProcedural ? buildProceduralClips() : [],
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

  function getClipProgress() {
    if (!activeAction) return { time: 0, duration: 0, clipName: null };
    const clip = activeAction.getClip();
    return { time: activeAction.time, duration: clip.duration, clipName: clip.name };
  }

  function seekClip(time) {
    if (!activeAction || !mixer) return;
    activeAction.time = Math.max(0, Math.min(time, activeAction.getClip().duration));
    mixer.update(0);
  }

  function listClipNames() {
    return animationClips.map((clip) => clip.name);
  }

  return {
    loadCharacter,
    setAutoRotate(value) {
      autoRotate = value;
    },
    playClip,
    getClipProgress,
    seekClip,
    listClipNames,
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
