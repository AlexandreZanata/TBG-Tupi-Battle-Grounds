// Spectator exit: Watch from the lobby panel, press Leave, and expect the lobby, a clean URL
// and the server counting no watcher. Needs a server with a public match running.
// Optional tool: needs Playwright (TBG_PLAYWRIGHT: module name or path) and a local Chrome.
// Usage: node scripts/spectate-leave.cjs <origin> <out dir> [query]  e.g. http://127.0.0.1:8080 /tmp/out
// Start the server on port 8080 (the page connects there) and run `tbg-bots` first so a match is listed.
const {chromium} = require(process.env.TBG_PLAYWRIGHT || 'playwright');
const origin = process.argv[2], out = process.argv[3], extra = process.argv[4] || '';
(async () => {
  const browser = await chromium.launch({channel: 'chrome', headless: true, args: ['--headless=new', '--enable-gpu', '--ignore-gpu-blocklist', '--use-angle=gl']});
  const page = await browser.newPage({viewport: {width: 1920, height: 1080}});
  const errors = []; page.on('pageerror', e => errors.push(e.message));
  await page.goto(origin + '/' + extra, {waitUntil: 'load'});
  await page.locator('#tbg-lobby:not([hidden])[data-screen="lobby"]').waitFor({timeout: 90000});
  await page.locator('[data-live-watch]').first().waitFor({timeout: 120000});
  await page.locator('[data-live-watch]').first().click();
  await page.waitForURL(/watch=[0-9a-f]{16}/, {timeout: 30000});
  await page.locator('#tbg-battle:not([hidden])').waitFor({timeout: 90000});
  await page.waitForTimeout(4000);
  const watching = async () => (await (await fetch(origin + '/api/live')).json()).online.watching;
  console.log('while watching: watchers =', await watching(), '| url =', page.url().replace(origin, ''));
  await page.screenshot({path: `${out}/1-watching.jpg`, type: 'jpeg', quality: 70});
  await page.locator('.battle-leave').click();
  const inLobby = await page.locator('#tbg-lobby:not([hidden])[data-screen="lobby"]').waitFor({timeout: 8000}).then(() => true, () => false);
  await page.waitForTimeout(1500);
  const url = page.url().replace(origin, '');
  console.log('after Leave: lobby shown =', inLobby, '| url =', url, '| battle hidden =', await page.locator('#tbg-battle').evaluate(n => n.hidden), '| watchers =', await watching(), '| stored watch =', await page.evaluate(() => sessionStorage.getItem('tbg_watch')));
  await page.screenshot({path: `${out}/2-after-leave.jpg`, type: 'jpeg', quality: 70});
  await browser.close();
  const ok = inLobby && !/watch=/.test(url);
  console.log(ok ? 'RESULT: PASS (left the match)' : 'RESULT: FAIL (still spectating)', errors.length ? '| page errors: ' + errors.join(' | ') : '');
  process.exit(ok ? 0 : 1);
})().catch(e => { console.error(e.message); process.exit(2); });
