/** Lightweight arena locomotion — velocity, bounds, ground snap. */

const BOUNDS = 11;
const ACCEL = 9;
const FRICTION = 7;
const MAX_SPEED = 1.25;

export function createArenaController(displayRoot, keys = () => ({}), { allowMove = () => true } = {}) {
  const velocity = { x: 0, z: 0 };
  let yaw = 0;
  const groundY = displayRoot.userData.groundY ?? displayRoot.position.y;

  return {
    update(delta) {
      if (!allowMove()) {
        velocity.x = 0;
        velocity.z = 0;
        return 0;
      }
      const input = keys();
      let ix = 0;
      let iz = 0;
      if (input.forward) iz += 1;
      if (input.back) iz -= 1;
      if (input.left) ix -= 1;
      if (input.right) ix += 1;
      const len = Math.hypot(ix, iz) || 1;
      ix /= len;
      iz /= len;

      velocity.x += ix * ACCEL * delta;
      velocity.z += iz * ACCEL * delta;
      velocity.x -= velocity.x * FRICTION * delta;
      velocity.z -= velocity.z * FRICTION * delta;

      const speed = Math.hypot(velocity.x, velocity.z);
      if (speed > MAX_SPEED) {
        velocity.x = (velocity.x / speed) * MAX_SPEED;
        velocity.z = (velocity.z / speed) * MAX_SPEED;
      }

      displayRoot.position.x += velocity.x * delta;
      displayRoot.position.z += velocity.z * delta;
      displayRoot.position.x = clamp(displayRoot.position.x, -BOUNDS, BOUNDS);
      displayRoot.position.z = clamp(displayRoot.position.z, -BOUNDS, BOUNDS);
      displayRoot.position.y = groundY;

      if (speed > 0.05) {
        yaw = Math.atan2(velocity.x, velocity.z);
        displayRoot.rotation.y = yaw;
      }
      return speed;
    },
    reset() {
      velocity.x = 0;
      velocity.z = 0;
      displayRoot.position.set(0, groundY, 0);
      displayRoot.rotation.y = 0;
    },
    getSpeed() {
      return Math.hypot(velocity.x, velocity.z);
    },
  };
}

function clamp(value, min, max) {
  return Math.max(min, Math.min(max, value));
}
