// Screenshot viewer: progressive enhancement over plain image links, using the native <dialog>.

export function initGallery() {
  const viewer = document.querySelector('[data-viewer]');
  const image = viewer?.querySelector('[data-viewer-image]');
  const caption = viewer?.querySelector('[data-viewer-caption]');
  if (!viewer || !image || typeof viewer.showModal !== 'function') return;
  let opener = null;
  for (const link of document.querySelectorAll('[data-gallery] a.shot')) {
    link.addEventListener('click', event => {
      event.preventDefault();
      const thumb = link.querySelector('img');
      opener = link;
      image.src = link.href;
      image.alt = thumb?.alt ?? '';
      caption.textContent = link.closest('figure')?.querySelector('figcaption')?.textContent ?? '';
      viewer.showModal();
    });
  }
  viewer.querySelector('[data-viewer-close]')?.addEventListener('click', () => viewer.close());
  // A click on the backdrop (outside the image panel) closes the viewer too.
  viewer.addEventListener('click', event => {
    if (event.target === viewer) viewer.close();
  });
  viewer.addEventListener('close', () => {
    image.removeAttribute('src');
    opener?.focus();
  });
}
