import fs from 'node:fs/promises';
import { connect } from './tbg-a1-cdp.mjs';

const PORT = 9333;
const BASE_URL = 'http://127.0.0.1:8131';
const DURATION_MS = 60000;

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
  const taskStart = cpuStart.metrics?.find(m => m.name === 'TaskDuration')?.value || 0;

  // Let it run for durationMs
  await new Promise(r => setTimeout(r, durationMs));

  const cpuEnd = await client.send('Performance.getMetrics').catch(() => ({ metrics: [] }));
  const taskEnd = cpuEnd.metrics?.find(m => m.name === 'TaskDuration')?.value || 0;
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
  console.log('Connected! Running Step 7 40-per-side gate benchmark (60s)...');

  const result = await runScenario(client, 'stress=80&measure', 'docs/media/a1/gate-stress-80.jpg', DURATION_MS);
  console.log('Result for 40-per-side:');
  console.log(JSON.stringify(result, null, 2));

  await fs.writeFile('docs/media/a1/gate-performance.json', JSON.stringify({ '40-per-side': result }, null, 2));
  console.log('Saved gate-performance.json and gate-stress-80.jpg');

  client.close();
}

main().catch(err => {
  console.error('Gate benchmark failed:', err);
  process.exit(1);
});
