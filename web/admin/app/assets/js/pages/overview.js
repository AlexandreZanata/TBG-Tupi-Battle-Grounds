// Overview: content counts, public service status, the pending access contract and planned areas.
import { el } from '../../shared/dom.js';
import { pick } from '../../shared/lang.js';
import { publicStatus } from '../api/public-status.js';
import { champions, arenas, patchNotes } from '../data/content.js';
import { activity } from '../data/fixtures.js';
import { lang, t } from '../i18n.js';
import { panel, statCard } from '../ui/card.js';
import * as states from '../ui/states.js';

const GAPS = ['identity', 'roles', 'session', 'audit', 'network'];
const EXTENSIONS = ['content', 'balance', 'moderation', 'permissions', 'audit'];

function statusRows(status) {
  if (!status.enabled) return [el('p', { class: 'muted', text: t('overview.status.off') })];
  const health = el('span', { class: `badge ${status.healthy ? 'badge-ok' : 'badge-danger'}`, text: t(status.healthy ? 'overview.status.ok' : 'overview.status.down') });
  const unknown = t('overview.status.unknown');
  return [el('dl', { class: 'facts' },
    el('dt', { text: t('overview.status.health') }), el('dd', {}, health),
    el('dt', { text: t('overview.status.protocol') }), el('dd', { text: status.protocol ?? unknown }),
    el('dt', { text: t('overview.status.client') }), el('dd', { text: status.latestClient ?? unknown }))];
}

function statusPanel() {
  const body = el('div', {}, states.loading());
  publicStatus()
    .then(status => body.replaceChildren(...statusRows(status)))
    .catch(error => body.replaceChildren(states.error(error.message)));
  return panel({ title: t('overview.status.title'), source: 'public', children: [body] });
}

function accessPanel() {
  return panel({
    title: t('overview.access.title'),
    badge: el('span', { class: 'badge badge-warn', text: t('overview.access.state') }),
    children: [
      el('p', { text: t('overview.access.text') }),
      el('ul', { class: 'checklist' }, ...GAPS.map(gap => el('li', { text: t(`overview.access.gap.${gap}`) }))),
      el('p', { class: 'muted', text: t('overview.access.doc') }),
    ],
  });
}

function activityPanel() {
  const time = new Intl.DateTimeFormat(lang(), { dateStyle: 'short', timeStyle: 'short', timeZone: 'UTC' });
  return panel({
    title: t('overview.activity.title'),
    source: 'fixture',
    children: [el('ol', { class: 'timeline' }, ...activity.map(item => el('li', {},
      el('time', { datetime: item.at, text: `${time.format(new Date(item.at))} UTC` }),
      el('span', { text: pick(item.text, lang()) }))))],
  });
}

function extensionsPanel() {
  return panel({
    title: t('overview.extensions.title'),
    children: [el('ul', { class: 'extensions' }, ...EXTENSIONS.map(name => el('li', {},
      el('strong', { text: t(`ext.${name}.name`) }),
      el('span', { class: 'badge', text: t('overview.extensions.state') }),
      el('p', { text: t(`ext.${name}.text`) }))))],
  });
}

export async function render(root) {
  const [roster, arenaData, notes] = await Promise.all([champions(), arenas(), patchNotes()]);
  root.append(
    el('p', { class: 'page-intro', text: t('overview.intro') }),
    el('div', { class: 'stats' },
      statCard({ label: t('overview.stat.champions'), value: roster.champions.length, source: 'repository' }),
      statCard({ label: t('overview.stat.factions'), value: roster.factions.length, source: 'repository' }),
      statCard({ label: t('overview.stat.arenas'), value: arenaData.arenas.length, source: 'repository' }),
      statCard({ label: t('overview.stat.notes'), value: notes.notes.length, source: 'repository' })),
    el('div', { class: 'panels' }, statusPanel(), accessPanel(), activityPanel(), extensionsPanel()));
}
