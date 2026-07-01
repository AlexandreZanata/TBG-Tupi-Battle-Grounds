/** Modular accessories — socket attach (MB-Lab hair uses morph targets). */

const PART_GROUPS = [
  { key: "hair", label: "Hair", primary: "hair_mblab_01" },
  { key: "hats", label: "Hat" },
  { key: "armor", label: "Armor" },
  { key: "weapons", label: "Weapon" },
];

const LEGACY_HAIR = new Set(["hair_long_01", "hair_short_01"]);

export function createPartsState() {
  return { hair: null, hats: null, armor: null, weapons: null };
}

function itemsForGroup(library, group, baseId) {
  const all = (library.parts[group.key] || []).filter((p) => (p.baseCompat || []).includes(baseId));
  if (group.key !== "hair") return all;
  const primary = all.find((p) => p.id === group.primary);
  if (primary) return [primary];
  return all.filter((p) => !LEGACY_HAIR.has(p.id));
}

export function renderPartsPanel(library, baseId, selected, onPick) {
  const host = document.getElementById("partsPanel");
  if (!host || !library?.parts) return;
  host.innerHTML = "";
  for (const group of PART_GROUPS) {
    const items = itemsForGroup(library, group, baseId);
    if (!items.length) continue;
    const title = document.createElement("p");
    title.className = "panel__meta";
    title.textContent = group.label;
    host.appendChild(title);
    const grid = document.createElement("div");
    grid.className = "assembly-parts-grid";
    for (const part of items) {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "assembly-part-card";
      if (selected[group.key] === part.id) btn.classList.add("assembly-part-card--selected");
      btn.textContent = part.id === "hair_mblab_01" ? "MB-Lab hair" : part.id.replace(/_/g, " ");
      btn.addEventListener("click", () => {
        const next = selected[group.key] === part.id ? null : part.id;
        onPick(group.key, next);
      });
      grid.appendChild(btn);
    }
    host.appendChild(grid);
  }
}

export function resolveSelectedParts(library, selected) {
  if (!library?.parts) return [];
  const out = [];
  for (const group of PART_GROUPS) {
    const id = selected[group.key];
    if (!id) continue;
    const part = (library.parts[group.key] || []).find((p) => p.id === id);
    if (part) out.push({ socket: part.socket, meshUrl: part.meshUrl });
  }
  return out;
}

export function selectedPartIds(selected) {
  return PART_GROUPS.map((g) => selected[g.key]).filter(Boolean);
}
