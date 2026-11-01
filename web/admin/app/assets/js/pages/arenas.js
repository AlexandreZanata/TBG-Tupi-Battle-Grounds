// Arenas: read-only cards with the modifiers derived from the game.
import { el } from '../../shared/dom.js';
import { pick } from '../../shared/lang.js';
import { arenas } from '../data/content.js';
import { lang, t } from '../i18n.js';
import { gatedAction } from '../ui/actions.js';
import { sourceBadge } from '../ui/card.js';

function card(arena, current) {
  const item = el('article', { class: 'arena-card' },
    el('img', { src: `/${arena.image}`, width: 320, height: 180, alt: '', loading: 'lazy' }),
    el('div', { class: 'arena-card-body' },
      el('h2', { text: pick(arena.name, current) }),
      el('p', { class: 'muted', text: pick(arena.theme, current) }),
      el('h3', { class: 'sr-only', text: t('arenas.effects') }),
      el('ul', { class: 'effects' }, ...arena.effects.map(effect => el('li', { text: pick(effect, current) }))),
      gatedAction(t('action.edit'), 'balance')));
  item.style.setProperty('--accent', arena.accent);
  return item;
}

export async function render(root) {
  const data = await arenas();
  const current = lang();
  root.append(
    el('div', { class: 'page-intro-row' }, el('p', { class: 'page-intro', text: t('arenas.intro') }), sourceBadge('repository')),
    el('div', { class: 'arena-cards' }, ...data.arenas.map(arena => card(arena, current))));
}
