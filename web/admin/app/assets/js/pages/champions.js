// Champions: read-only roster from the repository snapshot, with search and Faction/Archetype filters.
import { el } from '../../shared/dom.js';
import { pick } from '../../shared/lang.js';
import { champions } from '../data/content.js';
import { lang, t } from '../i18n.js';
import { gatedAction } from '../ui/actions.js';
import { panel } from '../ui/card.js';
import { filterBar } from '../ui/filters.js';
import { table } from '../ui/list.js';

function matches(champion, values, current) {
  const query = (values.query ?? '').trim().toLocaleLowerCase(current);
  const name = `${pick(champion.name, 'pt-BR')} ${pick(champion.name, 'en')}`.toLocaleLowerCase(current);
  return (!query || name.includes(query))
    && (values.faction === 'all' || champion.faction === values.faction)
    && (values.archetype === 'all' || champion.archetype === values.archetype);
}

export async function render(root) {
  const data = await champions();
  const current = lang();
  const factionName = Object.fromEntries(data.factions.map(f => [f.id, pick(f.name, current)]));
  const archetypeName = Object.fromEntries(data.archetypes.map(a => [a.id, pick(a.name, current)]));
  const results = el('div', {});
  const count = el('p', { class: 'muted', 'aria-live': 'polite' });
  const columns = [
    { label: t('table.portrait'), className: 'col-portrait', cell: c => el('img', { src: `/${c.portrait}`, width: 48, height: 48, alt: '', loading: 'lazy' }) },
    { label: t('table.name'), cell: c => pick(c.name, current) },
    { label: t('table.faction'), cell: c => factionName[c.faction] },
    { label: t('table.archetype'), cell: c => archetypeName[c.archetype] },
    { label: t('table.actions'), className: 'col-actions', cell: () => gatedAction(t('action.edit'), 'balance') },
  ];
  const update = values => {
    const rows = data.champions.filter(c => matches(c, values, current));
    count.textContent = t('list.count', { shown: rows.length, total: data.champions.length });
    results.replaceChildren(table({ caption: t('nav.champions'), columns, rows, emptyText: t('champions.empty') }));
  };
  const filters = filterBar({
    fields: [
      { key: 'query', type: 'search', label: t('filter.search'), placeholder: t('filter.search.champion') },
      { key: 'faction', type: 'select', label: t('filter.faction'), options: data.factions.map(f => ({ value: f.id, label: factionName[f.id] })) },
      { key: 'archetype', type: 'select', label: t('filter.archetype'), options: data.archetypes.map(a => ({ value: a.id, label: archetypeName[a.id] })) },
    ],
    onChange: update,
  });
  root.append(
    el('p', { class: 'page-intro', text: t('champions.intro') }),
    panel({ title: t('nav.champions'), source: 'repository', children: [filters.element, count, results] }));
  update(filters.values());
}
