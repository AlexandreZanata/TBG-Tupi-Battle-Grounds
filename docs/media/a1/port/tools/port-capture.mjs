// Screenshot helper for the A-1 troop port: node port-capture.mjs <cdpPort> <width> <height> <out.jpg> <url> [waitMs]
import { connect } from '../../tools/tbg-a1-cdp.mjs';

const [port, width, height, out, url, wait = '6000'] = process.argv.slice(2);
const client = await connect(Number(port));
await client.send('Emulation.setDeviceMetricsOverride', {
  width: Number(width), height: Number(height), deviceScaleFactor: 1, mobile: false,
});
await client.send('Page.navigate', { url });
await new Promise(r => setTimeout(r, Number(wait)));
await client.screenshot(out);
const errors = client.events.filter(e => e.method === 'Runtime.exceptionThrown' || (e.method === 'Log.entryAdded' && e.params.entry.level === 'error'));
console.log(out, 'errors:', errors.length, JSON.stringify(errors.map(e => e.params.entry?.text ?? e.params.exceptionDetails?.text)).slice(0, 400));
client.close();
