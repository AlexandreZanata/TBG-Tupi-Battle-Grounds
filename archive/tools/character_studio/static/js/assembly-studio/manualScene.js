/** Manual assembly — Three.js scene with skeleton + transform controls. */

import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { TransformControls } from "three/addons/controls/TransformControls.js";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
import { stabilizeTbgCharacter } from "./tbgPoseFix.js";

export function mountManualScene(container) {
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0x2a2d32);

  const renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  container.replaceChildren(renderer.domElement);

  const camera = new THREE.PerspectiveCamera(40, 1, 0.05, 100);
  camera.position.set(1.8, 1.5, 2.4);

  const orbit = new OrbitControls(camera, renderer.domElement);
  orbit.target.set(0, 0.95, 0);
  orbit.enableDamping = true;
  orbit.mouseButtons = { LEFT: THREE.MOUSE.ROTATE, MIDDLE: THREE.MOUSE.DOLLY, RIGHT: THREE.MOUSE.PAN };
  renderer.domElement.style.touchAction = "none";

  scene.add(new THREE.HemisphereLight(0xffffff, 0x444444, 1.2));
  const key = new THREE.DirectionalLight(0xffffff, 1.4);
  key.position.set(3, 5, 2);
  scene.add(key);
  scene.add(new THREE.GridHelper(4, 20, 0x555555, 0x333333));

  const loader = new GLTFLoader();
  let root = null;
  let skeletonHelper = null;
  let skinnedMesh = null;
  const bones = new Map();
  const attachments = new Map();
  let selected = null;

  const transform = new TransformControls(camera, renderer.domElement);
  transform.setSize(0.85);
  transform.addEventListener("dragging-changed", (e) => {
    orbit.enabled = !e.value;
  });
  scene.add(transform.getHelper());

  let frameId = 0;
  const resize = () => {
    const w = container.clientWidth || 800;
    const h = container.clientHeight || 500;
    renderer.setSize(w, h, false);
    camera.aspect = w / Math.max(h, 1);
    camera.updateProjectionMatrix();
  };
  new ResizeObserver(resize).observe(container);
  resize();

  const animate = () => {
    frameId = requestAnimationFrame(animate);
    orbit.update();
    renderer.render(scene, camera);
  };
  animate();

  function clearScene() {
    transform.detach();
    selected = null;
    attachments.clear();
    bones.clear();
    if (skeletonHelper) scene.remove(skeletonHelper);
    skeletonHelper = null;
    skinnedMesh = null;
    if (root) {
      scene.remove(root);
      root.traverse((o) => {
        if (o.geometry) o.geometry.dispose();
      });
      root = null;
    }
  }

  async function loadBase(url) {
    clearScene();
    const gltf = await loader.loadAsync(url);
    root = gltf.scene;
    scene.add(root);
    root.traverse((obj) => {
      if (obj.isBone) bones.set(obj.name, obj);
      if (obj.isSkinnedMesh) {
        if (!skinnedMesh) skinnedMesh = obj;
        obj.frustumCulled = false;
        if (obj.material) {
          const mats = Array.isArray(obj.material) ? obj.material : [obj.material];
          mats.forEach((m) => {
            m.side = THREE.DoubleSide;
          });
        }
      }
    });
    stabilizeTbgCharacter(root);
    if (skinnedMesh) {
      skeletonHelper = new THREE.SkeletonHelper(skinnedMesh);
      skeletonHelper.visible = true;
      scene.add(skeletonHelper);
    }
    fitCamera();
    return listBones();
  }

  function listBones() {
    return [...bones.keys()].filter((n) => n.startsWith("TBG_")).sort();
  }

  function fitCamera() {
    if (!root) return;
    const box = new THREE.Box3().setFromObject(root);
    const c = box.getCenter(new THREE.Vector3());
    const s = box.getSize(new THREE.Vector3());
    const d = Math.max(s.x, s.y, s.z) * 2;
    camera.position.set(c.x + d * 0.55, c.y + s.y * 0.35, c.z + d);
    orbit.target.copy(c);
  }

  async function attachPart({ partId, meshUrl, socket }) {
    const bone = bones.get(socket);
    if (!bone) throw new Error(`bone not found: ${socket}`);
    const gltf = await loader.loadAsync(meshUrl);
    const pivot = new THREE.Group();
    pivot.name = `attach_${partId}`;
    pivot.userData = { partId, socket, kind: "attachment" };
    const mesh = gltf.scene;
    mesh.traverse((c) => {
      if (c.isMesh) c.userData.kind = "part";
    });
    pivot.add(mesh);
    bone.add(pivot);
    attachments.set(partId, pivot);
    selectAttachment(partId);
    return pivot;
  }

  function selectAttachment(partId) {
    const obj = attachments.get(partId);
    if (!obj) return;
    selected = obj;
    transform.attach(obj);
  }

  function selectBone(name) {
    const bone = bones.get(name);
    if (!bone) return;
    transform.detach();
    const pos = new THREE.Vector3();
    bone.getWorldPosition(pos);
    orbit.target.copy(pos);
    orbit.update();
  }

  function setMode(mode) {
    transform.setMode(mode);
  }

  function setSkeletonVisible(on) {
    if (skeletonHelper) skeletonHelper.visible = on;
  }

  function applyMaterialProfile(profile) {
    if (!root) return;
    const bodyColor = new THREE.Color().fromArray(profile.body?.baseColorFactor || [0.7, 0.68, 0.62]);
    const partColor = new THREE.Color().fromArray(profile.parts?.baseColorFactor || [0.55, 0.52, 0.48]);
    root.traverse((obj) => {
      if (!obj.isMesh || !obj.material) return;
      const mat = obj.material.clone();
      const isPart = obj.userData.kind === "part" || obj.parent?.userData?.kind === "attachment";
      mat.color.copy(isPart ? partColor : bodyColor);
      obj.material = mat;
    });
  }

  function getExportPayload() {
    return [...attachments.values()].map((pivot) => ({
      partId: pivot.userData.partId,
      socket: pivot.userData.socket,
      translation: pivot.position.toArray(),
      rotation: pivot.quaternion.toArray(),
      scale: pivot.scale.toArray(),
    }));
  }

  return {
    loadBase,
    listBones,
    attachPart,
    selectAttachment,
    selectBone,
    setMode,
    setSkeletonVisible,
    applyMaterialProfile,
    getExportPayload,
    get attachments() {
      return attachments;
    },
  };
}
