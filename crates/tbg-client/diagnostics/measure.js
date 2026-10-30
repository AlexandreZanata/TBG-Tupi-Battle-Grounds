// Opt-in browser measurements: count actual WebGL draws, including shadow passes.
if (new URLSearchParams(location.search).has('measure')) {
  const output = document.createElement('output');
  output.id = 'tbg-measure';
  output.style.cssText = 'position:fixed;bottom:8px;left:8px;font:14px monospace;color:#2b2118;background:#efe3c6;padding:4px;z-index:20';
  document.addEventListener('DOMContentLoaded', () => document.body.append(output));
  const getContext = HTMLCanvasElement.prototype.getContext;
  const seen = new WeakSet();
  let draws = 0, frames = 0, total = 0, maximum = 0, firstFrame = 0, start = performance.now();
  HTMLCanvasElement.prototype.getContext = function (...args) {
    const gl = getContext.apply(this, args);
    if (gl && /^webgl/.test(args[0]) && !seen.has(gl)) {
      seen.add(gl);
      output.dataset.renderer = gl.getParameter(gl.RENDERER);
      for (const key of ['drawArrays', 'drawElements', 'drawArraysInstanced', 'drawElementsInstanced']) {
        const draw = gl[key].bind(gl);
        gl[key] = (...parameters) => { draws++; return draw(...parameters); };
      }
    }
    return gl;
  };
  function measure(now) {
    if (draws && !firstFrame) firstFrame = now;
    if (draws) { frames++; total += draws; maximum = Math.max(maximum, draws); }
    draws = 0;
    if (now - start >= 5000) {
      const fps = (1000 * frames / (now - start)).toFixed(1);
      const average = (total / Math.max(1, frames)).toFixed(1);
      output.textContent = `FPS ${fps} | draws ${average} (max ${maximum})`;
      Object.assign(output.dataset, { fps, draws: average, maximum: String(maximum), frames: String(frames),
        elapsedMs: now.toFixed(1), firstFrameMs: firstFrame.toFixed(1), visibility: document.visibilityState,
        // Chromium's JS heap is a partial memory measurement, not total browser/WASM/GPU memory.
        heapUsedBytes: String(performance.memory?.usedJSHeapSize || 0),
        heapTotalBytes: String(performance.memory?.totalJSHeapSize || 0) });
      frames = total = maximum = 0; start = now;
    }
    requestAnimationFrame(measure);
  }
  requestAnimationFrame(measure);
}
