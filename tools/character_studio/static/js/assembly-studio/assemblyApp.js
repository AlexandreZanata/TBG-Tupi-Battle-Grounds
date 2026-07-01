/** Assembly Studio — modular character builder (Phase 05). */

const STEPS = [
  { id: "body", label: "Body" },
  { id: "head", label: "Head" },
  { id: "parts", label: "Parts" },
  { id: "materials", label: "Materials" },
  { id: "review", label: "Review" },
  { id: "export", label: "Export" },
];

const BASES = [
  { id: "male", label: "Male" },
  { id: "female", label: "Female" },
  { id: "small", label: "Small" },
  { id: "large", label: "Large" },
];

const state = {
  step: 0,
  baseId: null,
  parts: {},
};

function renderSteps() {
  const list = document.getElementById("stepList");
  if (!list) return;
  list.innerHTML = "";
  STEPS.forEach((step, i) => {
    const li = document.createElement("li");
    li.className = "assembly-steps__item";
    if (i === state.step) li.classList.add("assembly-steps__item--active");
    if (i < state.step) li.classList.add("assembly-steps__item--done");
    li.innerHTML = `<span class="assembly-steps__num">${i + 1}</span>${step.label}`;
    li.addEventListener("click", () => goStep(i));
    list.appendChild(li);
  });
}

function goStep(index) {
  state.step = Math.max(0, Math.min(index, STEPS.length - 1));
  renderSteps();
  renderPanel();
}

function renderPanel() {
  const panel = document.getElementById("stepPanel");
  const title = document.getElementById("stepTitle");
  if (!panel || !title) return;
  const step = STEPS[state.step];
  title.textContent = step.label;

  if (step.id === "body") {
    panel.innerHTML = `<div class="assembly-parts-grid" id="baseGrid"></div>`;
    const grid = document.getElementById("baseGrid");
    BASES.forEach((b) => {
      const card = document.createElement("button");
      card.type = "button";
      card.className = "assembly-part-card";
      if (state.baseId === b.id) card.classList.add("assembly-part-card--selected");
      card.innerHTML = `<div class="assembly-part-card__thumb"></div>${b.label}`;
      card.addEventListener("click", () => {
        state.baseId = b.id;
        renderPanel();
        setStatus(`Base: ${b.label}`);
      });
      grid.appendChild(card);
    });
    return;
  }

  if (step.id === "export") {
    panel.innerHTML = `
      <p class="rig-lead">Blender merges your selection and exports a game-ready GLB.</p>
      <div class="field">
        <label for="unitId">Unit id</label>
        <input id="unitId" class="input" placeholder="tank_human_03" autocomplete="off" />
      </div>
      <div class="assembly-export-bar">
        <button class="btn btn--primary" id="btnExport" type="button">Export via Blender</button>
      </div>
      <p id="exportStatus" class="panel__meta">API not wired — Phase 05 §4</p>`;
    document.getElementById("btnExport")?.addEventListener("click", () => {
      setStatus("Export queued (stub)");
    });
    return;
  }

  panel.innerHTML = `<p class="rig-lead">Load parts from <code>content/characters/library/</code> — Phase 05 §2.</p>`;
}

function setStatus(msg) {
  const el = document.getElementById("assemblyStatus");
  if (el) el.textContent = msg;
}

async function checkHealth() {
  const badge = document.getElementById("health");
  try {
    const res = await fetch("/api/health");
    const data = await res.json();
    if (badge) {
      badge.textContent = data.status === "ok" ? "online" : "degraded";
      badge.classList.toggle("badge--ok", data.status === "ok");
    }
  } catch {
    if (badge) badge.textContent = "offline";
  }
}

function initViewport() {
  const host = document.getElementById("viewport");
  if (!host) return;
  host.innerHTML = `<p class="panel__meta" style="padding:2rem;text-align:center">3D preview — wire viewer3d in §4</p>`;
}

export function initAssemblyStudio() {
  renderSteps();
  renderPanel();
  initViewport();
  checkHealth();
  document.getElementById("btnNext")?.addEventListener("click", () => goStep(state.step + 1));
  document.getElementById("btnPrev")?.addEventListener("click", () => goStep(state.step - 1));
}

document.addEventListener("DOMContentLoaded", () => {
  initAssemblyStudio();
});
