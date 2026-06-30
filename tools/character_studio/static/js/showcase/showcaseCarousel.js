import { mountCharacterViewer } from "../viewer/viewer3d.js";
import { SHOWCASE_ORDER } from "./showcaseOrder.js";

function chunkPairs(characters) {
  const slides = [];
  for (let i = 0; i < characters.length; i += 2) {
    slides.push(characters.slice(i, i + 2));
  }
  return slides;
}

function slideMarkup(pair, slideIndex) {
  const single = pair.length === 1 ? " showcase-slide--single" : "";
  const cards = pair
    .map(
      (character) => `
    <article class="showcase-card" data-id="${character.id}" tabindex="0" role="button">
      <div class="showcase-card__viewer" id="viewer-${character.id}">
        <div class="showcase-card__loading">Waiting…</div>
      </div>
      <div class="showcase-card__body">
        <h3 class="showcase-card__name">${character.characterName}</h3>
        <p class="showcase-card__meta">${character.archetype} · ${character.faction}</p>
      </div>
    </article>`,
    )
    .join("");

  return `<div class="showcase-slide${single}" data-slide="${slideIndex}">${cards}</div>`;
}

export function createShowcaseCarousel({ trackEl, onOpenCharacter, onSlideChange }) {
  let slides = [];
  let slideIndex = 0;
  let activeViewers = [];

  function disposeViewers() {
    activeViewers.forEach((entry) => entry.dispose());
    activeViewers = [];
  }

  function mountSlideViewers(index) {
    disposeViewers();
    const slide = trackEl.querySelector(`[data-slide="${index}"]`);
    if (!slide) return;

    slide.querySelectorAll(".showcase-card").forEach((card) => {
      const id = card.dataset.id;
      const host = card.querySelector(".showcase-card__viewer");
      const loading = host.querySelector(".showcase-card__loading");
      const character = slides.flat().find((entry) => entry.id === id);
      if (!character) return;

      loading.textContent = "Loading 3D…";
      const viewer = mountCharacterViewer(host, {
        stageVariant: "light",
        maxPixelRatio: 1.25,
        onLoading: (_c, meta) => {
          if (meta?.cached) loading.hidden = true;
        },
        onProgress: (message) => {
          loading.hidden = false;
          loading.textContent = message;
        },
        onReady: () => {
          loading.hidden = true;
        },
        onError: (error) => {
          loading.hidden = false;
          loading.textContent = error?.message || "Preview failed";
        },
      });
      viewer.setAutoRotate(true);
      viewer.loadCharacter(character);
      activeViewers.push(viewer);
    });
  }

  function render(characters) {
    slides = chunkPairs(characters);
    trackEl.innerHTML = slides.map((pair, index) => slideMarkup(pair, index)).join("");
    trackEl.querySelectorAll(".showcase-card").forEach((card) => {
      const open = () => onOpenCharacter(card.dataset.id);
      card.addEventListener("click", open);
      card.addEventListener("keydown", (event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          open();
        }
      });
    });
    goTo(0);
  }

  function goTo(index) {
    slideIndex = Math.max(0, Math.min(index, slides.length - 1));
    trackEl.style.transform = `translateX(-${slideIndex * 100}%)`;
    mountSlideViewers(slideIndex);
    onSlideChange?.(slideIndex + 1, slides.length);
  }

  return {
    render,
    goTo,
    goNext: () => goTo(slideIndex + 1),
    goPrev: () => goTo(slideIndex - 1),
    pauseViewers: disposeViewers,
    resumeViewers: () => mountSlideViewers(slideIndex),
    destroy: disposeViewers,
  };
}

export async function fetchShowcaseCharacters() {
  const list = await fetch("/api/characters").then((response) => response.json());
  const byId = new Map(
    await Promise.all(
      list
        .filter((entry) => entry.id.startsWith("demo_"))
        .map(async (entry) => {
          const detail = await fetch(`/api/characters/${entry.id}`).then((r) => r.json());
          return [entry.id, detail];
        }),
    ),
  );
  return SHOWCASE_ORDER.filter((id) => byId.has(id)).map((id) => byId.get(id));
}
