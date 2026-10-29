import fs from 'node:fs/promises';
import {connect} from './tbg-a1-cdp.mjs';

const client = await connect(9333);
await client.send('Emulation.setDeviceMetricsOverride', {width:1280, height:800, deviceScaleFactor:1, mobile:false});
client.events.length = 0;
await client.send('Page.navigate', {url:'http://127.0.0.1:8131/?autoplay=practice&audit&measure'});
const started = Date.now();
let offset = 0;
let snapshots = 0;
const kinds = new Set();
const lines = [];
let finished = false;
for (let second = 0; second < 900 && !finished; second++) {
  await new Promise(resolve => setTimeout(resolve, 1000));
  let arrow = false;
  for (const event of client.events.slice(offset)) {
    if (event.method === 'Runtime.consoleAPICalled') {
      const text = event.params.args.map(arg => arg.value ?? arg.description ?? '').join(' ');
      lines.push({elapsed_ms:Date.now()-started, type:event.params.type, text});
      const kind = text.match(/A1_REACH .*kind=(\w+)/)?.[1];
      if (kind) kinds.add(kind);
      if (text.includes('A1_ARROW')) arrow = true;
      if (/panicked|unreachable|ERROR/.test(text)) console.log(text.slice(0,1500));
    }
    if (event.method === 'Runtime.exceptionThrown') {
      lines.push({elapsed_ms:Date.now()-started, exception:event.params});
      console.log(JSON.stringify(event.params).slice(0,1500));
    }
  }
  offset = client.events.length;
  if (arrow && snapshots < 12) {
    await client.screenshot(`/tmp/a1-audit-arrow-${snapshots++}.jpg`);
  }
  if (second % 30 === 0) {
    await fs.writeFile('/tmp/a1-audit-gpu-browser.json', JSON.stringify(lines,null,2));
    console.log(JSON.stringify({elapsed_s:second, observed_kinds:[...kinds], log_entries:lines.length, screenshots:snapshots}));
  }
}
await fs.writeFile('/tmp/a1-audit-gpu-browser.json', JSON.stringify(lines,null,2));
await client.screenshot('/tmp/a1-audit-final.jpg');
client.close();
