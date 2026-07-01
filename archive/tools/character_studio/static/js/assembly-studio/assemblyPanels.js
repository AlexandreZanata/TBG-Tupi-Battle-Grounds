/** Assembly Studio step panels and workflow UI. */

const PART_SLOTS = {
  heads: "head",
  hair: "hair",
  armor: "armor",
  weapons: "weapon",
  hats: "hat",
};

export function createPanelRenderer(ctx) {
  const { STEPS, state, setStatus, refreshPreview, runExport } = ctx;

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
      li.addEventListener("click", () => ctx.goStep(i));
      list.appendChild(li);
    });
  }

  function partCard(item, selected, onPick) {
    const card = document.createElement("button");
    card.type = "button";
    card.className = "assembly-part-card";
    if (selected) card.classList.add("assembly-part-card--selected");
    const thumb = item.thumbnailUrl
      ? `<img class="assembly-part-card__thumb" src="${item.thumbnailUrl}" alt="" />`
      : `<div class="assembly-part-card__thumb"></div>`;
    card.innerHTML = `${thumb}${item.id}`;
    card.addEventListener("click", onPick);
    return card;
  }

  function renderCategoryGrid(panel, category) {
    const items = (state.library?.parts?.[category] || []).filter((p) =>
      !state.baseId || p.baseCompat.includes(state.baseId),
    );
    const slot = PART_SLOTS[category];
    panel.innerHTML = `<p class="rig-lead">${category}</p><div class="assembly-parts-grid" id="grid"></div>`;
    const grid = panel.querySelector("#grid");
    items.forEach((item) => {
      grid.appendChild(
        partCard(item, state.parts[slot] === item.id, () => {
          state.parts[slot] = state.parts[slot] === item.id ? null : item.id;
          renderPanel();
          refreshPreview();
          setStatus(`${category}: ${state.parts[slot] || "none"}`);
        }),
      );
    });
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
      (state.library?.bases || []).forEach((b) => {
        grid.appendChild(
          partCard({ id: b.id, thumbnailUrl: null }, state.baseId === b.id, () => {
            state.baseId = b.id;
            state.parts = {};
            renderPanel();
            refreshPreview();
            setStatus(`Base: ${b.label}`);
          }),
        );
      });
      return;
    }

    if (step.id === "head" || step.id === "parts") {
      panel.innerHTML = "";
      step.categories.forEach((cat) => {
        const section = document.createElement("div");
        panel.appendChild(section);
        renderCategoryGrid(section, cat);
      });
      return;
    }

    if (step.id === "materials") {
      panel.innerHTML = `<div class="assembly-parts-grid" id="matGrid"></div>`;
      const grid = document.getElementById("matGrid");
      (state.library?.materials || []).forEach((m) => {
        grid.appendChild(
          partCard({ id: m.id, thumbnailUrl: null }, state.materialId === m.id, () => {
            state.materialId = m.id;
            renderPanel();
            setStatus(`Material: ${m.label || m.id}`);
          }),
        );
      });
      return;
    }

    if (step.id === "review") {
      const parts = Object.values(state.parts)
        .filter(Boolean)
        .map((id) => `<li>${id}</li>`)
        .join("") || "<li>none</li>";
      panel.innerHTML = `<ul class="rig-lead"><li>Base: ${state.baseId || "—"}</li><li>Material: ${state.materialId}</li>${parts}</ul>`;
      return;
    }

    panel.innerHTML = `
      <p class="rig-lead">Blender merges your selection and exports a game-ready GLB.</p>
      <div class="field"><label for="unitId">Unit id</label>
      <input id="unitId" class="input" placeholder="demo_guard_male_01" autocomplete="off" value="${state.unitId}" /></div>
      <div class="assembly-export-bar"><button class="btn btn--primary" id="btnExport" type="button">Export via Blender</button></div>
      <p id="exportStatus" class="panel__meta">Ready</p>`;
    document.getElementById("unitId")?.addEventListener("input", (e) => {
      state.unitId = e.target.value.trim();
    });
    document.getElementById("btnExport")?.addEventListener("click", () => void runExport());
  }

  return { renderSteps, renderPanel };
}
