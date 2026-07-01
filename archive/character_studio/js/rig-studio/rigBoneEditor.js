/**
 * In-browser TBG bone placement — x-ray mesh + joint markers.
 */

import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { loadGltfScene } from "../viewer/gltfLoader.js";
import { applyStudioLighting, configureRenderer } from "../viewer/viewerLighting.js";
import { bindViewerResize, prepareViewerRoot, readViewerSize, applyRendererSize } from "../viewer/viewerResize.js";
import { placeOnGround, fitCamera, disposeObject } from "../viewer/viewerSceneUtils.js";
import { createRigDragController } from "./rigBoneDrag.js";
import { createJointHandle } from "./rigBoneHandles.js";
import { createLockStore } from "./rigBoneLock.js";
import { bonePlacementNumber } from "./rigBoneRegions.js";
import { setMeshXray } from "./rigMeshXray.js";
import { createBoneLines, disposeBoneLines, isRigParent, syncBoneLines } from "./rigBoneLines.js";
import { toBlenderBoneName } from "./rigBoneNames.js";
import { syncTailMarker } from "./rigTailMarker.js";

const BONE_PREFIX = "TBG_";

export function mountRigBoneEditor(container) {
  const mount = prepareViewerRoot(container);
  const scene = new THREE.Scene();
  applyStudioLighting(scene, "dark");

  const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: false });
  configureRenderer(renderer);
  mount.appendChild(renderer.domElement);

  const { width, height } = readViewerSize(container);
  applyRendererSize(renderer, width, height, 2);

  const camera = new THREE.PerspectiveCamera(36, width / Math.max(height, 1), 0.01, 200);
  const controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;
  controls.target.set(0, 0.9, 0);

  let modelRoot = null;
  let meshRoot = null;
  let rigNodes = [];
  let boneLines = null;
  let handles = new Map();
  let xrayMesh = true;
  let wireframe = false;
  let showTails = true;
  let editable = false;
  let loadToken = 0;
  let tailOffsets = new Map();
  let dragCtl = null;
  let lockStore = null;
  let viewportW = width;
  let viewportH = height;

  bindViewerResize(container, renderer, camera, (w, h) => {
    viewportW = w;
    viewportH = h;
    if (modelRoot) fitCamera(modelRoot, camera, w, h);
  });

  const animate = () => {
    requestAnimationFrame(animate);
    controls.update();
    lockStore?.restoreAll();
    syncHandlesFromNodes();
    updateBoneLines();
    renderer.render(scene, camera);
  };
  animate();

  function applyMeshView() {
    if (!meshRoot) return;
    setMeshXray(meshRoot, editable && xrayMesh, { opacity: 0.32, wireframe });
  }

  function clearScene() {
    dragCtl?.dispose();
    dragCtl = null;
    handles.forEach((group) => scene.remove(group));
    handles.clear();
    rigNodes = [];
    lockStore = null;
    if (boneLines) boneLines = disposeBoneLines(scene, boneLines);
    if (modelRoot) {
      scene.remove(modelRoot);
      disposeObject(modelRoot);
      modelRoot = null;
      meshRoot = null;
    }
  }

  function collectRigNodes(root) {
    const list = [];
    root.traverse((obj) => {
      if (obj.isMesh || obj.isLine || obj.isLineSegments) return;
      if (obj.name?.startsWith(BONE_PREFIX)) list.push(obj);
    });
    return list.sort((a, b) => bonePlacementNumber(a.name) - bonePlacementNumber(b.name));
  }

  function isRigParentLocal(node) {
    return isRigParent(node, BONE_PREFIX);
  }

  function rebuildBoneLines() {
    boneLines = disposeBoneLines(scene, boneLines);
    if (!editable || !rigNodes.length) return;
    boneLines = createBoneLines(scene, rigNodes, BONE_PREFIX);
  }

  function updateBoneLines() {
    if (!boneLines || !editable) return;
    syncBoneLines(boneLines, rigNodes, BONE_PREFIX);
  }

  function buildHandles() {
    handles.forEach((group) => scene.remove(group));
    handles = new Map();
    rigNodes.forEach((node) => {
      const group = createJointHandle(node.name);
      group.visible = editable;
      scene.add(group);
      handles.set(node.name, group);
    });
    rebuildBoneLines();
    applyMeshView();
    lockStore = createLockStore(rigNodes);
    if (editable) {
      dragCtl = createRigDragController({
        camera,
        controls,
        renderer,
        bones: rigNodes,
        handles,
        lockStore,
        onLockChange,
      });
    }
  }

  function onLockChange(boneName, locked, hint) {
    editorLockCallback?.(boneName, locked, countLocked(), hint);
  }

  let editorLockCallback = null;

  function countLocked() {
    let n = 0;
    handles.forEach((g) => { if (g.userData.locked) n += 1; });
    return n;
  }

  function syncHandlesFromNodes() {
    if (!editable) return;
    rigNodes.forEach((node) => {
      const group = handles.get(node.name);
      if (!group) return;
      node.getWorldPosition(group.position);
      const tail = group.children[3];
      const label = group.children[4];
      if (label?.isSprite) {
        label.lookAt(camera.position);
      }
      tail.visible = showTails;
      if (showTails) {
        syncTailMarker(group, node, tailOffsets.get(node.name) ?? new THREE.Vector3(0, 0.1, 0));
      }
    });
  }

  function boneEntry(source, nodeName) {
    const bones = source?.bones;
    if (!bones) return null;
    return bones[nodeName] ?? bones[toBlenderBoneName(nodeName)] ?? null;
  }

  function captureTailOffsets(source) {
    tailOffsets = new Map();
    rigNodes.forEach((node) => {
      const saved = boneEntry(source, node.name);
      if (saved?.head && saved?.tail) {
        tailOffsets.set(
          node.name,
          new THREE.Vector3(
            saved.tail[0] - saved.head[0],
            saved.tail[1] - saved.head[1],
            saved.tail[2] - saved.head[2],
          ),
        );
      } else {
        tailOffsets.set(node.name, new THREE.Vector3(0, 0.1, 0));
      }
    });
  }

  async function loadPreview(url, { edit = false, bonesSource = null } = {}) {
    const token = ++loadToken;
    clearScene();
    const { scene: gltfScene } = await loadGltfScene(url);
    if (token !== loadToken) return;

    const root = new THREE.Group();
    root.add(gltfScene);
    placeOnGround(root, gltfScene);
    scene.add(root);
    modelRoot = root;
    meshRoot = gltfScene;
    rigNodes = collectRigNodes(gltfScene);
    captureTailOffsets(bonesSource);
    if (bonesSource?.bones) {
      rigNodes.forEach((node) => {
        const saved = boneEntry(bonesSource, node.name);
        if (saved?.head) {
          node.position.set(saved.head[0], saved.head[1], saved.head[2]);
          node.updateMatrixWorld(true);
        }
      });
    }
    editable = edit && rigNodes.length > 0;
    buildHandles();
    fitCamera(root, camera, viewportW, viewportH);
    syncHandlesFromNodes();
  }

  function exportBonesPayload() {
    const payload = { version: 1, armature: null, bones: {} };
    rigNodes.forEach((node) => {
      const offset = tailOffsets.get(node.name) ?? new THREE.Vector3(0, 0.1, 0);
      const head = node.position;
      const tail = head.clone().add(offset);
      const name = toBlenderBoneName(node.name);
      payload.bones[name] = {
        head: [head.x, head.y, head.z],
        tail: [tail.x, tail.y, tail.z],
        parent: isRigParentLocal(node.parent) ? toBlenderBoneName(node.parent.name) : null,
      };
    });
    return payload;
  }

  return {
    loadPreview,
    exportBonesPayload,
    onLockChange(callback) {
      editorLockCallback = callback;
    },
    getLockedCount: () => countLocked(),
    setXrayMesh(value) {
      xrayMesh = Boolean(value);
      applyMeshView();
    },
    setWireframe(value) {
      wireframe = Boolean(value);
      applyMeshView();
    },
    setShowTails(value) {
      showTails = Boolean(value);
    },
    getBoneNames: () => rigNodes.map((n) => n.name),
    isEditable: () => editable,
    dispose() {
      clearScene();
      renderer.dispose();
      mount.remove();
    },
  };
}
