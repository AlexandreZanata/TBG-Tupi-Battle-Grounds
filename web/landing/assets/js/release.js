// Availability gates for the browser game and the Linux package (config/release.json).
// Anything malformed or unverified stays in the safe "pending" state already in the HTML.
import { el, clear, fetchJson } from '../shared/dom.js';
import { t } from './i18n.js';

const WEB_ORIGIN = 'https://app.tupibg.com/';
const RELEASE_PREFIX = 'https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/releases/download/';
const KINDS = new Set(['appimage', 'tarball']);

export function webGameLive(config) {
  const web = config?.webGame;
  return web?.state === 'live' && typeof web.url === 'string' && web.url.startsWith(WEB_ORIGIN);
}

function validAsset(asset) {
  return KINDS.has(asset?.kind)
    && typeof asset.url === 'string' && asset.url.startsWith(RELEASE_PREFIX)
    && /^[0-9a-f]{64}$/.test(asset.sha256 ?? '')
    && Number.isInteger(asset.bytes) && asset.bytes > 0;
}

/** A Linux package is offered only when it targets the current server protocol and contract. */
export function linuxAvailable(config) {
  const linux = config?.linux;
  const server = config?.server;
  return linux?.state === 'available'
    && typeof linux.version === 'string' && linux.version.length > 0
    && Number.isInteger(server?.protocol) && linux.protocol === server.protocol
    && typeof server.contract === 'string' && linux.contract === server.contract
    && Array.isArray(linux.assets) && linux.assets.length > 0 && linux.assets.every(validAsset);
}

function toggle(selector, show) {
  for (const node of document.querySelectorAll(selector)) node.hidden = !show;
}

function applyWeb(config) {
  const live = webGameLive(config);
  for (const cta of document.querySelectorAll('[data-cta="web"]')) {
    if (live) {
      cta.href = config.webGame.url;
      cta.removeAttribute('aria-disabled');
    } else {
      cta.removeAttribute('href');
      cta.setAttribute('aria-disabled', 'true');
    }
  }
  toggle('[data-web-soon]', !live);
  toggle('[data-web-pending]', !live);
  toggle('[data-web-live]', live);
}

function downloadItem(asset) {
  const size = (asset.bytes / 1048576).toFixed(1);
  return el('li', {},
    el('a', { class: 'btn btn-gold', href: asset.url, rel: 'noopener noreferrer' },
      el('span', { text: t(`linux.download.${asset.kind}`) }),
      el('small', { text: t('linux.size', { size }) })),
    el('p', { class: 'sha' }, el('span', { text: 'SHA-256 ' }), el('code', { text: asset.sha256 })));
}

function applyLinux(config) {
  const available = linuxAvailable(config);
  toggle('[data-linux-pending]', !available);
  toggle('[data-linux-available]', available);
  if (!available) return;
  for (const node of document.querySelectorAll('[data-linux-version]')) node.textContent = config.linux.version;
  for (const list of document.querySelectorAll('[data-linux-downloads]')) {
    clear(list).append(...config.linux.assets.map(downloadItem));
  }
}

let config = null;

export async function loadRelease() {
  try {
    config = await fetchJson('/config/release.json', { maxBytes: 16 * 1024 });
  } catch {
    config = null;
  }
  renderRelease();
}

export function renderRelease() {
  applyWeb(config);
  applyLinux(config);
}
