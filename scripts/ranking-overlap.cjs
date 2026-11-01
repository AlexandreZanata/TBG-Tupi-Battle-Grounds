// Ranking view: nothing from the lobby home may sit on top of it, and its close control must be clickable.
// Optional tool: needs Playwright (TBG_PLAYWRIGHT: module name or path) and a local Chrome.
// Usage: node scripts/ranking-overlap.cjs <origin> <out dir>  e.g. http://127.0.0.1:8080 /tmp/out
// Needs the server on port 8080 (the page connects there); the live panel must exist on the lobby page.
const {chromium} = require(process.env.TBG_PLAYWRIGHT || 'playwright');
const origin = process.argv[2], out = process.argv[3];
(async () => {
  const browser = await chromium.launch({channel: 'chrome', headless: true, args: ['--headless=new', '--enable-gpu', '--ignore-gpu-blocklist', '--use-angle=gl']});
  const page = await browser.newPage({viewport: {width: 1920, height: 1080}});
  const errors = []; page.on('pageerror', e => errors.push(e.message));
  await page.goto(origin + '/', {waitUntil: 'load'});
  await page.locator('#tbg-lobby:not([hidden])[data-screen="lobby"]').waitFor({timeout: 90000});
  await page.locator('[data-live-panel]').waitFor({timeout: 30000});
  await page.waitForTimeout(1500);
  await page.locator('.lobby-tabs [data-action="ranking"]').click();
  await page.locator('#tbg-ranking:not([hidden])').waitFor({timeout: 30000});
  await page.waitForTimeout(2500);
  const probe = await page.evaluate(() => {
    const shown = sel => { const n = document.querySelector(sel); if (!n) return 'absent'; const r = n.getBoundingClientRect(); return getComputedStyle(n).display !== 'none' && r.width > 0 && r.height > 0; };
    const close = document.querySelector('#tbg-ranking .ranking-close, #tbg-ranking [data-ranking-close], #tbg-ranking [aria-label*="lose"]');
    let top = null, covered = null;
    if (close) { const r = close.getBoundingClientRect(); const el = document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2); covered = !(el === close || close.contains(el)); top = el ? (el.className || el.tagName) : null; }
    return {homePanelsVisible: {live: shown('.live-panel'), mode: shown('.mode-panel'), profile: shown('.profile-panel'), cards: shown('.feature-cards'), news: shown('.news-panel')}, closeFound: !!close, closeCovered: covered, elementOnTop: top};
  });
  console.log(JSON.stringify(probe));
  await page.screenshot({path: `${out}/ranking.jpg`, type: 'jpeg', quality: 70});
  let closed = false;
  if (probe.closeFound && !probe.closeCovered) { await page.locator('#tbg-ranking .ranking-close').click(); await page.waitForTimeout(800); closed = await page.evaluate(() => document.getElementById('tbg-ranking').hidden); }
  console.log('closed with its own control:', closed, errors.length ? '| page errors: ' + errors.join(' | ') : '');
  await browser.close();
  const ok = probe.homePanelsVisible.live === false && probe.closeCovered === false && closed;
  console.log(ok ? 'RESULT: PASS' : 'RESULT: FAIL (the live panel overlaps the Ranking page)');
  process.exit(ok ? 0 : 1);
})().catch(e => { console.error(e.message); process.exit(2); });
