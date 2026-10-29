import fs from 'node:fs/promises';
import {connect} from './tbg-a1-cdp.mjs';
const c = await connect(9333);
await c.send('Page.bringToFront');
await c.send('Emulation.setDeviceMetricsOverride',{width:1280,height:800,deviceScaleFactor:1,mobile:false});
c.events.length=0;
await c.send('Page.navigate',{url:'http://127.0.0.1:8131/?audit&measure'});
let started = false;
let playing = false;
const logs=[];
let offset=0;
for(let i=0;i<720;i++) {
  await new Promise(resolve=>setTimeout(resolve,500));
  const state=await c.evaluate(`(()=>{
    const lobby=document.querySelector('#tbg-lobby');
    const battle=document.querySelector('#tbg-battle');
    if(lobby&&!lobby.hidden) {lobby.querySelector('[data-action="practice"]').click();return 'starting';}
    if(!battle||battle.hidden)return 'off';
    const cards=[...battle.querySelectorAll('.battle-card')];
    const card=cards.find(card=>card.dataset.available==='true' && /^(Rampart|Sawbot),/.test(card.getAttribute('aria-label')||''));
    if(card) {const label=card.getAttribute('aria-label');card.click();return label;}
    return 'playing';
  })()`);
  if(state==='playing'||/^(Rampart|Sawbot),/.test(state))playing=true;
  if(state!=='playing'&&state!=='off') console.log(state);
  for(const e of c.events.slice(offset)) {
    if(e.method==='Runtime.consoleAPICalled') logs.push({time:e.params.timestamp,text:e.params.args.map(a=>a.value??a.description??'').join(' ')});
    if(e.method==='Runtime.exceptionThrown')console.log(JSON.stringify(e.params));
  }
  offset=c.events.length;
  if(i%20===0) {
    await fs.writeFile('/tmp/a1-targeted-audit.json',JSON.stringify(logs,null,2));
    console.log(JSON.stringify({seconds:i/2,logs:logs.length,state}));
  }
  if(playing&&state==='off')break;
}
await fs.writeFile('/tmp/a1-targeted-audit.json',JSON.stringify(logs,null,2));
await c.screenshot('/tmp/a1-targeted-result.jpg');c.close();
