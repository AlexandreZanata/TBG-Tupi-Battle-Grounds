/** Natural human walk — CMU segment gait (archive walk/segment_gait.py). */

import {
  ELBOW_CURVE_DEG,
  HEAD_PITCH_DEG,
  NECK_PITCH_DEG,
  PELVIS_VERTICAL_NORM,
  SHOULDER_CURVE_DEG,
  THORAX_PITCH_DEG,
  sampleCycleCurve,
  sampleLegAngles,
} from "./walkCurves.js";

const DEG = Math.PI / 180;
const ELBOW_LAG = 0.08;
export const WALK_FPS = 30;

export const DEFAULT_GAIT_CONFIG = {
  cadenceSpm: 115,
  angleScale: 0.9,
  pelvisVerticalM: 0.038,
  armSwing: 0.24,
  forearmFollow: 0.45,
  torsoScale: 1,
  neckScale: 1,
  headScale: 1,
  armSwingRScale: 1,
  spine1PitchScale: 0.68,
};

export const HUMANOID_LIGHT_STYLE = DEFAULT_GAIT_CONFIG;

export function cycleFrames(cadenceSpm = DEFAULT_GAIT_CONFIG.cadenceSpm) {
  return Math.max(24, Math.min(48, Math.round((120 / cadenceSpm) * WALK_FPS)));
}

function trunkPitch(tau, cfg) {
  const thorax = sampleCycleCurve(THORAX_PITCH_DEG, tau) * DEG * cfg.torsoScale;
  const neck = sampleCycleCurve(NECK_PITCH_DEG, tau) * DEG * cfg.neckScale;
  const head = sampleCycleCurve(HEAD_PITCH_DEG, tau) * DEG * cfg.headScale;
  return { spine: thorax, spine1: thorax * cfg.spine1PitchScale, neck, head };
}

function armPair(legTau, cfg, rScale) {
  const elbowTau = (legTau - ELBOW_LAG + 1) % 1;
  const shoulderRef = 15 * DEG;
  const elbowRef = 35 * DEG;
  let swing = sampleCycleCurve(SHOULDER_CURVE_DEG, legTau) * DEG;
  swing *= (cfg.armSwing / shoulderRef) * rScale;
  let elbow = sampleCycleCurve(ELBOW_CURVE_DEG, elbowTau) * DEG;
  elbow *= (cfg.armSwing / elbowRef) * rScale;
  const follow = elbow * cfg.forearmFollow;
  return {
    upper: [-0.04 + swing, 0, 0],
    lower: [-0.18 + follow, 0, 0],
  };
}

/** Pose deltas in bone-local space (radians). Root: Y bob only — no yaw/roll on bind. */
export function sampleWalkPose(frame, cycleLen, cfg = DEFAULT_GAIT_CONFIG) {
  const tau = (frame % cycleLen) / cycleLen;
  const tauL = (tau + 0.5) % 1;
  const s = cfg.angleScale;
  const leg = sampleLegAngles(tau, s);
  const trunk = trunkPitch(tau, cfg);
  const rootY = sampleCycleCurve(PELVIS_VERTICAL_NORM, tau) * cfg.pelvisVerticalM;
  const rArm = armPair(tauL, cfg, cfg.armSwingRScale);
  const lArm = armPair(tau, cfg, 1);

  const deg = (d) => d * DEG;
  return {
    TBG_Root: { pos: [0, rootY, 0] },
    TBG_Spine: { rot: [trunk.spine, 0, 0] },
    TBG_Spine1: { rot: [trunk.spine1, 0, 0] },
    TBG_Neck: { rot: [trunk.neck, 0, 0] },
    TBG_Head: { rot: [trunk.head, 0, 0] },
    "TBG_UpperLeg.R": { rot: [deg(leg.rHip), 0, 0] },
    "TBG_LowerLeg.R": { rot: [deg(leg.rKnee), 0, 0] },
    "TBG_Foot.R": { rot: [deg(leg.rAnkle), 0, 0] },
    "TBG_Toe.R": { rot: [deg(leg.rToe), 0, 0] },
    "TBG_UpperLeg.L": { rot: [deg(leg.lHip), 0, 0] },
    "TBG_LowerLeg.L": { rot: [deg(leg.lKnee), 0, 0] },
    "TBG_Foot.L": { rot: [deg(leg.lAnkle), 0, 0] },
    "TBG_Toe.L": { rot: [deg(leg.lToe), 0, 0] },
    "TBG_UpperArm.R": { rot: rArm.upper },
    "TBG_LowerArm.R": { rot: rArm.lower },
    "TBG_UpperArm.L": { rot: lArm.upper },
    "TBG_LowerArm.L": { rot: lArm.lower },
  };
}
