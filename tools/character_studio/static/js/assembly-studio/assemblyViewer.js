/** Minimal Three.js preview — base body + socket-attached parts. */

import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";

export function mountAssemblyViewer(container) {
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0xe8e6e1);

  const renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  container.replaceChildren(renderer.domElement);

  const camera = new THREE.PerspectiveCamera(36, 1, 0.05, 100);
  camera.position.set(1.6, 1.4, 2.2);

  const controls = new OrbitControls(camera, renderer.domElement);
  controls.target.set(0, 0.9, 0);
  controls.enableDamping = true;

  const hemi = new THREE.HemisphereLight(0xffffff, 0x8a8278, 1.1);
  const key = new THREE.DirectionalLight(0xffffff, 1.2);
  key.position.set(2, 4, 3);
  scene.add(hemi, key);

  const loader = new GLTFLoader();
  let root = null;
  let token = 0;
  let frameId = 0;

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

  const animate = () => {
    frameId = requestAnimationFrame(animate);
    controls.update();
    renderer.render(scene, camera);
  };
  animate();

  function clearRoot() {
    if (!root) return;
    scene.remove(root);
    root.traverse((obj) => {
      if (obj.geometry) obj.geometry.dispose();
      if (obj.material) {
        const mats = Array.isArray(obj.material) ? obj.material : [obj.material];
        mats.forEach((m) => m.dispose?.());
      }
    });
    root = null;
  }

  function fitCamera(target) {
    const box = new THREE.Box3().setFromObject(target);
    const size = box.getSize(new THREE.Vector3());
    const center = box.getCenter(new THREE.Vector3());
    const dist = Math.max(size.x, size.y, size.z) * 1.8;
    camera.position.set(center.x + dist * 0.6, center.y + size.y * 0.5, center.z + dist);
    controls.target.copy(center);
    controls.update();
  }

  async function loadPreview({ baseUrl, parts }) {
    const current = ++token;
    clearRoot();
    const gltf = await loader.loadAsync(baseUrl);
    if (current !== token) return;
    root = gltf.scene;
    scene.add(root);
    const bones = {};
    root.traverse((obj) => {
      if (obj.isBone) bones[obj.name] = obj;
    });
    for (const part of parts) {
      const bone = bones[part.socket];
      if (!bone || !part.meshUrl) continue;
      const partGltf = await loader.loadAsync(part.meshUrl);
      if (current !== token) return;
      const mesh = partGltf.scene;
      mesh.position.set(0, 0, 0);
      bone.add(mesh);
    }
    fitCamera(root);
  }

  return {
    loadPreview,
    dispose() {
      token += 1;
      cancelAnimationFrame(frameId);
      observer.disconnect();
      clearRoot();
      renderer.dispose();
    },
  };
}
