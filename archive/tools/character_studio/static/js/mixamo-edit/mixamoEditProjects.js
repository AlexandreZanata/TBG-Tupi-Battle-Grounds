/** Export / remove joined Mixamo Edit projects. */

import { MANIFEST_URL, exportProjectGlb, removeProject } from "./mixamoEditApi.js";

export async function exportActiveProject(project, setStatus) {
  if (!project) throw new Error("Select a project first");
  setStatus(`Exporting ${project.id}.glb…`);
  await exportProjectGlb(project);
  setStatus(`Download started · ${project.label || project.id}`);
}

export async function removeActiveProject(project, setStatus) {
  if (!project) throw new Error("Select a project first");
  const label = project.label || project.id;
  if (!window.confirm(`Remove "${label}" and delete its GLB files?`)) return null;
  setStatus(`Removing ${label}…`);
  await removeProject(project.id);
  setStatus(`Removed ${label}`);
  return fetch(MANIFEST_URL).then((r) => r.json());
}

export function bindProjectActions({ getSelected, setStatus, onRemoved }) {
  document.getElementById("btnExportProject")?.addEventListener("click", () => {
    void exportActiveProject(getSelected(), setStatus).catch((err) => {
      setStatus(err.message || "Export failed");
    });
  });
  document.getElementById("btnRemoveProject")?.addEventListener("click", () => {
    void (async () => {
      const project = getSelected();
      try {
        const next = await removeActiveProject(project, setStatus);
        if (next) onRemoved(project, next);
      } catch (err) {
        setStatus(err.message || "Remove failed");
      }
    })();
  });
}
