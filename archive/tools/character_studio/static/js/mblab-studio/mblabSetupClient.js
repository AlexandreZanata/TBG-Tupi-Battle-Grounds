/** MB-Lab pipeline setup polling for the studio UI. */

export async function pollMblabJob(jobId, onStatus) {
  for (let i = 0; i < 240; i += 1) {
    const job = await fetch(`/api/assembly/mblab/jobs/${jobId}`).then((r) => r.json());
    if (job.status === "running") onStatus("Blender setup…");
    if (job.status === "completed") return job;
    if (job.status === "failed") throw new Error(job.error || "Setup failed");
    await new Promise((r) => setTimeout(r, 500));
  }
  throw new Error("Setup timed out");
}

export async function ensureMblabPipeline(statusUrl, setupUrl, onStatus) {
  const status = await fetch(statusUrl).then((r) => r.json());
  if (status.libraryBasesReady) return status;
  onStatus("Setting up MB-Lab…");
  const res = await fetch(setupUrl, { method: "POST" });
  if (!res.ok) throw new Error("Setup failed to start");
  await pollMblabJob((await res.json()).id, onStatus);
  return fetch(statusUrl).then((r) => r.json());
}
