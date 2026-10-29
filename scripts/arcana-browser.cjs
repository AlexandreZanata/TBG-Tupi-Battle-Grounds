// Repeatable local-browser acceptance and JPEG evidence for the Arcana preview.
// Optional tool: install Playwright locally; set ARCANA_PLAYWRIGHT and ARCANA_CHROMIUM.
// Start tbg-server with TBG_STATIC_DIR=crates/tbg-client/dist first.
const fs = require('node:fs/promises');
const path = require('node:path');
const assert = require('node:assert/strict');
const {chromium} = require(process.env.ARCANA_PLAYWRIGHT || 'playwright');
const origin = process.env.TBG_PREVIEW_URL || 'http://127.0.0.1:18747';
const root = path.resolve(__dirname, '..');
const media = path.join(root, 'docs/media/arcana-ui');
const report = {screenshots:[], checks:[], errors:[], failedRequests:[]};
const requested = process.argv[2] || 'all';
const reportPath = path.join(root,'.local/arcana-browser-'+requested.replace(/[^a-z0-9-]/g,'-')+'.json');
const checkpoint = () => fs.writeFile(reportPath,JSON.stringify(report,null,2)+'\n');

async function screenshot(page, name) {
  for (const quality of [80, 70, 60, 50]) {
    const buffer = await page.screenshot({type:'jpeg', quality});
    if (buffer.length <= 300000) {
      await fs.writeFile(path.join(media, name + '.jpg'), buffer);
      report.screenshots.push({name:name+'.jpg', bytes:buffer.length, quality});
      return;
    }
  }
  throw new Error(name + ' exceeds the JPEG budget');
}
async function menu(page) {
  await page.goto(origin+'/?shadows=0', {waitUntil:'load'});
  await page.locator('#tbg-lobby:not([hidden])[data-screen="lobby"]').waitFor({timeout:60000});
}
async function open(page, entry='.lobby-tabs [data-arcana-open]') {
  await page.locator(entry).click();
  await page.waitForFunction(() => document.querySelector('#arcana-editor').open && document.querySelector('[data-a="default"]').textContent.startsWith('Default'));
  await page.waitForFunction(() => [...document.querySelectorAll('#arcana-editor img')].every(i => i.complete && i.naturalWidth > 0));
  await page.evaluate(() => document.fonts.ready);
}
async function ranks(page) {
  return page.locator('.arcana-node').evaluateAll(nodes => nodes.map(n => Number(n.dataset.rank)));
}
async function desktop(browser, width, height) {
  const context = await browser.newContext({viewport:{width,height}, reducedMotion:'reduce'});
  const page = await context.newPage();
  page.on('pageerror', error => report.errors.push(error.message));
  page.on('response', r => { if (r.status() >= 400) report.failedRequests.push(r.status()+' '+r.url()); });
  const requests=[]; page.on('request', request => requests.push(request.url()));
  await menu(page);
  assert.equal(requests.filter(url => /\/assets\/arcana\//.test(url)).length, 0);
  await open(page);
  const reset=page.locator('[data-a="reset"]'), defaults=page.locator('[data-a="default"]');
  await reset.click(); assert.deepEqual(await ranks(page), [0,0,0,0,0,0]);
  await screenshot(page, `editor-${width}x${height}-empty`);
  await defaults.click(); assert.deepEqual(await ranks(page), [2,0,2,0,2,0]);
  await screenshot(page, `editor-${width}x${height}-default`);
  const guard=page.locator('[data-node="guard"]');
  await guard.locator('.arcana-node-main').click({button:'right'});
  assert.deepEqual(await ranks(page), [1,0,2,0,2,0]);
  assert.match(await page.locator('[data-a="status"]').textContent(), /Partial preset/);
  await screenshot(page, `editor-${width}x${height}-partial`);
  const scroll=await page.locator('.arcana-scroll').evaluate(n => ({content:n.scrollHeight,visible:n.clientHeight}));
  assert.ok(scroll.content <= scroll.visible + 1, 'All six nodes and eligibility text fit the desktop panel: '+JSON.stringify(scroll));
  if(width===1280){report.checks.push({viewport:`${width}x${height}`,lazyArt:true,emptyDefaultPartial:true,panel:scroll});await checkpoint();await context.close();return;}
  const editRequests=requests.length;
  await reset.click();
  const main=guard.locator('.arcana-node-main');
  await main.focus(); await main.press('Enter'); await main.press('Space');
  assert.equal((await ranks(page))[0], 2);
  await main.press('Equal'); assert.equal((await ranks(page))[0], 2);
  await main.press('-'); assert.equal((await ranks(page))[0], 1);
  await guard.locator('.arcana-affected').click({button:'right'}); assert.equal((await ranks(page))[0], 0);
  await guard.locator('.arcana-affected').click(); assert.equal((await ranks(page))[0], 1);
  await main.focus(); await main.press('Equal'); assert.equal((await ranks(page))[0], 2);
  await defaults.click();
  await page.locator('[data-node="assault"] .arcana-node-main').click();
  assert.deepEqual(await ranks(page), [2,0,2,0,2,0]);
  assert.ok(await page.locator('[data-node="assault"] .arcana-node-tools button').last().isDisabled());
  assert.equal(requests.length, editRequests, 'Editing sends no HTTP request');
  assert.equal(await page.locator('#tbg-lobby').getAttribute('data-action'), null);
  await reset.click(); await main.click();
  await menu(page); await open(page, '.feature[data-arcana-open]');
  assert.deepEqual(await ranks(page), [1,0,0,0,0,0]);
  await page.keyboard.press('Escape');
  assert.equal(await page.locator('#arcana-editor').evaluate(d=>d.open), false);
  assert.ok(await page.locator('.feature[data-arcana-open]').evaluate(n=>n===document.activeElement));
  await page.evaluate(()=>localStorage.setItem('tbg.arcana.preview', '{bad JSON'));
  await menu(page); await open(page);
  assert.deepEqual(await ranks(page), [2,0,2,0,2,0]);
  assert.match(await page.locator('[data-a="storage"]').textContent(), /invalid/);
  await page.evaluate(()=>{
    const write=Storage.prototype.setItem;
    Storage.prototype.setItem=function(key,value){ if(key==='tbg.arcana.preview') throw new DOMException('Denied','SecurityError'); return write.call(this,key,value); };
  });
  await reset.click(); await main.click();
  assert.deepEqual(await ranks(page), [1,0,0,0,0,0]);
  assert.match(await page.locator('[data-a="storage"]').textContent(), /storage unavailable/);
  await page.keyboard.press('Tab');
  assert.ok(await page.locator('#arcana-editor').evaluate(d=>d.contains(document.activeElement)));
  await page.setViewportSize({width:390,height:844});
  await page.waitForFunction(()=>!document.querySelector('#arcana-editor').open);
  await page.locator('#tbg-lobby').waitFor({state:'hidden'});
  await page.waitForTimeout(1200);
  if(width===1920) await screenshot(page,'phone-390x844-menu');
  report.checks.push({viewport:`${width}x${height}`,lazyArt:true,entries:2,emptyDefaultPartial:true,keyboard:true,rightClick:true,budgetCap:true,persistence:true,invalidStorage:true,storageDenied:true,focusReturn:true,phoneCloses:true,noEditRequests:true,panel:scroll});
  await checkpoint();
  await context.close();
}
async function artGallery(browser) {
  const data=JSON.parse(await fs.readFile(path.join(root,'crates/tbg-client/assets/arcana/arcana.json'),'utf8'));
  const page=await browser.newPage({viewport:{width:1200,height:520},deviceScaleFactor:1});
  for(const [name,items,columns,size,height] of [
    ['emblems-close-up',data.families.map(f=>({name:f.name,asset:f.emblem})),3,320,520],
    ['nodes-close-up',data.nodes.map(n=>({name:n.name,asset:n.asset})),3,230,660]
  ]) {
    await page.setViewportSize({width:1200,height});
    await page.goto(origin+'/assets/arcana/arcana.json');
    await page.evaluate(({items,columns,size,origin})=>{
      document.body.replaceChildren();
      document.body.style.cssText='margin:0;padding:25px;box-sizing:border-box;background:#11212a;color:#e9d5ae;font:20px Georgia;text-align:center';
      const title=document.createElement('h1'); title.textContent='Arcana art · '+(size===320?'Families':'Nodes');title.style.cssText='font-size:24px;letter-spacing:2px';document.body.append(title);
      const grid=document.createElement('main');grid.style.cssText=`display:grid;grid-template-columns:repeat(${columns},1fr);gap:14px`;document.body.append(grid);
      for(const item of items){const figure=document.createElement('figure');figure.style.margin='0';const image=document.createElement('img');image.width=size;image.height=size;image.src=origin+'/assets/arcana/'+item.asset.display;image.alt=item.name;const caption=document.createElement('figcaption');caption.textContent=item.name;figure.append(image,caption);grid.append(figure);}
    },{items,columns,size,origin});
    await page.waitForFunction(()=>[...document.images].every(i=>i.complete&&i.naturalWidth));
    await screenshot(page,name); await checkpoint();
  }
  await page.close();
}
async function gallery(browser,width,height) {
  const page=await browser.newPage({viewport:{width,height}});
  await page.goto(origin+'/?gallery&shadows=0');
  await page.waitForFunction(()=>Boolean(window.wasmBindings)&&document.querySelector('#tbg-lobby')?.dataset.screen==='off'&&document.querySelector('#bevy')?.width>0);
  await page.waitForTimeout(4000);
  await screenshot(page,`gallery-${width}x${height}`);
  await checkpoint();
  await page.close();
}
(async()=>{
  await fs.mkdir(media,{recursive:true});
  const browser=await chromium.launch({headless:true,executablePath:process.env.ARCANA_CHROMIUM,args:['--no-sandbox','--enable-unsafe-swiftshader','--use-angle=swiftshader','--ignore-gpu-blocklist']});
  try {
    if(['all','desktop-1920'].includes(requested)) await desktop(browser,1920,1080);
    if(['all','desktop-1280'].includes(requested)) await desktop(browser,1280,800);
    if(['all','art'].includes(requested)) await artGallery(browser);
    if(['all','gallery-1920'].includes(requested)) await gallery(browser,1920,1080);
    if(['all','gallery-phone'].includes(requested)) await gallery(browser,390,844);
    assert.deepEqual(report.errors,[]); assert.deepEqual(report.failedRequests,[]);
    console.log(JSON.stringify(report,null,2));
    await checkpoint();
  } finally { await browser.close(); }
})().catch(error=>{console.error(error);process.exitCode=1;});
