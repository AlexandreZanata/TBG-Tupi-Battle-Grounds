// Hash router: one page module per area, with loading, error (retry) and not-found states.
import { t } from './i18n.js';
import * as states from './ui/states.js';
import * as overview from './pages/overview.js';
import * as champions from './pages/champions.js';
import * as arenas from './pages/arenas.js';
import * as matches from './pages/matches.js';
import * as updates from './pages/updates.js';
import * as settings from './pages/settings.js';

const PAGES = { overview, champions, arenas, matches, updates, settings };
let frame = null;
let generation = 0;

function routeId() {
  const id = location.hash.replace(/^#\/?/, '');
  return id || 'overview';
}

export async function showRoute({ focus = true } = {}) {
  if (!frame) return;
  const id = routeId();
  const page = PAGES[id];
  const run = ++generation;
  for (const link of document.querySelectorAll('[data-route]')) {
    if (link.dataset.route === id) link.setAttribute('aria-current', 'page');
    else link.removeAttribute('aria-current');
  }
  const heading = page ? t(`nav.${id}`) : t('state.notfound.title');
  frame.title.textContent = heading;
  document.title = `${heading} · ${t('app.title')} · TBG`;
  if (focus) frame.title.focus();
  if (!page) {
    frame.page.replaceChildren(states.empty(t('state.notfound.text')));
    return;
  }
  frame.page.replaceChildren(states.loading());
  const content = document.createElement('div');
  try {
    await page.render(content);
    if (run === generation) frame.page.replaceChildren(content);
  } catch (error) {
    if (run === generation) frame.page.replaceChildren(states.error(error.message, () => showRoute()));
  }
}

export function startRouter(nextFrame) {
  frame = nextFrame;
  window.addEventListener('hashchange', () => showRoute());
}

export function setFrame(nextFrame) {
  frame = nextFrame;
}
