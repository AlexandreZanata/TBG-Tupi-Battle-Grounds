// Opt-in browser measurements: count actual WebGL draws, including shadow passes.
if (['measure','perf','bench'].some(key=>new URLSearchParams(location.search).has(key))) {
  window.addEventListener("TrunkApplicationStarted", e => { window.__tbgWasm = e.detail?.wasm; });
  const output = document.createElement('output');
  output.id = 'tbg-measure';
  output.style.cssText = 'position:fixed;bottom:8px;left:8px;font:14px monospace;color:#2b2118;background:#efe3c6;padding:4px;z-index:20';
  document.addEventListener('DOMContentLoaded', () => document.body.append(output));
  const getContext = HTMLCanvasElement.prototype.getContext;
  const seen = new WeakSet();
  let triangles = 0, totalTriangles = 0, draws = 0, frames = 0, total = 0, maximum = 0, firstFrame = 0, start = performance.now();
  const frameHistory = [];
  let lastTime = performance.now();
  window.__tbgFrames = frameHistory;

  HTMLCanvasElement.prototype.getContext = function (...args) {
    const gl = getContext.apply(this, args);
    if (gl && /^webgl/.test(args[0]) && !seen.has(gl)) {
      seen.add(gl);
      output.dataset.renderer = gl.getParameter(gl.RENDERER);
      for (const key of ['drawArrays', 'drawElements', 'drawArraysInstanced', 'drawElementsInstanced']) {
        const draw = gl[key].bind(gl);
        gl[key] = (...parameters) => {
          draws++;
          const count = parameters[key.startsWith('drawArrays') ? 2 : 1];
          const instances = key.endsWith('Instanced') ? parameters[key.startsWith('drawArrays') ? 3 : 4] : 1;
          const primitiveTriangles = parameters[0] === gl.TRIANGLES ? Math.floor(count / 3) : [gl.TRIANGLE_STRIP,gl.TRIANGLE_FAN].includes(parameters[0]) ? Math.max(0,count-2) : 0;
          triangles += primitiveTriangles * instances;
          return draw(...parameters);
        };
      }
    }
    return gl;
  };
  function measure(now) {
    const dt = now - lastTime;
    lastTime = now;
    if (dt > 0 && dt < 1000) { frameHistory.push(dt); }
    if (draws && !firstFrame) firstFrame = now;
    if (draws) { frames++; total += draws; totalTriangles += triangles; maximum = Math.max(maximum, draws); }
    draws = triangles = 0;
    if (now - start >= 5000) {
      const fps = (1000 * frames / (now - start)).toFixed(1);
      const average = (total / Math.max(1, frames)).toFixed(1);
      output.textContent = `FPS ${fps} | draws ${average} (max ${maximum})`;
      Object.assign(output.dataset, { fps, draws: average, maximum: String(maximum), frames: String(frames),
        triangles: String(Math.round(totalTriangles/Math.max(1,frames))),
        elapsedMs: now.toFixed(1), firstFrameMs: firstFrame.toFixed(1), visibility: document.visibilityState,
        // Chromium's JS heap is a partial memory measurement, not total browser/WASM/GPU memory.
        heapUsedBytes: String(performance.memory?.usedJSHeapSize || 0),
        heapTotalBytes: String(performance.memory?.totalJSHeapSize || 0) });
      frames = total = maximum = totalTriangles = 0; start = now;
    }
    requestAnimationFrame(measure);
  }
  requestAnimationFrame(measure);

  window.__getTbgMetrics = () => {
    const sorted = [...frameHistory].sort((a,b) => a - b);
    const n = sorted.length;
    const p50 = n ? sorted[Math.floor(n * 0.50)] : 0;
    const p95 = n ? sorted[Math.floor(n * 0.95)] : 0;
    const p99 = n ? sorted[Math.floor(n * 0.99)] : 0;
    const sum = sorted.reduce((a, b) => a + b, 0);
    const avg = n ? sum / n : 0;
    const fps = avg > 0 ? 1000 / avg : 0;
    const el = document.querySelector('#tbg-measure');
    const jsHeapBytes = performance.memory ? performance.memory.usedJSHeapSize : 0;
    const wasmMemoryBytes = window.__tbgWasm?.memory?.buffer?.byteLength || 0;
    return {
      sampleCount: n,
      fps: Number(fps.toFixed(1)),
      p50: Number(p50.toFixed(2)),
      p95: Number(p95.toFixed(2)),
      p99: Number(p99.toFixed(2)),
      max: n ? sorted.reduce((maximum,value)=>Math.max(maximum,value),0) : 0,
      triangles: Number(el?.dataset.triangles || 0),
      renderer: el?.dataset.renderer || "",
      viewport: [innerWidth,innerHeight],
      dpr: devicePixelRatio,
      entities: Number(el?.dataset.entities || 0),
      meshes: Number(el?.dataset.meshes || 0),
      materials: Number(el?.dataset.materials || 0),
      draws: Number(el?.dataset.draws || 0),
      jsHeapBytes,
      wasmMemoryBytes,
    };
  };
}

// Fixed 3-second warmup and 20-second sample; export data for a browser harness.
if (new URLSearchParams(location.search).has('bench')) {
  const waitForFrames = () => {
    if (!window.__getTbgMetrics || !Number(document.querySelector('#tbg-measure')?.dataset.draws)) { setTimeout(waitForFrames,250); return; }
    setTimeout(()=>{window.__tbgFrames.length=0;setTimeout(()=>{
      window.__tbgBench={...window.__getTbgMetrics(),units:Number(new URLSearchParams(location.search).get('bench')),warmupSeconds:3,sampleSeconds:20};
      console.info('TBG_BENCH',JSON.stringify(window.__tbgBench));
    },20000);},3000);
  };
  addEventListener('DOMContentLoaded',waitForFrames);
}
