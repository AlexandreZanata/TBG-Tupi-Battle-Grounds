/** Assembly Studio API client. */

export async function fetchLibrary() {
  const res = await fetch("/api/assembly/library");
  if (!res.ok) throw new Error(`library ${res.status}`);
  return res.json();
}

export async function queueExport(recipe) {
  const res = await fetch("/api/assembly/export", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(recipe),
  });
  if (!res.ok) {
    const detail = await res.json().catch(() => ({}));
    throw new Error(detail.detail ? JSON.stringify(detail.detail) : `export ${res.status}`);
  }
  return res.json();
}

export async function pollJob(jobId) {
  const res = await fetch(`/api/assembly/jobs/${jobId}`);
  if (!res.ok) throw new Error(`job ${res.status}`);
  return res.json();
}

export function recipeFromState(state) {
  const parts = [];
  for (const value of Object.values(state.parts)) {
    if (value) parts.push(value);
  }
  return {
    unitId: state.unitId || "assembled_unit_01",
    baseId: state.baseId,
    parts,
    materialId: state.materialId || "default",
  };
}
