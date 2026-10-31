// Shop portraits: captures each buyable unit's base form in the gallery portrait
// mode (?gallery=<Kind>&portrait) and writes a 400x400 WebP to
// crates/tbg-client/assets/battle/portraits/<slug>.webp (the slug rule of hud.js).
// Optional tool: needs Playwright and a local Chrome, and a static server of the
// client build first: python3 -m http.server <port> --directory crates/tbg-client/dist
// Image analysis and WebP encoding run inside Chrome (canvas), so nothing else
// (no Python, no Pillow) is required.
// Env: TBG_PREVIEW_URL (default http://127.0.0.1:8765), TBG_PLAYWRIGHT (module
// name or path, default 'playwright'), TBG_CHROME_CHANNEL (default 'chrome'), and
// TBG_UNITS_URL (optional: a separate build to shoot Archer and Cargo Crab from,
// e.g. after their kits change; by default every unit comes from TBG_PREVIEW_URL).
// After changing any portrait, bump PORTRAIT_REV in assets/battle/hud.js: the proxy
// caches images (and 404s) for four hours, so the URL needs a new revision.
// Usage: node scripts/unit-portraits.cjs [Kind ...]   (no kinds: all 25 buyable units)
const fs = require('node:fs/promises');
const path = require('node:path');
const {chromium} = require(process.env.TBG_PLAYWRIGHT || 'playwright');

const origin = process.env.TBG_PREVIEW_URL || 'http://127.0.0.1:8765';
const root = path.resolve(__dirname, '..');
const portraits = path.join(root, 'crates/tbg-client/assets/battle/portraits');
const scratch = path.join(root, '.local/portraits');
const SIZE = 400, BUDGET = 40 * 1024, SETTLE_MS = 6000;
const BACKGROUND = [21, 33, 42];
const args = process.argv.slice(2);
const special = new Set(['Archer', 'CargoCrab']);

// Same rule as hud.js `slug`: the UnitKind name, lowercase, no separators.
const slug = kind => kind.toLowerCase().replace(/[^a-z0-9]/g, '');

// Buyable roster = tbg-core FULL_POOL (summons are never in the Shop).
async function buyable() {
  const source = await fs.readFile(path.join(root, 'crates/tbg-core/src/types.rs'), 'utf8');
  const pool = source.match(/FULL_POOL: \[UnitKind; \d+\] = \[([\s\S]*?)\];/);
  if (!pool) throw new Error('FULL_POOL not found in tbg-core types.rs');
  return [...pool[1].matchAll(/UnitKind::(\w+)/g)].map(m => m[1]);
}

// Bounding box of everything that differs from the portrait background
// (luma of the absolute difference above 35), measured in the browser.
function bounds(page, png) {
  return page.evaluate(async ({b64, size, bg}) => {
    const image = new Image();
    image.src = 'data:image/png;base64,' + b64;
    await image.decode();
    const canvas = document.createElement('canvas');
    canvas.width = canvas.height = size;
    const context = canvas.getContext('2d', {willReadFrequently: true});
    context.drawImage(image, 0, 0);
    const data = context.getImageData(0, 0, size, size).data;
    let left = size, top = size, right = -1, bottom = -1;
    for (let y = 0; y < size; y++) {
      for (let x = 0; x < size; x++) {
        const i = (y * size + x) * 4;
        const luma = (299 * Math.abs(data[i] - bg[0]) + 587 * Math.abs(data[i + 1] - bg[1]) + 114 * Math.abs(data[i + 2] - bg[2])) / 1000;
        if (luma > 35) {
          if (x < left) left = x;
          if (x > right) right = x;
          if (y < top) top = y;
          if (y > bottom) bottom = y;
        }
      }
    }
    if (right < 0) throw new Error('portrait contains no visible figure');
    return [left, top, right + 1, bottom + 1];
  }, {b64: png.toString('base64'), size: SIZE, bg: BACKGROUND});
}

