/** Rig Studio job API client. */

export async function api(path, options = {}) {
  const response = await fetch(path, {
    headers: { "Content-Type": "application/json", ...options.headers },
    ...options,
  });
  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(data.detail ?? response.statusText);
  }
  return data;
}

export function startJob(unitId, kind, fast = true) {
  return api("/api/rig/jobs", {
    method: "POST",
    body: JSON.stringify({ unitId, kind, fast }),
  });
}

export function fetchJob(jobId) {
  return api(`/api/rig/jobs/${jobId}`);
}

export function pollJob(jobId, { intervalMs = 2000, onTick } = {}) {
  return new Promise((resolve, reject) => {
    const tick = async () => {
      try {
        const job = await fetchJob(jobId);
        onTick?.(job);
        if (job.status === "completed") {
          resolve(job);
          return;
        }
        if (job.status === "failed") {
          reject(new Error(job.error ?? "job failed"));
          return;
        }
        setTimeout(tick, intervalMs);
      } catch (error) {
        reject(error);
      }
    };
    tick();
  });
}

export function fetchRigConfig(unitId) {
  return api(`/api/rig/units/${unitId}/config`);
}

export function fetchSession(unitId) {
  return api(`/api/rig/sessions/${unitId}`);
}

export function fetchBones(unitId) {
  return api(`/api/rig/sessions/${unitId}/bones`);
}

export function saveBones(unitId, payload) {
  return api(`/api/rig/sessions/${unitId}/bones`, {
    method: "PUT",
    body: JSON.stringify(payload),
  });
}

export async function uploadBlend(unitId, file) {
  const form = new FormData();
  form.append("file", file);
  const response = await fetch(`/api/rig/sessions/${unitId}/blend`, {
    method: "PUT",
    body: form,
  });
  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(data.detail ?? response.statusText);
  }
  return data;
}
