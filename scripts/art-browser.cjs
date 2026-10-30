//! Desktop art evidence: real lobby/Practice flow and separately labeled component fixtures.
const fs=require('node:fs/promises');
const path=require('node:path');
const assert=require('node:assert/strict');
const {chromium}=require(process.env.ART_PLAYWRIGHT||'playwright');
const root=path.resolve(__dirname,'..');
const media=path.join(root,'docs/media/art');
const origin='http://127.0.0.1:18795';
const report={screenshots:[],checks:[],errors:[],assetFailures:[]};
async function save(page,name){
 for(const quality of [80,70,60,50]){
  const buffer=await page.screenshot({type:'jpeg',quality});
  if(buffer.length>300000)continue;
  await fs.writeFile(path.join(media,name+'.jpg'),buffer);
  report.screenshots.push({name:name+'.jpg',bytes:buffer.length,quality});
  await fs.writeFile(path.join(root,'.local/art-browser.json'),JSON.stringify(report,null,2));return;
 }
 throw new Error(name+' exceeds 300 KB');
}
async function frame(page){await page.evaluate(()=>new Promise(done=>requestAnimationFrame(()=>requestAnimationFrame(done))));}
async function menuEvidence(page){
 const requests=[];page.on('request',r=>requests.push(r.url()));
 await page.goto(origin+'/?server=ws://127.0.0.1:18795/ws&shadows=0');
 await page.locator('#tbg-lobby:not([hidden])[data-screen="lobby"]').waitFor({timeout:90000});
 assert.equal(requests.filter(u=>/\/assets\/(arcana|augments)\/.*\.(json|svg|webp)$/.test(u)).length,0);
 assert.equal(requests.filter(u=>/\/assets\/stronghold\/.*\.webp$/.test(u)).length,0);
 for(const [width,height] of [[1920,1080],[1366,768]]){
  await page.setViewportSize({width,height});await frame(page);
  await page.locator('[data-arcana-open]').first().click();
  await page.locator('.arcana-node[data-node="purification"]').waitFor();
  await page.locator('.arcana-scroll').evaluate(n=>{n.scrollTop=n.scrollHeight;});
  await page.waitForFunction(()=>[...document.querySelectorAll('.arcana-node-child img,.arcana-node-supreme img')].every(i=>i.complete&&i.naturalWidth>0));
  await frame(page);
  assert.equal(await page.locator('.arcana-node-child,.arcana-node-supreme').count(),9);
  await save(page,`arcanas-${width}x${height}`);
  await page.keyboard.press('Escape');
  await page.locator('[data-tower-open]').first().click();
  await page.waitForFunction(()=>document.querySelector('#stronghold-picker').open&&document.querySelectorAll('.tower-portrait').length===4);
  await page.waitForFunction(()=>[...document.querySelectorAll('.tower-portrait,.tower-banner')].every(i=>i.complete&&i.naturalWidth>0));
  await frame(page);await save(page,`strongholds-${width}x${height}`);
  await page.keyboard.press('Escape');
  report.checks.push({viewport:`${width}x${height}`,newArcanaIcons:9,towers:4});
 }
 report.checks.push({lazyBeforeOpen:true});
}
async function practice(page){
 await page.setViewportSize({width:1920,height:1080});
 await page.locator('[data-action="practice"]').first().click();
 await page.locator('#tbg-battle:not([hidden])').waitFor({timeout:60000});
 const cdp=await page.context().newCDPSession(page);
 await cdp.send('Emulation.setDeviceMetricsOverride',{width:1920,height:1080,deviceScaleFactor:0.25,mobile:false});
 await page.evaluate(()=>{
  window.artRecruit=setInterval(()=>{
   const root=document.querySelector('#tbg-battle');
   if(!root||root.hidden||root.dataset.council!=='closed')return;
   const offers=[...root.querySelectorAll('.battle-card[data-available="true"]')];
   const tank=offers.find(b=>(b.querySelector('.champion-role')?.textContent||'').includes('Tank'));
   (tank||offers[0])?.click();
  },500);
 });
 await page.waitForFunction(()=>{
  const root=document.querySelector('#tbg-battle');
  return root&&!root.hidden&&root.dataset.council&&root.dataset.council!=='closed'&&!root.querySelector('[data-pause]').hidden;
 },{},{timeout:240000});
 await page.evaluate(()=>clearInterval(window.artRecruit));
 await cdp.send('Emulation.setDeviceMetricsOverride',{width:1920,height:1080,deviceScaleFactor:1,mobile:false});
 await frame(page);
 await page.waitForFunction(()=>[...document.querySelectorAll('.council-illustration,.council-frame')].every(i=>i.complete&&i.naturalWidth>0));
 for(const [width,height] of [[1920,1080],[1366,768]]){
  await page.setViewportSize({width,height});await frame(page);
  assert.equal(await page.locator('.council-card').count(),4);
  await save(page,`augments-practice-${width}x${height}`);
 }
 const choices=await page.locator('.council-card').evaluateAll(cards=>cards.map(c=>({name:c.querySelector('.council-name').textContent,rarity:c.querySelector('.council-rarity').textContent,disabled:c.disabled})));
 assert.ok(choices.every(c=>['Bronze','Silver','Gold','Diamond'].includes(c.rarity)));
 const available=page.locator('.council-card:not([disabled])').first();
 if(await available.count()){
  await available.click();
  const unchanged=await page.evaluate(()=>{
   const root=document.querySelector('#tbg-battle'),before=root.dataset.energy;
   document.querySelector('[data-battle-action="confirm"]').click();
   return {before,after:root.dataset.energy};
  });
  assert.equal(unchanged.before,unchanged.after);
  report.checks.push({practiceRound:true,choices,energyWaitsForView:true});
 }else report.checks.push({practiceRound:true,choices,allOptionsUnaffordable:true});
}
async function layoutFixture(page){
 const aug=JSON.parse(await fs.readFile(path.join(root,'crates/tbg-client/assets/augments/augments.json'),'utf8'));
 const towers=JSON.parse(await fs.readFile(path.join(root,'crates/tbg-client/assets/stronghold/strongholds.json'),'utf8'));
 await page.route('**/?art-layout-fixture=1',async route=>{
  const response=await route.fetch();
  const html=(await response.text()).replace(/<script type="module">[\s\S]*?<\/script>/,'');
  await route.fulfill({response,body:html});
 });
 await page.goto(origin+'/?art-layout-fixture=1');
 await page.evaluate(({aug,towers})=>{
  const ids=[0,5,20,30], costs=[3,6,10,15];
  const battle=document.querySelector('#tbg-battle'), hp=towers.kinds.find(k=>k.kind===towers.defaultKind).stats.maxHp;
  Object.assign(battle.dataset,{you:'Layout fixture',opponent:'No live match',clock:'1:00',hpOwn:String(hp),hpEnemy:String(hp),hpMaxOwn:String(hp),hpMaxEnemy:String(hp),towerYou:towers.defaultKind+'|0|0',towerEnemy:towers.defaultKind+'|0|0',fortify:'paused|paused',energy:'9000',energyScale:'600',army:'0',regen:'10000',council:'1|'+aug.roundTicks+'|false|'+ids.join(','),councilOptions:ids.map((id,i)=>{const a=aug.augments[id];return [id,a.name,a.text,costs[i]].join(':');}).join('|')});
  battle.hidden=false;
  const label=document.createElement('aside');label.textContent='ART LAYOUT FIXTURE — not a live match';label.style.cssText='position:fixed;top:5px;left:8px;padding:5px 9px;z-index:999;background:#512e18;color:#ffdb9d;font:12px sans-serif';document.body.append(label);
 },{aug,towers});
 await page.waitForFunction(()=>[...document.querySelectorAll('.council-illustration,.council-frame')].every(i=>i.complete&&i.naturalWidth>0));
 for(const [width,height] of [[1920,1080],[1366,768]]){
  await page.setViewportSize({width,height});await frame(page);await save(page,`augments-layout-${width}x${height}`);
 }
 for(const [width,height] of [[1920,1080],[1366,768]]){
 await page.setViewportSize({width,height});await frame(page);
 const contract=await page.evaluate(async aug=>{
  const battle=document.querySelector('#tbg-battle');delete battle.dataset.action;
  const settle=()=>new Promise(done=>requestAnimationFrame(()=>requestAnimationFrame(done)));
  const overflow=[];
  for(let first=0;first<40;first+=4){
   const ids=[first,first+1,first+2,first+3];
   battle.dataset.council=(first+2)+'|200|false|'+ids.join(',');
   battle.dataset.councilOptions=ids.map(id=>{const a=aug.augments[id];return [id,a.name,a.text,[3,6,10,15][aug.rarities.findIndex(r=>r.name===a.rarity)]].join(':');}).join('|');
   await settle();
   for(const card of document.querySelectorAll('.council-card')){
    const rect=selector=>card.querySelector(selector).getBoundingClientRect();
    if(rect('.council-name').bottom>rect('.council-text').top||rect('.council-text').bottom>rect('.council-price').top)overflow.push(card.querySelector('.council-name').textContent);
   }
  }
  const ids=[0,5,20,30];
  battle.dataset.council='99|200|false|'+ids.join(',');
  battle.dataset.councilOptions=ids.map((id,i)=>{const a=aug.augments[id];return [id,a.name,a.text,[3,6,10,15][i]].join(':');}).join('|');
  battle.dataset.energy='1800';await settle();
  const cards=[...document.querySelectorAll('.council-card')],confirm=document.querySelector('[data-battle-action=confirm]');
  const affordable=cards.map(c=>!c.disabled);
  cards[0].click();await settle();
  const pickOnly=cards[0].classList.contains('selected')&&!confirm.disabled&&!battle.dataset.action;
  confirm.click();await settle();
  const intentionOnly=battle.dataset.action==='aug:0'&&battle.dataset.energy==='1800'&&battle.dataset.council.includes('|false|');
  battle.dataset.council='99|200|true|'+ids.join(',');await settle();
  const waitsForView=!document.querySelector('[data-council-wait]').hidden&&document.querySelector('.council-actions').hidden;
  battle.dataset.council='100|100|false|'+ids.join(',');await settle();
  document.querySelector('[data-battle-action=forgo]').click();await settle();
  return {overflow,affordable,pickOnly,intentionOnly,waitsForView,forgo:battle.dataset.action==='forgo',timer:document.querySelector('[data-council-timer]').textContent,progress:document.querySelector('[data-council-progress]').style.width};
 },aug);
 assert.deepEqual(contract.overflow,[]);
 assert.deepEqual(contract.affordable,[true,false,false,false]);
 assert.ok(contract.pickOnly&&contract.intentionOnly&&contract.waitsForView&&contract.forgo);
 assert.equal(contract.timer,'0:05');assert.equal(contract.progress,'25%');
 report.checks.push({viewport:`${width}x${height}`,layoutFixture:true,livePractice:false,fourCanonicalRarities:true,componentContracts:contract});
 }
}
(async()=>{
 await fs.mkdir(media,{recursive:true});
 const browser=await chromium.launch({headless:true,executablePath:process.env.ART_CHROMIUM,args:['--no-sandbox','--enable-unsafe-swiftshader','--use-angle=swiftshader','--ignore-gpu-blocklist']});
 try{
  const page=await browser.newPage({viewport:{width:1920,height:1080},reducedMotion:'reduce'});
  page.on('pageerror',e=>report.errors.push(e.message));
  page.on('response',r=>{if(r.status()>=400&&/\/assets\/(arcana|augments|stronghold)\//.test(r.url()))report.assetFailures.push(r.status()+' '+r.url());});
  if(process.argv[2]==='fixture') await layoutFixture(page);
  else{
  if(process.argv[2]==='practice'){
   await page.goto(origin+'/?server=ws://127.0.0.1:18795/ws&shadows=0');
   await page.locator('#tbg-lobby:not([hidden])[data-screen="lobby"]').waitFor({timeout:90000});
  }else await menuEvidence(page);
  const progress=setInterval(async()=>{try{console.log('PRACTICE',await page.locator('#tbg-battle').evaluate(n=>({hidden:n.hidden,clock:n.dataset.clock,council:n.dataset.council,energy:n.dataset.energy})));}catch(_){}},15000);
  try{await practice(page);}finally{clearInterval(progress);}
  }
  assert.deepEqual(report.errors,[]);assert.deepEqual(report.assetFailures,[]);
  console.log(JSON.stringify(report,null,2));
  await fs.writeFile(path.join(root,'.local/art-browser.json'),JSON.stringify(report,null,2)+'\n');
 }catch(error){
  const pages=browser.contexts().flatMap(c=>c.pages());
  if(pages[0]){console.log('FAILURE STATE',await pages[0].locator('#tbg-battle').evaluate(n=>({...n.dataset,hidden:n.hidden})));await pages[0].screenshot({path:path.join(root,'.local/browser-failure.jpg'),type:'jpeg',quality:70});}
  throw error;
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
