/** Showcase vs game mesh tier state for Animation Studio. */

export function createMeshTierToggle(container, onChange) {
  let tier = "game";

  container.innerHTML = `
    <div class="mesh-tier-toggle" role="group" aria-label="Mesh tier">
      <button type="button" class="mesh-tier-toggle__btn" data-tier="showcase">Showcase</button>
      <button type="button" class="mesh-tier-toggle__btn is-active" data-tier="game">Game</button>
    </div>
  `;

  const buttons = container.querySelectorAll("[data-tier]");

  function setTier(next) {
    tier = next === "showcase" ? "showcase" : "game";
    buttons.forEach((btn) => btn.classList.toggle("is-active", btn.dataset.tier === tier));
    onChange?.(tier);
  }

  buttons.forEach((btn) => {
    btn.addEventListener("click", () => setTier(btn.dataset.tier));
  });

  return {
    getTier: () => tier,
    setTier,
  };
}
