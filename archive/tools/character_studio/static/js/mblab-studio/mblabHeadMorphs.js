/** Head shape sliders — subset of mblab_studio_morphs.json */

const CONFIG_URL = "/api/content/characters/library/mblab_studio_morphs.json";

export async function loadStudioMorphConfig() {
  const res = await fetch(CONFIG_URL);
  if (!res.ok) throw new Error("studio morph config missing");
  return res.json();
}

export function headSliders(config) {
  return (config?.sliders || []).filter((s) => s.mblab);
}

export function renderHeadSliders(config, onChange) {
  const host = document.getElementById("headMorphPanel");
  if (!host) return;
  host.innerHTML = "";
  for (const slider of headSliders(config)) {
    const wrap = document.createElement("div");
    wrap.className = "mblab-slider";
    const label = document.createElement("label");
    label.htmlFor = slider.id;
    label.textContent = slider.label;
    const input = document.createElement("input");
    input.type = "range";
    input.id = slider.id;
    input.min = "0";
    input.max = "100";
    input.value = "50";
    const out = document.createElement("output");
    out.id = `${slider.id}Out`;
    const sync = () => {
      out.textContent = (Number(input.value) / 100).toFixed(2);
      onChange();
    };
    input.addEventListener("input", sync);
    wrap.append(label, input, out);
    host.appendChild(wrap);
    sync();
  }
}

export function readHeadMorphs(config) {
  const morphs = {};
  for (const slider of headSliders(config)) {
    const el = document.getElementById(slider.id);
    if (el) morphs[slider.id] = Number(el.value) / 100;
  }
  return morphs;
}
