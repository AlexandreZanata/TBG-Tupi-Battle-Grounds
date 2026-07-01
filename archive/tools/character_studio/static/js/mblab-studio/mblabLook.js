/** Skin / gear color pickers + library material presets. */

const DEFAULT_SKIN = "#b8a99e";
const DEFAULT_GEAR = "#8c8578";

function rgbToHex(factor) {
  const clamp = (v) => Math.round(Math.max(0, Math.min(1, v)) * 255);
  const r = clamp(factor[0]).toString(16).padStart(2, "0");
  const g = clamp(factor[1]).toString(16).padStart(2, "0");
  const b = clamp(factor[2]).toString(16).padStart(2, "0");
  return `#${r}${g}${b}`;
}

export function renderLookPanel(library, { materialId, onChange, onMaterial }) {
  const host = document.getElementById("lookPanel");
  if (!host) return;
  host.innerHTML = "";

  const skinRow = document.createElement("label");
  skinRow.className = "mblab-field mblab-color-field";
  skinRow.innerHTML = "<span>Skin tone</span>";
  const skinInput = document.createElement("input");
  skinInput.type = "color";
  skinInput.id = "skinColor";
  skinInput.value = DEFAULT_SKIN;
  skinInput.addEventListener("input", onChange);
  skinRow.appendChild(skinInput);
  host.appendChild(skinRow);

  const gearRow = document.createElement("label");
  gearRow.className = "mblab-field mblab-color-field";
  gearRow.innerHTML = "<span>Gear color</span>";
  const gearInput = document.createElement("input");
  gearInput.type = "color";
  gearInput.id = "gearColor";
  gearInput.value = DEFAULT_GEAR;
  gearInput.addEventListener("input", onChange);
  gearRow.appendChild(gearInput);
  host.appendChild(gearRow);

  const presets = library?.materials || [];
  if (!presets.length) return;
  const title = document.createElement("p");
  title.className = "panel__meta";
  title.textContent = "Presets";
  host.appendChild(title);
  const grid = document.createElement("div");
  grid.className = "assembly-parts-grid";
  for (const mat of presets) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "assembly-part-card";
    if (mat.id === materialId) btn.classList.add("assembly-part-card--selected");
    btn.textContent = mat.label || mat.id;
    btn.addEventListener("click", () => onMaterial(mat));
    grid.appendChild(btn);
  }
  host.appendChild(grid);
}

export function applyMaterialPreset(mat) {
  const body = mat?.profile?.body?.baseColorFactor;
  const parts = mat?.profile?.parts?.baseColorFactor;
  const skin = document.getElementById("skinColor");
  const gear = document.getElementById("gearColor");
  if (body && skin) skin.value = rgbToHex(body);
  if (parts && gear) gear.value = rgbToHex(parts);
}

export function readLook() {
  return {
    skinHex: document.getElementById("skinColor")?.value || DEFAULT_SKIN,
    gearHex: document.getElementById("gearColor")?.value || DEFAULT_GEAR,
  };
}
