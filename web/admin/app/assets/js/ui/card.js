// Cards: a statistic tile and a titled panel, each labelled with where its data comes from.
import { el } from '../../shared/dom.js';
import { t } from '../i18n.js';

const SOURCE_BADGE = { repository: 'badge-info', fixture: 'badge-warn', public: 'badge-ok' };

/** A badge naming the data source: 'repository', 'fixture' or 'public'. */
export function sourceBadge(source) {
  return el('span', { class: `badge ${SOURCE_BADGE[source] ?? ''}`, text: t(`source.${source}`) });
}

export function statCard({ label, value, source }) {
  return el('article', { class: 'stat-card' },
    el('p', { class: 'stat-label', text: label }),
    el('p', { class: 'stat-value', text: String(value) }),
    source ? sourceBadge(source) : null);
}

export function panel({ title, source, badge, children = [] }) {
  return el('section', { class: 'panel' },
    el('header', { class: 'panel-head' },
      el('h2', { text: title }),
      badge ?? null,
      source ? sourceBadge(source) : null),
    ...children);
}
