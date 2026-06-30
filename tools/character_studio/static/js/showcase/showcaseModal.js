import { mountCharacterViewer } from "../viewer/viewer3d.js";
import { probeAsset } from "../viewer/gltfLoader.js";

const MODAL_DISPLAY_SCALE = 1.3;

export function createShowcaseModal(elements, options = {}) {
  const {
    modal,
    backdrop,
    closeBtn,
    prevBtn,
    nextBtn,
    viewerHost,
    loadingEl,
    titleEl,
    subtitleEl,
    counterEl,
  } = elements;
  const characters = options.characters ?? [];
  let viewer = null;
  let currentIndex = 0;
  let switching = false;

  function updateMeta(index) {
    const character = characters[index];
    if (!character) return;
    titleEl.textContent = character.characterName;
    subtitleEl.textContent = `${character.archetype} · ${character.faction} · ${character.id}`;
    if (counterEl) counterEl.textContent = `${index + 1} / ${characters.length}`;
    prevBtn.disabled = characters.length <= 1;
    nextBtn.disabled = characters.length <= 1;
  }

  function ensureViewer() {
    if (viewer) return viewer;
    viewer = mountCharacterViewer(viewerHost, {
      stageVariant: "light",
      displayScale: MODAL_DISPLAY_SCALE,
      maxPixelRatio: 1.25,
      onLoading: (_character, meta) => {
        if (meta?.cached) {
          loadingEl.hidden = true;
          return;
        }
        loadingEl.hidden = false;
        loadingEl.textContent = "Loading 3D…";
      },
      onProgress: (message) => {
        loadingEl.hidden = false;
        loadingEl.textContent = message;
      },
      onReady: () => {
        loadingEl.hidden = true;
        viewerHost.classList.add("is-ready");
      },
      onError: (error) => {
        loadingEl.hidden = false;
        loadingEl.textContent = error?.message || "3D preview failed";
      },
    });
    viewer.setAutoRotate(true);
    return viewer;
  }

  function close() {
    viewer?.dispose();
    viewer = null;
    modal.classList.remove("is-open");
    modal.hidden = true;
    loadingEl.hidden = true;
    viewerHost.classList.remove("is-ready");
    options.onClose?.();
  }

  async function showAt(index) {
    if (!characters.length || switching) return;
    switching = true;
    currentIndex = (index + characters.length) % characters.length;
    const character = characters[currentIndex];
    updateMeta(currentIndex);

    const probe = await probeAsset(`/api/characters/${character.id}/mesh`);
    if (probe.cached) loadingEl.hidden = true;

    try {
      await ensureViewer().loadCharacter(character);
    } catch (error) {
      loadingEl.hidden = false;
      loadingEl.textContent = error?.message || "3D preview failed";
    } finally {
      switching = false;
    }
  }

  async function open(characterId) {
    const index = characters.findIndex((entry) => entry.id === characterId);
    if (index < 0) return;
    options.onOpen?.();
    modal.hidden = false;
    requestAnimationFrame(() => modal.classList.add("is-open"));
    await showAt(index);
  }

  function goNext() {
    showAt(currentIndex + 1);
  }

  function goPrev() {
    showAt(currentIndex - 1);
  }

  backdrop.addEventListener("click", close);
  closeBtn.addEventListener("click", close);
  prevBtn.addEventListener("click", (event) => {
    event.stopPropagation();
    goPrev();
  });
  nextBtn.addEventListener("click", (event) => {
    event.stopPropagation();
    goNext();
  });

  window.addEventListener("keydown", (event) => {
    if (modal.hidden) return;
    if (event.key === "Escape") close();
    if (event.key === "ArrowRight") goNext();
    if (event.key === "ArrowLeft") goPrev();
  });

  return { open, close, goNext, goPrev };
}
