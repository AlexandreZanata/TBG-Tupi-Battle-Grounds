// Confirmation dialog pattern for irreversible actions: states the action and its impact and asks
// for a typed word. While ADMIN_AVAILABLE is false the confirm button can never be enabled.
import { el } from '../../shared/dom.js';
import { ADMIN_AVAILABLE } from '../api/admin.js';
import { t } from '../i18n.js';

/** @param {{ action: string, impact: string, word: string }} spec */
export function openConfirm({ action, impact, word }) {
  const opener = document.activeElement;
  const input = el('input', { id: 'confirm-word', type: 'text', autocomplete: 'off', spellcheck: 'false' });
  const confirm = el('button', { type: 'button', class: 'btn btn-danger', disabled: true, text: t('confirm.confirm') });
  const cancel = el('button', { type: 'button', class: 'btn btn-quiet', text: t('confirm.cancel') });
  const dialog = el('dialog', { class: 'confirm-dialog', 'aria-labelledby': 'confirm-title' },
    el('h2', { id: 'confirm-title', text: t('confirm.title') }),
    el('p', { class: 'confirm-action', text: action }),
    el('p', { class: 'confirm-impact', text: impact }),
    el('label', { for: 'confirm-word', text: t('confirm.typed') }),
    input,
    ADMIN_AVAILABLE ? null : el('p', { class: 'field-hint', text: t('confirm.preview') }),
    el('div', { class: 'confirm-buttons' }, cancel, confirm));
  input.addEventListener('input', () => {
    confirm.disabled = !ADMIN_AVAILABLE || input.value.trim() !== word;
  });
  cancel.addEventListener('click', () => dialog.close());
  dialog.addEventListener('close', () => {
    dialog.remove();
    opener?.focus?.();
  });
  document.body.append(dialog);
  dialog.showModal();
  input.focus();
}
