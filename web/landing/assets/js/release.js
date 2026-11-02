// Availability data combines static browser configuration with the live verified native catalog.
import { fetchJson } from '../shared/dom.js';
import { renderNative } from './native-downloads.js';
import { mergeCatalog } from './download-rules.js';

export { available as platformAvailable } from './download-rules.js';

const WEB_ORIGIN = 'https://app.tupibg.com/';
let config = null;

export function webGameLive(value) {
  return value?.webGame?.state === 'live' && typeof value.webGame.url === 'string' && value.webGame.url.startsWith(WEB_ORIGIN);
}

function toggle(selector, show) {
  for (const node of document.querySelectorAll(selector)) node.hidden = !show;
}

export async function loadRelease() {
  // Bind the pending dialog before metadata requests; publication must not gate interaction.
  renderNative(config);
  try {
    config = await fetchJson('/config/release.json', { maxBytes: 16 * 1024 });
  } catch {
    config = null;
  }
  try {
    const catalog = await fetchJson('/downloads.json', { maxBytes: 16 * 1024 });
    config = mergeCatalog(config, catalog);
  } catch {
    // An unavailable verifier cannot enable downloads; preserve the authored pending state.
    config = { ...config, linux: { state: 'pending' }, windows: { state: 'pending' } };
  }
  renderRelease();
}

export function renderRelease() {
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
  renderNative(config);
}
