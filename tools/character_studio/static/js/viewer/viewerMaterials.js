/** Material handling — preserve embedded GLB PBR; optional external textures + unit color tint. */

import * as THREE from "three";

export function meshHasEmbeddedMaps(root) {
  let found = false;
  root.traverse((obj) => {
    if (!(obj instanceof THREE.Mesh)) return;
    const mats = Array.isArray(obj.material) ? obj.material : [obj.material];
    mats.forEach((mat) => {
      if (mat?.map || mat?.normalMap || mat?.metalnessMap || mat?.roughnessMap) {
        found = true;
      }
    });
  });
  return found;
}

export async function applyCharacterMaterials(root, character) {
  const hasEmbedded = meshHasEmbeddedMaps(root);
  if (character.textures) {
    await applyExternalTextures(root, character);
    return;
  }
  if (hasEmbedded) {
    upgradeEmbeddedMaterials(root);
    applyColorTint(root, character.primaryColor, character.secondaryColor);
    return;
  }
  applyFallbackMaterial(root, character.primaryColor);
}

function upgradeEmbeddedMaterials(root) {
  root.traverse((obj) => {
    if (!(obj instanceof THREE.Mesh)) return;
    const source = Array.isArray(obj.material) ? obj.material : [obj.material];
    const mapped = source.map((mat) => {
      if (!mat) return new THREE.MeshStandardMaterial({ color: 0xffffff, roughness: 0.55 });
      mat.side = THREE.DoubleSide;
      if (mat.map) mat.map.colorSpace = THREE.SRGBColorSpace;
      return mat;
    });
    obj.material = Array.isArray(obj.material) ? mapped : mapped[0];
  });
}

function applyColorTint(root, primaryHex, secondaryHex) {
  const primary = (primaryHex || "#ffffff").toLowerCase();
  const secondary = (secondaryHex || "#ffffff").toLowerCase();
  if (primary === "#ffffff" && secondary === "#ffffff") {
    return;
  }
  const primaryColor = new THREE.Color(primaryHex || "#ffffff");
  const secondaryColor = new THREE.Color(secondaryHex || "#888888");
  let index = 0;
  root.traverse((obj) => {
    if (!(obj instanceof THREE.Mesh)) return;
    const mats = Array.isArray(obj.material) ? obj.material : [obj.material];
    mats.forEach((mat) => {
      if (!mat) return;
      const tint = index % 2 === 0 ? primaryColor : secondaryColor;
      index += 1;
      if (mat.map) {
        mat.color.copy(tint);
      } else {
        mat.color.copy(tint);
      }
      mat.needsUpdate = true;
    });
  });
}

function applyFallbackMaterial(root, primaryHex) {
  const color = new THREE.Color(primaryHex || "#9ca3af");
  root.traverse((obj) => {
    if (!(obj instanceof THREE.Mesh)) return;
    obj.material = new THREE.MeshStandardMaterial({
      color,
      roughness: 0.55,
      metalness: 0.12,
      side: THREE.DoubleSide,
    });
  });
}

async function applyExternalTextures(root, character) {
  const textures = character.textures;
  const loader = new THREE.TextureLoader();
  const albedoUrl = toContentUrl(textures.albedo);
  const normalUrl = toContentUrl(textures.normal);
  const [albedoMap, normalMap] = await Promise.all([
    albedoUrl ? loadTexture(loader, albedoUrl) : null,
    normalUrl ? loadTexture(loader, normalUrl) : null,
  ]);
  root.traverse((obj) => {
    if (!(obj instanceof THREE.Mesh)) return;
    obj.material = new THREE.MeshStandardMaterial({
      color: 0xffffff,
      map: albedoMap,
      normalMap: normalMap,
      roughness: 0.55,
      metalness: 0.1,
      side: THREE.DoubleSide,
    });
  });
}

function loadTexture(loader, url) {
  return new Promise((resolve, reject) => {
    loader.load(url, resolve, undefined, reject);
  });
}

function toContentUrl(repoPath) {
  if (!repoPath) return null;
  return `/api/content/${repoPath.replace(/^content\//, "")}`;
}

export function normalizeMeshes(root) {
  root.traverse((obj) => {
    if (!(obj instanceof THREE.Mesh)) return;
    if (!obj.geometry.attributes.normal) {
      obj.geometry.computeVertexNormals();
    }
  });
}
