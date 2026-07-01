/** Motion arena — grid floor + low walls for locomotion preview. */

import * as THREE from "three";

export function buildMotionArena(scene) {
  const root = new THREE.Group();
  root.name = "MotionArena";

  const floor = new THREE.Mesh(
    new THREE.PlaneGeometry(24, 24),
    new THREE.MeshStandardMaterial({ color: 0x2a2d35, roughness: 0.95, metalness: 0.05 }),
  );
  floor.rotation.x = -Math.PI / 2;
  floor.receiveShadow = true;
  root.add(floor);

  const grid = new THREE.GridHelper(24, 24, 0x5a6270, 0x3a4048);
  grid.position.y = 0.01;
  root.add(grid);

  const ring = new THREE.Mesh(
    new THREE.RingGeometry(3.5, 3.7, 64),
    new THREE.MeshBasicMaterial({ color: 0x88ccff, transparent: true, opacity: 0.35, side: THREE.DoubleSide }),
  );
  ring.rotation.x = -Math.PI / 2;
  ring.position.y = 0.02;
  root.add(ring);

  const wallMat = new THREE.MeshStandardMaterial({
    color: 0x1e2128,
    roughness: 1,
    transparent: true,
    opacity: 0.35,
  });
  for (const [w, h, d, x, y, z] of [
    [24, 2, 0.2, 0, 1, -12],
    [24, 2, 0.2, 0, 1, 12],
    [0.2, 2, 24, -12, 1, 0],
    [0.2, 2, 24, 12, 1, 0],
  ]) {
    const wall = new THREE.Mesh(new THREE.BoxGeometry(w, h, d), wallMat);
    wall.position.set(x, y, z);
    root.add(wall);
  }

  scene.add(root);
  return root;
}
