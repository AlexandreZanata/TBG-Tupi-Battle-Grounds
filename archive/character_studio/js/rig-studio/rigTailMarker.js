/** Tail marker world position from bone head + local offset. */

import * as THREE from "three";

export function syncTailMarker(group, node, tailOffset) {
  const tail = group.children[3];
  const headW = new THREE.Vector3();
  const tailW = new THREE.Vector3();
  node.getWorldPosition(headW);
  const tailLocal = node.position.clone().add(tailOffset);
  tailW.copy(tailLocal);
  if (node.parent) node.parent.localToWorld(tailW);
  tail.position.copy(tailW.sub(headW));
}
