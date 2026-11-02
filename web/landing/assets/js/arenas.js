// Arena cards from data/arenas.json (names, themes and modifiers derived from the game by sync_content.py).
import { el, clear, fetchJson } from '../shared/dom.js';
import { pick } from '../shared/lang.js';
import { lang, t } from './i18n.js';

let data = null;
let failed = false;

function card(arena) {
  const current = lang();
  const name = pick(arena.name, current);
  const item = el('li', { class: 'arena' },
    el('img', { src: `/${arena.image}`, width: 640, height: 360, loading: 'lazy', alt: t('arenas.preview.alt', { name }) }),
    el('div', { class: 'arena-body' },
      el('h3', { text: name }),
      el('p', { class: 'arena-theme', text: pick(arena.theme, current) }),
      el('ul', { class: 'arena-effects', role: 'list' },
        ...arena.effects.map(effect => el('li', { text: pick(effect, current) }))),
      el('p', { class: 'arena-symmetric', text: t('arenas.symmetric') })));
  // CSSOM custom property, allowed under the strict style-src CSP.
  item.style.setProperty('--accent', arena.accent);
  return item;
}

export function renderArenas() {
  const grid = document.querySelector('[data-arena-grid]');
  if (!grid) return;
  if (!data) {
    clear(grid).append(el('li', { class: 'state-note', text: t(failed ? 'state.error' : 'state.loading') }));
    return;
  }
  clear(grid).append(...data.arenas.map(card));
}

export async function loadArenas() {
  try {
    data = await fetchJson('/data/arenas.json');
  } catch {
    failed = true;
  }
  renderArenas();
}
