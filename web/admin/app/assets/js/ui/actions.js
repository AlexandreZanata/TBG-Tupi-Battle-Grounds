// Gated actions: focusable buttons that never pretend to work. Activating one explains why it is
// unavailable (no server authorization yet) instead of running anything.
import { el } from '../../shared/dom.js';
import { ADMIN_AVAILABLE, REASONS } from '../api/admin.js';
import { t } from '../i18n.js';

function explain(button, reason) {
  const notice = document.querySelector('[data-action-notice]');
  if (!notice) return;
  notice.querySelector('[data-notice-title]').textContent = `${button.textContent} · ${t('action.unavailable')}`;
  notice.querySelector('[data-notice-text]').textContent = t(REASONS[reason] ?? REASONS.auth);
  notice.hidden = false;
  notice.querySelector('button').focus();
}

/** A button for a future administrative action; `reason` is a key of REASONS. */
export function gatedAction(label, reason = 'auth', { primary = false } = {}) {
  const button = el('button', { type: 'button', class: primary ? 'btn btn-gold' : 'btn btn-quiet', text: label });
  if (!ADMIN_AVAILABLE) {
    button.setAttribute('aria-disabled', 'true');
    button.title = t(REASONS[reason] ?? REASONS.auth);
    button.addEventListener('click', () => explain(button, reason));
  }
  return button;
}

/** The shared notice region the shell renders once (see shell.js). */
export function actionNotice() {
  const dismiss = el('button', { type: 'button', class: 'btn btn-quiet', text: t('action.dismiss') });
  const notice = el('div', { class: 'action-notice', role: 'status', hidden: true, 'data-action-notice': true },
    el('strong', { 'data-notice-title': true }),
    el('p', { 'data-notice-text': true }),
    dismiss);
  dismiss.addEventListener('click', () => {
    notice.hidden = true;
  });
  return notice;
}
