/** Body-region colors + recommended placement order (1–21). */

import { toBlenderBoneName } from "./rigBoneNames.js";

export const PLACEMENT_ORDER = [
  "TBG_Root",
  "TBG_Spine",
  "TBG_Spine1",
  "TBG_Neck",
  "TBG_Head",
  "TBG_Shoulder.L",
  "TBG_UpperArm.L",
  "TBG_LowerArm.L",
  "TBG_Hand.L",
  "TBG_Shoulder.R",
  "TBG_UpperArm.R",
  "TBG_LowerArm.R",
  "TBG_Hand.R",
  "TBG_UpperLeg.L",
  "TBG_LowerLeg.L",
  "TBG_Foot.L",
  "TBG_Toe.L",
  "TBG_UpperLeg.R",
  "TBG_LowerLeg.R",
  "TBG_Foot.R",
  "TBG_Toe.R",
];

export const REGION_PALETTE = {
  head: { label: "Head & neck (4–5)", color: "#d946ef" },
  spine: { label: "Spine & pelvis (1–3)", color: "#22c55e" },
  armL: { label: "Left arm (6–9)", color: "#0ea5e9" },
  armR: { label: "Right arm (10–13)", color: "#f97316" },
  legL: { label: "Left leg (14–17)", color: "#f43f5e" },
  legR: { label: "Right leg (18–21)", color: "#eab308" },
};

const ORDER_INDEX = new Map(PLACEMENT_ORDER.map((name, i) => [name, i + 1]));

export function boneRegion(boneName) {
  const name = toBlenderBoneName(boneName);
  if (/TBG_(Head|Neck)/.test(name)) return "head";
  if (/TBG_(Root|Spine)/.test(name)) return "spine";
  if (/\.L$/.test(name) && /(Shoulder|UpperArm|LowerArm|Hand)/.test(name)) return "armL";
  if (/\.R$/.test(name) && /(Shoulder|UpperArm|LowerArm|Hand)/.test(name)) return "armR";
  if (/\.L$/.test(name)) return "legL";
  if (/\.R$/.test(name)) return "legR";
  return "spine";
}

export function regionHex(boneName) {
  return REGION_PALETTE[boneRegion(boneName)]?.color ?? "#94a3b8";
}

export function bonePlacementNumber(boneName) {
  return ORDER_INDEX.get(toBlenderBoneName(boneName)) ?? 0;
}

export function renderRegionLegend(container) {
  if (!container) return;
  const regions = Object.entries(REGION_PALETTE)
    .map(
      ([id, row]) =>
        `<span class="rig-region-legend__item" data-region="${id}">
          <span class="rig-region-legend__swatch" style="background:${row.color}"></span>${row.label}
        </span>`,
    )
    .join("");
  const numbers = PLACEMENT_ORDER.map(
    (name, i) => `<span class="rig-num-guide__item">${i + 1}=${name.replace("TBG_", "")}</span>`,
  ).join("");
  container.innerHTML = `${regions}<div class="rig-num-guide">${numbers}</div>`;
}
