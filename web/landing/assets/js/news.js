// Latest public patch notes from data/patch-notes.json (the reviewed docs/patches catalog, copied at staging).
// Only the player-facing title and summary are shown; technical sections stay in the game and repository.
import { el, clear, fetchJson } from '../shared/dom.js';
import { pick } from '../shared/lang.js';
import { lang, t } from './i18n.js';

const SHOWN = 4;
const STATUS_BADGE = { preview: 'badge-info', candidate: 'badge-warn', released: 'badge-ok' };
let notes = null;
let failed = false;

function formatDate(iso, current) {
  const date = new Date(`${iso}T12:00:00Z`);
  if (Number.isNaN(date.getTime())) return iso;
  return new Intl.DateTimeFormat(current, { dateStyle: 'long', timeZone: 'UTC' }).format(date);
}

function statusLabel(status) {
  const key = `news.status.${status}`;
  const label = t(key);
  return label === key ? status : label;
}

function item(note) {
  const current = lang();
  return el('li', { class: 'news-item' },
    el('article', {},
      el('p', { class: 'news-meta' },
        el('time', { datetime: note.date, text: formatDate(note.date, current) }),
        el('span', { text: t('news.version', { version: note.version }) }),
        el('span', { class: `badge ${STATUS_BADGE[note.status] ?? ''}`, text: statusLabel(note.status) })),
      el('h3', { text: pick(note.title, current) }),
      el('p', { text: pick(note.summary, current) })));
}

export function renderNews() {
  const list = document.querySelector('[data-news-list]');
  if (!list) return;
  if (!notes) {
    clear(list).append(el('li', { class: 'state-note', text: t(failed ? 'news.error' : 'state.loading') }));
    return;
  }
  if (notes.length === 0) {
    clear(list).append(el('li', { class: 'state-note', text: t('news.empty') }));
    return;
  }
  clear(list).append(...notes.slice(0, SHOWN).map(item));
}

export async function loadNews() {
  try {
    const catalog = await fetchJson('/data/patch-notes.json', { maxBytes: 256 * 1024 });
    notes = [...(catalog.notes ?? [])].sort((a, b) => (a.date < b.date ? 1 : a.date > b.date ? -1 : 0));
  } catch {
    failed = true;
  }
  renderNews();
}
