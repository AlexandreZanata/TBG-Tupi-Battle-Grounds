// Click helper for the A-1 troop port: node port-click.mjs <cdpPort> <out.jpg> <waitMs> <x,y> [<x,y> ...]
// Reuses the open page (no navigation) so a flow can be driven step by step.
import { connect } from '../../tools/tbg-a1-cdp.mjs';

const [port, out, wait, ...points] = process.argv.slice(2);
const client = await connect(Number(port));
await client.send('Emulation.setDeviceMetricsOverride', { width: 1440, height: 900, deviceScaleFactor: 1, mobile: false });
for (const p of points) {
  const [x, y] = p.split(',').map(Number);
  for (const type of ['mouseMoved', 'mousePressed', 'mouseReleased']) {
    await client.send('Input.dispatchMouseEvent', { type, x, y, button: 'left', clickCount: 1 });
  }
  await new Promise(r => setTimeout(r, 700));
}
await new Promise(r => setTimeout(r, Number(wait)));
await client.screenshot(out);
console.log(out);
client.close();
