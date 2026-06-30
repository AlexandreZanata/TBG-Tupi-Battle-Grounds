/** Mesh budget badge rendering for Animation Studio. */

export function formatBytes(size) {
  if (size >= 1_048_576) return `${(size / 1_048_576).toFixed(2)} MB`;
  if (size >= 1024) return `${(size / 1024).toFixed(1)} KB`;
  return `${size} B`;
}

export function renderMeshBadge(container, info) {
  if (!info) {
    container.innerHTML = "";
    return;
  }

  if (info.tier === "showcase") {
    container.innerHTML = `<span class="mesh-badge__item">Showcase · ${formatBytes(info.size)}</span>`;
    return;
  }

  const budgetClass = info.budgetOk ? "mesh-badge__item--ok" : "mesh-badge__item--warn";
  container.innerHTML = [
    `<span class="mesh-badge__item ${budgetClass}">${info.budgetOk ? "Budget OK" : "Budget warn"}</span>`,
    `<span class="mesh-badge__item">${info.triangles} tris</span>`,
    `<span class="mesh-badge__item">${info.bones} bones</span>`,
    `<span class="mesh-badge__item">${info.animations} clips</span>`,
    `<span class="mesh-badge__item">${formatBytes(info.size)}</span>`,
  ].join("");
}
