// Champion roster with a Faction filter, from data/champions.json (derived from tbg-core by sync_content.py).
import { el, clear, fetchJson } from '../shared/dom.js';
import { pick } from '../shared/lang.js';
import { lang, t } from './i18n.js';

let data = null;
let failed = false;
let faction = 'all';

function names() {
  const current = lang();
  return {
    faction: Object.fromEntries(data.factions.map(f => [f.id, pick(f.name, current)])),
    archetype: Object.fromEntries(data.archetypes.map(a => [a.id, pick(a.name, current)])),
  };
}

function card(champion, labels) {
  return el('li', { class: 'champion' },
    el('img', { src: `/${champion.portrait}`, width: 200, height: 200, loading: 'lazy', alt: '' }),
    el('h3', { text: pick(champion.name, lang()) }),
    ...(champion.availability === 'preview' ? [el('span', { class: 'champion-preview', text: t('champions.preview') })] : []),
    el('p', { text: `${labels.archetype[champion.archetype]} · ${labels.faction[champion.faction]}` }));
}

// Only the grid and the pressed state change on a filter click, so keyboard focus stays on the button.
function renderGrid() {
  const grid = document.querySelector('[data-champion-grid]');
  const count = document.querySelector('[data-champion-count]');
  const all = data.champions;
  const shown = faction === 'all' ? all : all.filter(c => c.faction === faction);
  const labels = names();
  clear(grid).append(...shown.map(c => card(c, labels)));
  count.textContent = t('champions.count', { shown: shown.length, total: all.length });
  for (const button of document.querySelectorAll('[data-champion-filters] button')) {
    button.setAttribute('aria-pressed', String(button.dataset.faction === faction));
  }
}

function filterButton(id, label, count) {
  const button = el('button', { type: 'button', 'aria-pressed': String(faction === id), 'data-faction': id },
    el('span', { text: label }), el('span', { class: 'chip-count', text: String(count) }));
  button.addEventListener('click', () => {
    faction = id;
    renderGrid();
  });
  return button;
}

function render() {
  const filters = document.querySelector('[data-champion-filters]');
  const grid = document.querySelector('[data-champion-grid]');
  if (!filters || !grid) return;
  if (!data) {
    clear(grid).append(el('li', { class: 'state-note', text: t(failed ? 'state.error' : 'state.loading') }));
    return;
  }
  const labels = names();
  clear(filters).append(
    filterButton('all', t('champions.all'), data.champions.length),
    ...data.factions.map(f => filterButton(f.id, labels.faction[f.id], data.champions.filter(c => c.faction === f.id).length)));
  renderGrid();
}

export async function loadChampions() {
  try {
    data = await fetchJson('/data/champions.json');
    // Hero counts follow the published catalog so they never go stale.
    for (const [key, list] of [['champions', data.champions], ['factions', data.factions]]) {
      const node = document.querySelector(`[data-stat="${key}"]`);
      if (node) node.textContent = String(list.length);
    }
  } catch {
    failed = true;
  }
  render();
}

export const renderChampions = render;
