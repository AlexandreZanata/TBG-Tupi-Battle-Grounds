// DOM driver for the A-1 troop port: node port-ui.mjs <cdpPort> <out.jpg> <waitMs> <js...>
// Each js argument is evaluated in the page in order (HTML UI buttons are clicked by text).
import { connect } from '../../tools/tbg-a1-cdp.mjs';

const [port, out, wait, ...steps] = process.argv.slice(2);
const client = await connect(Number(port));
await client.send('Emulation.setDeviceMetricsOverride', { width: Number(process.env.W || 1440), height: Number(process.env.H || 900), deviceScaleFactor: 1, mobile: false });
for (const js of steps) {
  const result = await client.evaluate(js);
  if (result !== undefined) console.log(JSON.stringify(result));
  await new Promise(r => setTimeout(r, 700));
}
await new Promise(r => setTimeout(r, Number(wait)));
if (out !== '-') await client.screenshot(out);
console.log(out);
client.close();
