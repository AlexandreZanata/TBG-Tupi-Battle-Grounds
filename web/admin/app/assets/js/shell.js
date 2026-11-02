// Application frame: sidebar navigation, top bar with environment badges, preview banner and page slot.
import { el } from '../shared/dom.js';
import { lang, setLang, t } from './i18n.js';
import { actionNotice } from './ui/actions.js';

export const NAV = [
  { id: 'overview', icon: 'home' },
  { id: 'champions', icon: 'helmet' },
  { id: 'arenas', icon: 'island' },
  { id: 'matches', icon: 'swords' },
  { id: 'updates', icon: 'scroll' },
  { id: 'settings', icon: 'settings' },
];

function navItem(item) {
  return el('li', {},
    el('a', { href: `#/${item.id}`, 'data-route': item.id },
      el('img', { src: `/media/icons/${item.icon}.webp`, width: 22, height: 22, alt: '' }),
      el('span', { text: t(`nav.${item.id}`) })));
}

function langSwitch() {
  const group = el('div', { class: 'lang-switch', role: 'group', 'aria-label': t('nav.language') });
  for (const [code, label] of [['pt-BR', 'PT'], ['en', 'EN']]) {
    const button = el('button', { type: 'button', lang: code, 'aria-pressed': String(lang() === code), text: label });
    button.addEventListener('click', () => setLang(code));
    group.append(button);
  }
  return group;
}

/** Build the frame into `root`; returns the nodes the router updates. */
export function renderShell(root) {
  const title = el('h1', { id: 'page-title', tabindex: '-1' });
  const page = el('main', { id: 'page', class: 'page', 'aria-labelledby': 'page-title' });
  const skip = el('a', { class: 'skip-link', href: '#page', text: t('nav.skip') });
  // The URL hash holds the route, so the skip link moves focus without touching it.
  skip.addEventListener('click', event => {
    event.preventDefault();
    page.focus();
  });
  root.replaceChildren(
    skip,
    el('div', { class: 'app' },
      el('aside', { class: 'sidebar' },
        el('div', { class: 'sidebar-brand' },
          el('img', { src: '/media/emblem-160.webp', width: 44, height: 44, alt: '' }),
          el('div', {}, el('img', { class: 'sidebar-wordmark', src: '/media/wordmark-320.webp', width: 140, height: 56, alt: 'Tupi Battle Grounds' }),
            el('p', { text: t('app.title') }))),
        el('nav', { 'aria-label': t('nav.label') }, el('ul', {}, ...NAV.map(navItem))),
        el('p', { class: 'sidebar-foot' }, el('span', { class: 'badge badge-warn', text: t('app.preview') }))),
      el('div', { class: 'workspace' },
        el('header', { class: 'topbar' },
          title,
          el('div', { class: 'topbar-meta' },
            el('span', { class: 'badge badge-warn', text: t('app.preview') }),
            el('span', { class: 'badge', text: t('app.auth') }),
            langSwitch())),
        el('p', { class: 'preview-banner', role: 'note', text: t('app.banner') }),
        actionNotice(),
        page)));
  // The skip link targets the main region, which needs to accept focus.
  page.setAttribute('tabindex', '-1');
  return { title, page };
}