// Opaque lossy WebP at the highest quality that meets the file budget.
async function toWebp(page, png) {
  const out = await page.evaluate(async ({b64, size, budget}) => {
    const image = new Image();
    image.src = 'data:image/png;base64,' + b64;
    await image.decode();
    const canvas = document.createElement('canvas');
    canvas.width = canvas.height = size;
    // alpha:false keeps every pixel opaque: the Shop portraits are RGB.
    canvas.getContext('2d', {alpha: false}).drawImage(image, 0, 0);
    for (const quality of [0.85, 0.8, 0.75, 0.7, 0.65, 0.6]) {
      const blob = await new Promise(resolve => canvas.toBlob(resolve, 'image/webp', quality));
      if (!blob || blob.type !== 'image/webp') throw new Error('this browser cannot encode WebP');
      if (blob.size <= budget) {
        const bytes = new Uint8Array(await blob.arrayBuffer());
        let binary = '';
        for (let i = 0; i < bytes.length; i += 0x8000) binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
        return {quality, b64: btoa(binary)};
      }
    }
    throw new Error('no WebP quality fits ' + budget + ' bytes');
  }, {b64: png.toString('base64'), size: SIZE, budget: BUDGET});
  return {quality: out.quality, webp: Buffer.from(out.b64, 'base64')};
}

async function capture(page, kind) {
  const url = special.has(kind) && process.env.TBG_UNITS_URL ? process.env.TBG_UNITS_URL : origin;
  let zoom = 1, x = 0;
  const large = ['BoneBrute', 'SiegeWalker'].includes(kind);
  for (let attempt = 0; attempt < 7; attempt++) {
    const query = new URLSearchParams({gallery: kind, portrait: '', portrait_zoom: String(zoom), portrait_x: String(x)});
    await page.goto(`${url}/?${query}`, {waitUntil: 'load'});
    await page.waitForTimeout(SETTLE_MS);
    const png = await page.screenshot({type: 'png'});
    const [left, top, right, bottom] = await bounds(page, png);
    // The Shop's cover crop removes about 21 px at the top and 27 px below.
    const clipped = left <= 3 || top <= 25 || right >= SIZE - 3 || bottom >= SIZE - 30;
    if (clipped) {
      zoom *= 1.3;
      continue;
    }
    const delta = (left + right) / 2 - SIZE / 2;
    if (Math.abs(delta) > 25) {
      x += delta * 2 * (large ? 1.6 : 1.05) * zoom / SIZE;
      continue;
    }
    return {png, fit: {zoom, x, bounds: [left, top, right, bottom]}};
  }
  throw new Error(`${kind}: could not fit and centre the figure after seven captures`);
}

async function main() {
  const kinds = args.length ? args : await buyable();
  await fs.mkdir(scratch, {recursive: true});
  const browser = await chromium.launch({
    channel: process.env.TBG_CHROME_CHANNEL || 'chrome',
    headless: true,
    args: ['--headless=new', '--enable-gpu', '--ignore-gpu-blocklist', '--use-angle=gl'],
  });
  try {
    for (const kind of kinds) {
      const context = await browser.newContext({viewport: {width: SIZE, height: SIZE}, deviceScaleFactor: 1});
      const page = await context.newPage();
      const errors = [];
      page.on('pageerror', error => errors.push(error.message));
      const {png, fit} = await capture(page, kind);
      const {quality, webp} = await toWebp(page, png);
      await context.close();
      if (errors.length) throw new Error(`${kind}: page error ${errors[0]}`);
      await fs.writeFile(path.join(scratch, slug(kind) + '.png'), png);
      await fs.writeFile(path.join(portraits, slug(kind) + '.webp'), webp);
      console.log(`${kind}\t${slug(kind)}.webp\tq${Math.round(quality * 100)}\t${webp.length} B\t${JSON.stringify(fit)}`);
    }
  } finally {
    await browser.close();
  }
}

main().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
