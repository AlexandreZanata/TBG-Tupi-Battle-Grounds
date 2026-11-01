// Updates: the reviewed public patch catalog, read-only; publishing stays a repository review.
import { el } from '../../shared/dom.js';
import { pick } from '../../shared/lang.js';
import { patchNotes } from '../data/content.js';
import { lang, t } from '../i18n.js';
import { gatedAction } from '../ui/actions.js';
import { panel } from '../ui/card.js';
import * as states from '../ui/states.js';

const STATUS_BADGE = { preview: 'badge-info', candidate: 'badge-warn', released: 'badge-ok' };

function note(entry, current) {
  const key = `updates.status.${entry.status}`;
  const status = t(key) === key ? entry.status : t(key);
  return el('li', { class: 'note' },
    el('p', { class: 'note-meta' },
      el('time', { datetime: entry.date, text: entry.date }),
      el('span', { text: t('updates.version', { version: entry.version }) }),
      el('span', { class: `badge ${STATUS_BADGE[entry.status] ?? ''}`, text: status })),
    el('h3', { text: pick(entry.title, current) }),
    el('p', { text: pick(entry.summary, current) }));
}

export async function render(root) {
  const catalog = await patchNotes();
  const current = lang();
  const entries = [...(catalog.notes ?? [])].sort((a, b) => b.date.localeCompare(a.date));
  root.append(
    el('div', { class: 'page-intro-row' },
      el('p', { class: 'page-intro', text: t('updates.intro') }),
      gatedAction(t('action.new_note'), 'notes', { primary: true })),
    panel({
      title: t('nav.updates'),
      source: 'repository',
      children: [entries.length ? el('ol', { class: 'notes' }, ...entries.map(entry => note(entry, current))) : states.empty('—')],
    }));
}
