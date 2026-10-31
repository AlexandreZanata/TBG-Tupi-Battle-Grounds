// Long-lived CDP session for the A-1 troop port: node port-drive.mjs <cdpPort> <httpPort> <width> <height>
// Keeps the device-metrics override alive (it vanishes on detach) so the desktop shell stays active.
// POST /eval (body = JS), POST /shot?path=out.jpg, POST /goto (body = url).
import http from 'node:http';
import { connect } from '../../tools/tbg-a1-cdp.mjs';

const [cdp, listen, width, height] = process.argv.slice(2).map(Number);
const client = await connect(cdp);
const apply = () => client.send('Emulation.setDeviceMetricsOverride', { width, height, deviceScaleFactor: 1, mobile: false });
await apply();
await client.send('Emulation.setFocusEmulationEnabled', { enabled: true });
await client.send('Page.bringToFront');
http.createServer(async (req, res) => {
  let body = '';
  for await (const chunk of req) body += chunk;
  const url = new URL(req.url, 'http://x');
  try {
    let out = 'ok';
    if (url.pathname === '/eval') out = JSON.stringify(await client.evaluate(body));
    else if (url.pathname === '/shot') await client.screenshot(url.searchParams.get('path'));
    else if (url.pathname === '/goto') await client.send('Page.navigate', { url: body });
    else if (url.pathname === '/size') { await client.send('Emulation.setDeviceMetricsOverride', { width: Number(url.searchParams.get('w')), height: Number(url.searchParams.get('h')), deviceScaleFactor: 1, mobile: url.searchParams.get('m') === '1' }); }
    else if (url.pathname === '/errors') out = JSON.stringify(client.events.filter(e => e.method === 'Runtime.exceptionThrown' || (e.method === 'Log.entryAdded' && e.params.entry.level === 'error')).map(e => e.params.entry?.text ?? e.params.exceptionDetails?.text).slice(-10));
    res.end(out);
  } catch (e) { res.statusCode = 500; res.end(String(e)); }
}).listen(listen, '127.0.0.1');
console.log('driver ready');
