/** Studio-style lighting — dark (studio) or light (showcase modal). */

import * as THREE from "three";

const STAGE = {
  dark: {
    background: 0x1a1a22,
    fogNear: 8,
    fogFar: 22,
    hemisphereSky: 0xe8eef8,
    hemisphereGround: 0xe8e4dc,
    floor: 0xf2f0eb,
  },
  light: {
    background: 0xf7f5f0,
    fogNear: 14,
    fogFar: 32,
    hemisphereSky: 0xffffff,
    hemisphereGround: 0xf0ebe3,
    floor: 0xfaf8f4,
  },
  arena: {
    background: 0x12141a,
    fogNear: 10,
    fogFar: 28,
    hemisphereSky: 0xd0d8e8,
    hemisphereGround: 0x3a4048,
    floor: null,
  },
};

export function applyStudioLighting(scene, variant = "dark") {
  const palette = STAGE[variant] ?? STAGE.dark;
  scene.background = new THREE.Color(palette.background);
  scene.fog = new THREE.Fog(palette.background, palette.fogNear, palette.fogFar);

  scene.add(new THREE.HemisphereLight(palette.hemisphereSky, palette.hemisphereGround, 0.62));

  const key = new THREE.DirectionalLight(0xfff8f0, variant === "light" ? 1.15 : 1.35);
  key.position.set(2.8, 5.5, 3.2);
  scene.add(key);

  const rim = new THREE.DirectionalLight(0xd8e8ff, variant === "light" ? 0.35 : 0.65);
  rim.position.set(-3.5, 2.5, -2.5);
  scene.add(rim);

  const fill = new THREE.DirectionalLight(0xffffff, 0.4);
  fill.position.set(-2, 1.5, 4);
  scene.add(fill);

  const floor = new THREE.Mesh(
    new THREE.CircleGeometry(2.2, 48),
    new THREE.MeshStandardMaterial({
      color: palette.floor ?? 0xf2f0eb,
      roughness: 0.98,
      metalness: 0.0,
    }),
  );
  if (palette.floor != null) {
    floor.rotation.x = -Math.PI / 2;
    floor.position.y = 0.001;
    floor.receiveShadow = true;
    scene.add(floor);
  }
}

export function configureRenderer(renderer) {
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.05;
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = THREE.PCFSoftShadowMap;
}
