/** Semi-transparent mesh + optional wireframe for joint alignment. */

export function setMeshXray(root, enabled, { opacity = 0.3, wireframe = false } = {}) {
  if (!root) return;
  root.traverse((obj) => {
    if (!obj.isMesh) return;
    const materials = Array.isArray(obj.material) ? obj.material : [obj.material];
    materials.forEach((mat) => {
      if (!mat) return;
      if (!mat.userData.rigSaved) {
        mat.userData.rigSaved = {
          transparent: mat.transparent,
          opacity: mat.opacity,
          depthWrite: mat.depthWrite,
          wireframe: mat.wireframe,
          side: mat.side,
        };
      }
      const saved = mat.userData.rigSaved;
      if (enabled) {
        mat.transparent = true;
        mat.opacity = opacity;
        mat.depthWrite = false;
        mat.wireframe = wireframe;
        mat.side = THREE_SIDE_DOUBLE;
      } else {
        mat.transparent = saved.transparent;
        mat.opacity = saved.opacity;
        mat.depthWrite = saved.depthWrite;
        mat.wireframe = saved.wireframe;
        mat.side = saved.side;
      }
      mat.needsUpdate = true;
    });
  });
}

const THREE_SIDE_DOUBLE = 2;
