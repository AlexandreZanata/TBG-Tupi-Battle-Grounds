/** Mixamo Edit API helpers. */

export const STATUS_URL = "/api/assembly/mixamo-edit/status";
export const MANIFEST_URL = "/api/assembly/mixamo-edit/manifest";
export const BUILD_URL = "/api/assembly/mixamo-edit/build";
export const UPLOAD_URL = "/api/assembly/mixamo-edit/upload-mesh";
export const CONTENT = "/api/content/characters/imports/mixamo";
export const TEST_WALK_FBX = "Walking.fbx";

export function fbxUrl(name) {
  return `${CONTENT}/raw/${encodeURIComponent(name)}`;
}

/** Turn FastAPI `detail` (string | array | object) into readable text. */
export function formatApiDetail(detail) {
  if (!detail) return "";
  if (typeof detail === "string") return detail;
  if (Array.isArray(detail)) {
    return detail.map((e) => e?.msg || JSON.stringify(e)).join("; ");
  }
  if (typeof detail === "object" && detail.msg) return String(detail.msg);
  try {
    return JSON.stringify(detail);
  } catch {
    return "Request failed";
  }
}

export async function pollJob(jobId, onProgress) {
  for (let i = 0; i < 120; i += 1) {
    await new Promise((r) => setTimeout(r, 1500));
    const job = await fetch(`/api/assembly/mixamo-edit/jobs/${jobId}`).then((r) => r.json());
    if (job.status === "running") {
      onProgress?.("Blender joining models…");
      continue;
    }
    if (job.status === "failed") throw new Error(formatApiDetail(job.error) || job.error || "Join failed");
    return job.report;
  }
  throw new Error("Timed out");
}

export async function uploadSkin(unitId, file) {
  const form = new FormData();
  form.append("file", file);
  const res = await fetch(`${UPLOAD_URL}?unitId=${encodeURIComponent(unitId)}`, { method: "POST", body: form });
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error(formatApiDetail(body.detail) || "Upload failed");
  }
  return (await res.json()).customMesh;
}

export function renderFbxSelect(files) {
  const sel = document.getElementById("mixamoFbx");
  sel.innerHTML = "";
  for (const name of files) {
    const opt = document.createElement("option");
    opt.value = name;
    opt.textContent = name;
    sel.appendChild(opt);
  }
  const pick = files.find((f) => f === "Y Bot.fbx") || files[0];
  if (pick) sel.value = pick;
}

export function triggerDownload(url, filename) {
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
}

export async function exportProjectGlb(project) {
  if (!project?.meshUrl) throw new Error("No GLB for this project");
  const url = `${project.meshUrl}?t=${Date.now()}`;
  const res = await fetch(url);
  if (!res.ok) throw new Error("GLB file not found — re-run Join or pick another project");
  triggerDownload(url, `${project.id}.glb`);
}

export async function removeProject(unitId) {
  const res = await fetch(`/api/assembly/mixamo-edit/projects/${encodeURIComponent(unitId)}`, {
    method: "DELETE",
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error(formatApiDetail(body.detail) || "Remove failed");
  }
  return res.json();
}

export function syncProjectActionButtons(projects, selectedId) {
  const project = projects.find((p) => p.id === selectedId);
  document.getElementById("btnExportProject").disabled = !project?.meshUrl;
  document.getElementById("btnRemoveProject").disabled = !project;
}

export function renderProjects(projects, selectedId, onPick) {
  const list = document.getElementById("projectList");
  list.innerHTML = "";
  for (const p of projects) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "manual-list__item";
    if (p.id === selectedId) btn.classList.add("manual-list__item--active");
    btn.textContent = p.label || p.id;
    btn.addEventListener("click", () => onPick(p));
    list.appendChild(btn);
  }
  syncProjectActionButtons(projects, selectedId);
}

export function renderClips(names) {
  const sel = document.getElementById("clipSelect");
  sel.innerHTML = "";
  if (!names?.length) {
    sel.disabled = true;
    sel.innerHTML = "<option value=\"\">—</option>";
    return;
  }
  sel.disabled = false;
  for (const name of names) {
    const opt = document.createElement("option");
    opt.value = name;
    opt.textContent = name;
    sel.appendChild(opt);
  }
}
