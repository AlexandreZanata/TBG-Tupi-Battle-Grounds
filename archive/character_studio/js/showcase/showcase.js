import { createShowcaseCarousel, fetchShowcaseCharacters } from "./showcaseCarousel.js";
import { createShowcaseModal } from "./showcaseModal.js";

let carousel = null;
let modal = null;

async function init() {
  const characters = await fetchShowcaseCharacters();
  const indicator = document.getElementById("slideIndicator");

  modal = createShowcaseModal(
    {
      modal: document.getElementById("showcaseModal"),
      backdrop: document.getElementById("modalBackdrop"),
      closeBtn: document.getElementById("modalClose"),
      prevBtn: document.getElementById("modalPrev"),
      nextBtn: document.getElementById("modalNext"),
      viewerHost: document.getElementById("modalViewer"),
      loadingEl: document.getElementById("modalLoading"),
      titleEl: document.getElementById("modalTitle"),
      subtitleEl: document.getElementById("modalSubtitle"),
      counterEl: document.getElementById("modalCounter"),
    },
    {
      characters,
      onOpen: () => carousel?.pauseViewers(),
      onClose: () => carousel?.resumeViewers(),
    },
  );

  carousel = createShowcaseCarousel({
    trackEl: document.getElementById("showcaseTrack"),
    onOpenCharacter: (id) => modal.open(id),
    onSlideChange: (current, total) => {
      indicator.textContent = `${current} / ${total}`;
      document.getElementById("btnPrev").disabled = current <= 1;
      document.getElementById("btnNext").disabled = current >= total;
    },
  });
  carousel.render(characters);

  document.getElementById("btnPrev").addEventListener("click", () => carousel.goPrev());
  document.getElementById("btnNext").addEventListener("click", () => carousel.goNext());

  const viewport = document.getElementById("showcaseViewport");
  let touchStartX = 0;
  viewport.addEventListener("touchstart", (event) => {
    touchStartX = event.changedTouches[0].screenX;
  }, { passive: true });
  viewport.addEventListener("touchend", (event) => {
    const delta = event.changedTouches[0].screenX - touchStartX;
    if (Math.abs(delta) < 48) return;
    if (delta < 0) carousel.goNext();
    else carousel.goPrev();
  }, { passive: true });
}

init().catch((error) => {
  document.getElementById("showcaseTrack").innerHTML = `<p class="showcase-toolbar__hint">${error.message}</p>`;
});
