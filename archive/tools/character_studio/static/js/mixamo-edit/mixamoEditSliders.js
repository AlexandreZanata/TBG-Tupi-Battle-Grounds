/** Mixamo Edit align panel — bone proportion slider helpers. */

export const BONE_PROP_KEYS = ["hipWidth", "shoulderWidth", "torsoDepth"];

export function readBoneProportions() {
  return Object.fromEntries(
    BONE_PROP_KEYS.map((k) => [k, Number(document.getElementById(k)?.value ?? 100) / 100]),
  );
}

export function syncBoneProportions(props) {
  for (const k of BONE_PROP_KEYS) {
    const mult = props[k] ?? 1;
    document.getElementById(k).value = String(Math.round(mult * 100));
    document.getElementById(`${k}Out`).textContent = `${mult.toFixed(2)}×`;
  }
}

export function resetBoneProportionSliders() {
  syncBoneProportions({ hipWidth: 1, shoulderWidth: 1, torsoDepth: 1 });
}

export function bindBonePropSliders(alignRef) {
  for (const id of BONE_PROP_KEYS) {
    document.getElementById(id)?.addEventListener("input", (e) => {
      const mult = Number(e.target.value) / 100;
      document.getElementById(`${id}Out`).textContent = `${mult.toFixed(2)}×`;
      alignRef()?.setBoneProportions(readBoneProportions());
    });
  }
}
