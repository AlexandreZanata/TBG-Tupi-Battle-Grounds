import fs from 'node:fs/promises';
import { connect } from './tbg-a1-cdp.mjs';

const PORT = 9333;
const BASE_URL = 'http://127.0.0.1:8131';
const DURATION_MS = 60000;

function median(arr) {
  if (!arr.length) return 0;
  const s = [...arr].sort((a, b) => a - b);
  const mid = Math.floor(s.length / 2);
  return s.length % 2 !== 0 ? s[mid] : (s[mid - 1] + s[mid]) / 2;
}

async function runScenario(client, query, screenshotPath, durationMs = DURATION_MS) {
  await client.send('Page.bringToFront');
  await client.send('Emulation.setDeviceMetricsOverride', {
    width: 1280,
    height: 800,
    deviceScaleFactor: 1,
    mobile: false,
  });

  client.events.length = 0;
  await client.send('Page.navigate', { url: `${BASE_URL}/?${query}` });

  // Wait for initial load
  await new Promise(r => setTimeout(r, 2000));

  const cpuStart = await client.send('Performance.getMetrics').catch(() => ({ metrics: [] }));
  const taskStart = cpuStart.metrics.find(m => m.name === 'TaskDuration')?.value || 0;

  // Let it run for durationMs
  await new Promise(r => setTimeout(r, durationMs));

  const cpuEnd = await client.send('Performance.getMetrics').catch(() => ({ metrics: [] }));
  const taskEnd = cpuEnd.metrics.find(m => m.name === 'TaskDuration')?.value || 0;
  const cpuTaskSecs = Math.max(0, taskEnd - taskStart);

  let metrics = null;
  try {
    metrics = await client.evaluate(`typeof window.__getTbgMetrics === 'function' ? window.__getTbgMetrics() : null`);
  } catch (err) {
    console.error('Failed to get metrics:', err);
  }

  if (screenshotPath) {
    await client.screenshot(screenshotPath);
  }

  return {
    ...metrics,
    cpuTaskSecs: Number(cpuTaskSecs.toFixed(2)),
  };
}

async function main() {
  console.log('Connecting to Chrome CDP on port', PORT);
  const client = await connect(PORT);
  console.log('Connected! Starting Step 2 baseline benchmark...');

  // 1. Aim lab baseline screenshot
  console.log('Running Aim Lab baseline (?gallery=aim)...');
  await runScenario(client, 'gallery=aim', 'docs/media/a1/baseline/baseline-aim.jpg', 6000);
  console.log('Saved docs/media/a1/baseline/baseline-aim.jpg');

  const scenarios = [
    { name: '10_per_side', query: 'stress=20&measure', screenshot: 'docs/media/a1/baseline/baseline-stress-20.jpg' },
    { name: '40_per_side', query: 'stress=80&measure', screenshot: 'docs/media/a1/baseline/baseline-stress-80.jpg' },
    { name: '100_per_side', query: 'stress=200&measure', screenshot: 'docs/media/a1/baseline/baseline-stress-200.jpg' },
    { name: '200_per_side', query: 'stress=400&measure', screenshot: 'docs/media/a1/baseline/baseline-stress-400.jpg' },
    { name: 'autoplay_practice', query: 'autoplay=practice&measure', screenshot: 'docs/media/a1/baseline/baseline-autoplay.jpg' },
  ];

  const results = {};

  for (const sc of scenarios) {
    console.log(`\n========================================`);
    console.log(`Scenario: ${sc.name} (?${sc.query}) - 3 runs of 60s`);
    console.log(`========================================`);
    const runs = [];
    for (let r = 1; r <= 3; r++) {
      console.log(`Run ${r}/3 for ${sc.name}...`);
      const shot = r === 1 ? sc.screenshot : null;
      const data = await runScenario(client, sc.query, shot, DURATION_MS);
      console.log(`  Run ${r} result:`, JSON.stringify(data));
      runs.push(data);
    }

    const med = {
      fps: median(runs.map(r => r.fps || 0)),
      p50: median(runs.map(r => r.p50 || 0)),
      p95: median(runs.map(r => r.p95 || 0)),
      p99: median(runs.map(r => r.p99 || 0)),
      jsHeapMB: Number((median(runs.map(r => r.jsHeapBytes || 0)) / (1024 * 1024)).toFixed(2)),
      wasmMemMB: Number((median(runs.map(r => r.wasmMemoryBytes || 0)) / (1024 * 1024)).toFixed(2)),
      entities: Math.round(median(runs.map(r => r.entities || 0))),
      meshes: Math.round(median(runs.map(r => r.meshes || 0))),
      materials: Math.round(median(runs.map(r => r.materials || 0))),
      cpuTaskSecs: Number(median(runs.map(r => r.cpuTaskSecs || 0)).toFixed(2)),
      draws: Math.round(median(runs.map(r => r.draws || 0))),
    };

    console.log(`Median for ${sc.name}:`, JSON.stringify(med));
    results[sc.name] = { runs, median: med };
  }

  await fs.writeFile(
    'docs/media/a1/baseline/baseline-performance.json',
    JSON.stringify(results, null, 2),
  );
  console.log('\nResults saved to docs/media/a1/baseline/baseline-performance.json');

  client.close();
}

main().catch(err => {
  console.error('Benchmark failed:', err);
  process.exit(1);
});
