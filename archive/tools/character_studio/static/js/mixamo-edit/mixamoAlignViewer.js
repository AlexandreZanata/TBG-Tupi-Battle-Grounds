/** Mixamo align viewport — semi-transparent skin + draggable Mixamo rig inside. */

import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { TransformControls } from "three/addons/controls/TransformControls.js";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
import { FBXLoader } from "three/addons/loaders/FBXLoader.js";
import { createBoneProportions } from "./mixamoBoneProportions.js";
import {
  brightenMeshes,
  enforceUniformScale,
  fitCamera,
  normalizeHeight,
  objScaleUniform,
  placeOnGround,
  worldBox,
} from "./mixamoViewUtils.js";

export function mountMixamoAlignViewer(container) {
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0x5c5a62);

  const renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  container.replaceChildren(renderer.domElement);

  const camera = new THREE.PerspectiveCamera(45, 1, 0.001, 5000);
  camera.position.set(0.8, 1.2, 3.2);
  const controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;
  controls.target.set(0, 0.9, 0);

  scene.add(new THREE.AmbientLight(0xffffff, 1.35));
  scene.add(new THREE.HemisphereLight(0xffffff, 0x6a6458, 1.1));
  const key = new THREE.DirectionalLight(0xffffff, 1.6);
  key.position.set(4, 8, 5);
  scene.add(key);
  scene.add(new THREE.GridHelper(6, 24, 0x888888, 0x444444));

  const stage = new THREE.Group();
  const skinRoot = new THREE.Group();
  const rigPivot = new THREE.Group();
  stage.add(skinRoot, rigPivot);
  scene.add(stage);

  let baseRigScale = 1;
  let sizeMultiplier = 1;
  const initialRigPos = new THREE.Vector3();
  let skinOpacity = 0.38;
  let hasPair = false;
  let onTransformChange = null;
  let boneProps = null;
  let frameId = 0;

  function clampFeetOnGround() {
    const box = worldBox(rigPivot);
    if (!box.isEmpty() && box.min.y < 0) rigPivot.position.y -= box.min.y;
  }

  function applySkeletonSize(mult) {
    sizeMultiplier = Math.max(0.25, Math.min(3, mult));
    const feetBefore = worldBox(rigPivot).min.y;
    objScaleUniform(rigPivot, baseRigScale * sizeMultiplier);
    rigPivot.position.y += feetBefore - worldBox(rigPivot).min.y;
    clampFeetOnGround();
    onTransformChange?.();
  }

  function notifyTransform() {
    enforceUniformScale(rigPivot);
    clampFeetOnGround();
    onTransformChange?.();
  }

  const transform = new TransformControls(camera, renderer.domElement);
  transform.setMode("translate");
  transform.showX = transform.showY = transform.showZ = true;
  const transformHelper = transform.getHelper();
  transform.addEventListener("dragging-changed", (ev) => {
    controls.enabled = !ev.value;
  });
  transform.addEventListener("objectChange", notifyTransform);
  scene.add(transformHelper);

  const resize = () => {
    const w = container.clientWidth || 640;
    const h = container.clientHeight || 420;
    renderer.setSize(w, h, false);
    camera.aspect = w / Math.max(h, 1);
    camera.updateProjectionMatrix();
  };
  new ResizeObserver(resize).observe(container);
  resize();

  const animate = () => {
    frameId = requestAnimationFrame(animate);
    controls.update();
    renderer.render(scene, camera);
  };
  animate();

  function clearPair() {
    transform.detach();
    skinRoot.clear();
    rigPivot.clear();
    rigPivot.position.set(0, 0, 0);
    rigPivot.scale.set(1, 1, 1);
    baseRigScale = 1;
    sizeMultiplier = 1;
    boneProps = null;
    hasPair = false;
  }

  async function loadAlignPair({ mixamoUrl, skinUrl }) {
    clearPair();
    skinRoot.add((await new GLTFLoader().loadAsync(skinUrl)).scene);
    normalizeHeight(skinRoot);
    placeOnGround(skinRoot);
    brightenMeshes(skinRoot, { skin: true, opacity: skinOpacity });

    const rig = await new FBXLoader().loadAsync(mixamoUrl);
    enforceUniformScale(rig);
    rigPivot.add(rig);
    normalizeHeight(rigPivot);
    placeOnGround(rigPivot);
    brightenMeshes(rig, { skin: false, opacity: 1 });
    boneProps = createBoneProportions(rig);
    if (boneProps.boneCount < 5) console.warn("Mixamo Edit: few bones found — proportion sliders may not work");

    baseRigScale = enforceUniformScale(rigPivot);
    initialRigPos.copy(rigPivot.position);
    transform.attach(rigPivot);
    fitCamera(stage, camera, controls);
    hasPair = true;
    return { clipNames: rig.animations?.map((c) => c.name) ?? [], skeletonSize: 1 };
  }

  return {
    loadAlignPair,
    setSkinOpacity(v) {
      skinOpacity = Math.max(0.08, Math.min(1, v));
      brightenMeshes(skinRoot, { skin: true, opacity: skinOpacity });
    },
    setTransformMode: () => transform.setMode("translate"),
    getAlignment: () => ({
      rigPosition: rigPivot.position.toArray(),
      rigScale: sizeMultiplier,
      skeletonSize: sizeMultiplier,
      boneProportions: boneProps?.get() ?? { hipWidth: 1, shoulderWidth: 1, torsoDepth: 1 },
    }),
    setRigPosition(x, y, z) {
      rigPivot.position.set(x, y, z);
      notifyTransform();
    },
    setSkeletonSize: applySkeletonSize,
    setBoneProportions(props) {
      boneProps?.apply(props);
      clampFeetOnGround();
      onTransformChange?.();
    },
    resetRigTransform() {
      rigPivot.position.copy(initialRigPos);
      applySkeletonSize(1);
      boneProps?.reset();
    },
    setOnTransformChange(fn) {
      onTransformChange = fn;
    },
    hasPair: () => hasPair,
    dispose() {
      cancelAnimationFrame(frameId);
      transform.detach();
      scene.remove(transformHelper);
      transform.dispose();
      clearPair();
      container.replaceChildren();
      renderer.dispose();
    },
  };
}
