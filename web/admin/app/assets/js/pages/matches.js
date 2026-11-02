// Matches and players: layout for future lookups and moderation over clearly fictional fixtures.
import { el } from '../../shared/dom.js';
import { pick } from '../../shared/lang.js';
import { arenas } from '../data/content.js';
import { matches, players } from '../data/fixtures.js';
import { lang, t } from '../i18n.js';
import { gatedAction } from '../ui/actions.js';
import { panel } from '../ui/card.js';
import { filterBar } from '../ui/filters.js';
import { table } from '../ui/list.js';

const MODES = ['casual', 'ranked', 'challenge'];

function duration(seconds) {
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`;
}

function winner(match) {
  if (match.winner === 'draw') return t('matches.draw');
  return match.winner === 'a' ? match.a : match.b;
}

function matchesPanel(arenaName, current) {
  const time = new Intl.DateTimeFormat(current, { dateStyle: 'short', timeStyle: 'short', timeZone: 'UTC' });
  const results = el('div', {});
  const columns = [
    { label: t('table.match'), cell: m => el('code', { text: m.id }) },
    { label: t('table.players'), cell: m => `${m.a} × ${m.b}` },
    { label: t('table.mode'), cell: m => t(`mode.${m.mode}`) },
    { label: t('table.arena'), cell: m => arenaName[m.arena] ?? m.arena },
    { label: t('table.duration'), cell: m => duration(m.seconds) },
    { label: t('table.winner'), cell: winner },
    { label: t('table.when'), cell: m => `${time.format(new Date(m.at))} UTC` },
    { label: t('table.actions'), className: 'col-actions', cell: () => gatedAction(t('action.review'), 'moderation') },
  ];
  const update = values => {
    const query = (values.query ?? '').trim().toLocaleLowerCase(current);
    const rows = matches.filter(m => (values.mode === 'all' || m.mode === values.mode)
      && (!query || `${m.a} ${m.b}`.toLocaleLowerCase(current).includes(query)));
    results.replaceChildren(table({ caption: t('matches.recent'), columns, rows, emptyText: t('matches.empty') }));
  };
  const filters = filterBar({
    fields: [
      { key: 'query', type: 'search', label: t('filter.search'), placeholder: t('filter.search.player') },
      { key: 'mode', type: 'select', label: t('filter.mode'), options: MODES.map(mode => ({ value: mode, label: t(`mode.${mode}`) })) },
    ],
    onChange: update,
  });
  update(filters.values());
  return panel({ title: t('matches.recent'), source: 'fixture', children: [filters.element, results] });
}

function playersPanel() {
  const columns = [
    { label: t('table.name'), cell: p => p.name },
    { label: t('table.rating'), cell: p => String(p.rating) },
    { label: t('table.matches'), cell: p => String(p.matches) },
    { label: t('table.status'), cell: p => el('span', { class: `badge ${p.status === 'active' ? 'badge-ok' : 'badge-warn'}`, text: t(`player.${p.status}`) }) },
    { label: t('table.actions'), className: 'col-actions', cell: () => gatedAction(t('action.moderate'), 'moderation') },
  ];
  return panel({ title: t('matches.players'), source: 'fixture', children: [table({ caption: t('matches.players'), columns, rows: players, emptyText: '' })] });
}

export async function render(root) {
  const data = await arenas();
  const current = lang();
  const arenaName = Object.fromEntries(data.arenas.map(a => [a.slug, pick(a.name, current)]));
  root.append(el('p', { class: 'page-intro', text: t('matches.intro') }), matchesPanel(arenaName, current), playersPanel());
}
