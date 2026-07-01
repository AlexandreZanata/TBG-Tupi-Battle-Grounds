/** Bounded WebGL viewport sizing — prevents canvas height feedback loops. */

const MAX_VIEWPORT_HEIGHT = 2048;

export function prepareViewerRoot(container) {
  container.classList.add("viewer3d-root");
  let mount = container.querySelector(".viewer3d-mount");
  if (!mount) {
    mount = document.createElement("div");
    mount.className = "viewer3d-mount";
    container.appendChild(mount);
  }
  return mount;
}

export function readViewerSize(container) {
  const rect = container.getBoundingClientRect();
  const width = Math.max(1, Math.floor(rect.width));
  const height = Math.max(1, Math.min(Math.floor(rect.height), MAX_VIEWPORT_HEIGHT));
  return { width, height };
}

export function applyRendererSize(renderer, width, height, maxPixelRatio = 2) {
  const ratio = Math.min(window.devicePixelRatio || 1, maxPixelRatio);
  renderer.setPixelRatio(ratio);
  renderer.setSize(width, height, false);
  const canvas = renderer.domElement;
  canvas.style.display = "block";
  canvas.style.width = "100%";
  canvas.style.height = "100%";
}

export function bindViewerResize(container, renderer, camera, onResize) {
  let lastWidth = 0;
  let lastHeight = 0;

  const update = () => {
    const { width, height } = readViewerSize(container);
    if (width === lastWidth && height === lastHeight) return;
    lastWidth = width;
    lastHeight = height;
    applyRendererSize(renderer, width, height);
    camera.aspect = width / height;
    camera.updateProjectionMatrix();
    onResize?.(width, height);
  };

  const observer = new ResizeObserver(() => update());
  observer.observe(container);
  update();
  return observer;
}
