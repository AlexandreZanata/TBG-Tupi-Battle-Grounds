/** Pointer drag — lock pins world position; numbered independent joints. */

import * as THREE from "three";
import { applyHandleVisual, pickHandleMeshes } from "./rigBoneHandles.js";

const DOUBLE_CLICK_MS = 360;

export function createRigDragController({ camera, controls, renderer, bones, handles, lockStore, onLockChange }) {
  const raycaster = new THREE.Raycaster();
  const pointer = new THREE.Vector2();
  const dragPlane = new THREE.Plane();
  const hit = new THREE.Vector3();
  let dragState = null;
  let clickMemory = { name: null, time: 0 };

  function setPointer(event) {
    const rect = renderer.domElement.getBoundingClientRect();
    pointer.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
    pointer.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
  }

  function findHandleGroup(mesh) {
    let group = mesh;
    while (group && !group.userData?.boneName) group = group.parent;
    return group ?? null;
  }

  function findBone(group) {
    return bones.find((b) => b.name === group.userData.boneName) ?? null;
  }

  function pickHandle(event) {
    setPointer(event);
    raycaster.setFromCamera(pointer, camera);
    const hits = raycaster.intersectObjects(pickHandleMeshes(handles), false);
    return hits[0]?.object ?? null;
  }

  function toggleLock(group, bone) {
    const next = !group.userData.locked;
    group.userData.locked = next;
    if (next) lockStore.pin(bone.name);
    else lockStore.unpin(bone.name);
    applyHandleVisual(group, "idle");
    onLockChange?.(bone.name, next, "toggle");
  }

  function onPointerDown(event) {
    if (event.button !== 0) return;
    const mesh = pickHandle(event);
    if (!mesh) return;
    const group = findHandleGroup(mesh);
    const bone = findBone(group);
    if (!bone || !group) return;

    const now = performance.now();
    if (clickMemory.name === bone.name && now - clickMemory.time < DOUBLE_CLICK_MS) {
      toggleLock(group, bone);
      clickMemory = { name: null, time: 0 };
      return;
    }
    clickMemory = { name: bone.name, time: now };

    if (group.userData.locked) {
      onLockChange?.(bone.name, true, "tap_locked");
      return;
    }

    controls.enabled = false;
    const normal = camera.getWorldDirection(new THREE.Vector3()).negate();
    dragPlane.setFromNormalAndCoplanarPoint(normal, group.position);
    applyHandleVisual(group, "active");
    dragState = { bone, group };
  }

  function onPointerMove(event) {
    if (!dragState) {
      handles.forEach((group) => applyHandleVisual(group, "idle"));
      const mesh = pickHandle(event);
      if (mesh) applyHandleVisual(findHandleGroup(mesh), "hover");
      return;
    }
    setPointer(event);
    raycaster.setFromCamera(pointer, camera);
    if (raycaster.ray.intersectPlane(dragPlane, hit)) {
      lockStore.moveBone(dragState.bone, hit);
    }
  }

  function onPointerUp() {
    if (!dragState) return;
    lockStore.restoreAll();
    applyHandleVisual(dragState.group, "idle");
    dragState = null;
    controls.enabled = true;
  }

  renderer.domElement.addEventListener("pointerdown", onPointerDown);
  renderer.domElement.addEventListener("pointermove", onPointerMove);
  renderer.domElement.addEventListener("pointerup", onPointerUp);
  renderer.domElement.addEventListener("pointerleave", onPointerUp);

  return {
    restoreLocked: () => lockStore.restoreAll(),
    dispose() {
      renderer.domElement.removeEventListener("pointerdown", onPointerDown);
      renderer.domElement.removeEventListener("pointermove", onPointerMove);
      renderer.domElement.removeEventListener("pointerup", onPointerUp);
      renderer.domElement.removeEventListener("pointerleave", onPointerUp);
    },
  };
}
