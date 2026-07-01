/** Locked joints keep fixed world position while other bones move. */

import * as THREE from "three";

export function createLockStore(bones) {
  const fixedWorld = new Map();

  function applyHeadWorld(bone, worldPos) {
    if (bone.parent) {
      bone.position.copy(bone.parent.worldToLocal(worldPos.clone()));
    } else {
      bone.position.copy(worldPos);
    }
    bone.updateMatrixWorld(true);
  }

  function pin(boneName) {
    const bone = bones.find((b) => b.name === boneName);
    if (!bone) return;
    const pos = new THREE.Vector3();
    bone.getWorldPosition(pos);
    fixedWorld.set(boneName, pos);
  }

  function unpin(boneName) {
    fixedWorld.delete(boneName);
  }

  function restoreAll() {
    fixedWorld.forEach((pos, name) => {
      const bone = bones.find((b) => b.name === name);
      if (bone) applyHeadWorld(bone, pos);
    });
  }

  function moveBone(bone, worldPos) {
    applyHeadWorld(bone, worldPos);
    restoreAll();
  }

  return { pin, unpin, restoreAll, moveBone, isPinned: (name) => fixedWorld.has(name) };
}
