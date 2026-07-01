/** GLB export — morph body download + assembled character job. */

import { pollJob, queueExport } from "../assembly-studio/assemblyApi.js";
import { LIVE_UNIT_ID } from "./mblabLive.js";

export function morphGlbUrl(unitId = LIVE_UNIT_ID) {
  return `/api/assembly/exports/${unitId}/mesh`;
}

export function triggerDownload(url, filename) {
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
}

export async function downloadMorphGlb(unitId, onStatus) {
  const url = `${morphGlbUrl(unitId)}?t=${Date.now()}`;
  onStatus("Downloading body GLB…");
  const probe = await fetch(url);
  if (!probe.ok) throw new Error("No morph GLB yet — move sliders or wait for live preview");
  triggerDownload(url, `${unitId}.glb`);
  onStatus("Download started");
}

export async function exportAssembledGlb({ unitId, baseId, partIds, materialId = "default", onStatus }) {
  onStatus("Assembling GLB…");
  const job = await queueExport({
    unitId,
    baseId,
    parts: partIds,
    materialId,
  });
  for (let i = 0; i < 120; i += 1) {
    const final = await pollJob(job.id);
    if (final.status === "completed") {
      const url = final.report?.exportUrl || `/api/assembly/exports/${unitId}/mesh`;
      triggerDownload(`${url}?t=${Date.now()}`, `${unitId}_assembled.glb`);
      onStatus("Assembled GLB ready");
      return url;
    }
    if (final.status === "failed") throw new Error(final.error || "Assembly failed");
    onStatus("Blender assembling…");
    await new Promise((r) => setTimeout(r, 500));
  }
  throw new Error("Assembly timed out");
}
