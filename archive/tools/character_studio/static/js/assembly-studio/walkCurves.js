/** CMU-calibrated gait curves (subjects 35/39) — Catmull-Rom cyclic splines. */

function catmullRom(p0, p1, p2, p3, t) {
  const t2 = t * t;
  const t3 = t2 * t;
  return 0.5 * (
    2 * p1 + (-p0 + p2) * t
    + (2 * p0 - 5 * p1 + 4 * p2 - p3) * t2
    + (-p0 + 3 * p1 - 3 * p2 + p3) * t3
  );
}

export function sampleCycleCurve(keyframes, tau) {
  if (!keyframes.length) return 0;
  let pts = [...keyframes].sort((a, b) => a[0] - b[0]);
  if (pts[0][0] > 0) pts = [[0, pts[pts.length - 1][1]], ...pts];
  if (pts[pts.length - 1][0] < 1) pts = [...pts, [1, pts[0][1]]];

  const times = pts.map((p) => p[0]);
  const core = pts.slice(0, -1).map((p) => p[1]);
  const n = core.length;
  tau = ((tau % 1) + 1) % 1;

  let seg = 0;
  for (let i = 0; i < times.length - 1; i += 1) {
    if (tau >= times[i] && tau <= times[i + 1]) {
      seg = i;
      break;
    }
  }
  const span = Math.max(1e-9, times[seg + 1] - times[seg]);
  const local = (tau - times[seg]) / span;
  const at = (off) => core[(seg + off + n) % n];
  return catmullRom(at(-1), at(0), at(1), at(2), local);
}

export const STANCE_HIP_ADD_DEG = [
  [0, 2.5], [0.15, 4], [0.3, 3], [0.5, 2.5], [0.65, 4], [0.8, 3], [1, 2.5],
];

export const HIP_CURVE_DEG = [
  [0, 30], [0.02, 29.9], [0.05, 29.6], [0.1, 29.3], [0.3, 5.8], [0.5, -12.8],
  [0.55, -11.4], [0.6, -2.7], [0.73, 6.5], [0.87, 20.4], [1, 26.8],
];

export const KNEE_CURVE_DEG = [
  [0, 0], [0.05, 14], [0.1, 16.3], [0.3, 6.5], [0.5, 2.2], [0.55, 2],
  [0.6, 33.1], [0.73, 45.8], [0.87, 23.5], [1, -2.8],
];

export const ANKLE_CURVE_DEG = [
  [0, 0], [0.08, -6], [0.3, 5.3], [0.5, 8.8], [0.55, 8.9], [0.6, -13.8],
  [0.73, 0.3], [0.87, 0.1], [1, -0.5],
];

export const TOE_CURVE_DEG = [
  [0, 0], [0.45, 0], [0.55, -22], [0.7, -12], [0.85, -4], [1, 0],
];

export const PELVIS_VERTICAL_NORM = [
  [0, 0], [0.25, 1], [0.5, 0], [0.75, 1], [1, 0],
];

export const THORAX_PITCH_DEG = [
  [0, -2.5], [0.25, 2.5], [0.5, -2.5], [0.75, 2.5], [1, -2.5],
];

export const NECK_PITCH_DEG = [
  [0, 1.5], [0.25, -1.5], [0.5, 1.5], [0.75, -1.5], [1, 1.5],
];

export const HEAD_PITCH_DEG = [
  [0, 0.5], [0.25, -0.5], [0.5, 0.5], [0.75, -0.5], [1, 0.5],
];

export const SHOULDER_CURVE_DEG = [[0, -15], [0.5, 15], [1, -15]];
export const ELBOW_CURVE_DEG = [[0, 35], [0.5, 20], [1, 35]];

const STANCE_END = 0.6;

function legPair(curve, tau, tauL, scale) {
  return [
    sampleCycleCurve(curve, tau) * scale,
    sampleCycleCurve(curve, tauL) * scale,
  ];
}

/** Right + left leg sagittal angles (degrees). Left ankle mirrored (Blender convention). */
export function sampleLegAngles(tau, scale = 1) {
  const tauL = (tau + 0.5) % 1;
  let [rHip, lHip] = legPair(HIP_CURVE_DEG, tau, tauL, scale);
  const boost = 0.55;
  rHip += sampleCycleCurve(STANCE_HIP_ADD_DEG, tau) * boost;
  lHip += sampleCycleCurve(STANCE_HIP_ADD_DEG, tauL) * boost;

  const [rKnee, lKnee] = legPair(KNEE_CURVE_DEG, tau, tauL, scale);
  let [rAnkle, lAnkle] = legPair(ANKLE_CURVE_DEG, tau, tauL, scale);
  lAnkle = -lAnkle;

  let [rToe, lToe] = legPair(TOE_CURVE_DEG, tau, tauL, scale * 0.85);
  lToe = -lToe;

  const stanceR = tau < STANCE_END ? 1 - tau / STANCE_END : 0;
  const stanceL = tauL < STANCE_END ? 1 - tauL / STANCE_END : 0;
  rToe *= 1 - stanceR * 0.85;
  lToe *= 1 - stanceL * 0.85;

  return { rHip, lHip, rKnee, lKnee, rAnkle, lAnkle, rToe, lToe };
}
