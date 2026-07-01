/** Force skinned meshes to pick up post-mixer / IK bone edits. */

export function refreshSkinnedMeshes(root) {
  if (!root) return;
  root.traverse((obj) => {
    if (obj.isSkinnedMesh?.skeleton) {
      obj.skeleton.update();
    }
  });
}
