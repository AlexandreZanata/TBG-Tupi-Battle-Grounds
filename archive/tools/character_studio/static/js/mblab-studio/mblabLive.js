/** Debounced headless MB-Lab morph jobs for near-live preview. */

const GENERATE_URL = "/api/assembly/mblab/generate";
export const LIVE_UNIT_ID = "mblab_live_preview";

let debounceTimer = null;
let liveToken = 0;
let polling = false;

export function scheduleLiveMorph({ baseId, morphs, onStatus, onReady }) {
  clearTimeout(debounceTimer);
  onStatus("Adjusting…");
  debounceTimer = setTimeout(() => {
    void runLiveMorph({ baseId, morphs, onStatus, onReady });
  }, 650);
}

async function pollJob(jobId, onStatus) {
  for (let i = 0; i < 180; i += 1) {
    const res = await fetch(`/api/assembly/mblab/jobs/${jobId}`);
    const job = await res.json();
    if (job.status === "running") onStatus("Blender morph…");
    if (job.status === "completed") return job;
    if (job.status === "failed") throw new Error(job.error || "Morph failed");
    await new Promise((r) => setTimeout(r, 400));
  }
  throw new Error("Morph timed out");
}

export async function runLiveMorph({ baseId, morphs, onStatus, onReady }) {
  if (polling) liveToken += 1;
  const token = ++liveToken;
  polling = true;
  onStatus("Queuing morph…");
  try {
    const res = await fetch(GENERATE_URL, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        unitId: LIVE_UNIT_ID,
        baseId,
        promoteToLibrary: false,
        morphs,
      }),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({}));
      throw new Error(err.detail || res.statusText);
    }
    const job = await res.json();
    const final = await pollJob(job.id, onStatus);
    if (token !== liveToken) return null;
    const url = `${final.report?.exportUrl || `/api/assembly/exports/${LIVE_UNIT_ID}/mesh`}?t=${Date.now()}`;
    await onReady(url);
    onStatus("Live");
    return url;
  } catch (err) {
    if (token === liveToken) onStatus(err.message || "Morph failed");
    return null;
  } finally {
    polling = false;
  }
}
