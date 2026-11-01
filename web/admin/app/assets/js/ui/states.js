// Loading, error and empty states shared by every page.
import { el } from '../../shared/dom.js';
import { t } from '../i18n.js';

export function loading() {
  return el('div', { class: 'state state-loading', role: 'status' },
    el('span', { class: 'spinner', 'aria-hidden': 'true' }),
    el('span', { text: t('state.loading') }));
}

export function error(message, onRetry) {
  const box = el('div', { class: 'state state-error', role: 'alert' },
    el('strong', { text: t('state.error.title') }),
    el('p', { text: message }));
  if (onRetry) {
    const retry = el('button', { type: 'button', class: 'btn', text: t('state.error.retry') });
    retry.addEventListener('click', onRetry);
    box.append(retry);
  }
  return box;
}

export function empty(text) {
  return el('div', { class: 'state state-empty' }, el('p', { text }));
}
