// Settings: honest environment facts, the preview language, and disabled future form/confirm patterns.
import { el } from '../../shared/dom.js';
import { previewEnvironment } from '../api/public-status.js';
import { lang, setLang, t } from '../i18n.js';
import { gatedAction } from '../ui/actions.js';
import { panel } from '../ui/card.js';
import { openConfirm } from '../ui/confirm.js';
import { field } from '../ui/form.js';

function environmentPanel(env) {
  const rows = [
    ['settings.env.mode', t('settings.env.mode.value')],
    ['settings.env.content', t('settings.env.content.value')],
    ['settings.env.operational', t('settings.env.operational.value')],
    ['settings.env.status', t(env.publicStatus ? 'settings.env.status.on' : 'settings.env.status.off')],
    ['settings.env.admin', t('settings.env.admin.value')],
  ];
  return panel({
    title: t('settings.env.title'),
    children: [el('dl', { class: 'facts' }, ...rows.flatMap(([key, value]) => [el('dt', { text: t(key) }), el('dd', { text: value })]))],
  });
}

function preferencesPanel() {
  const select = el('select', { id: 'pref-lang' },
    el('option', { value: 'pt-BR', text: 'Português (Brasil)' }),
    el('option', { value: 'en', text: 'English' }));
  select.value = lang();
  select.addEventListener('change', () => setLang(select.value));
  return panel({
    title: t('settings.prefs.title'),
    children: [el('div', { class: 'field' }, el('label', { for: 'pref-lang', text: t('settings.prefs.language') }), select)],
  });
}

function futureFormPanel() {
  const severities = ['info', 'warning', 'critical'].map(value => ({ value, label: t(`form.notice.severity.${value}`) }));
  return panel({
    title: t('settings.form.title'),
    children: [
      el('p', { class: 'muted', text: t('settings.form.intro') }),
      el('div', { class: 'form-grid' },
        field({ label: t('form.notice.title'), hint: t('form.notice.title.hint'), control: 'text', maxlength: 80, disabled: true }),
        field({ label: t('form.notice.severity'), control: 'select', options: severities, disabled: true }),
        field({ label: t('form.notice.message'), hint: t('form.notice.message.hint'), control: 'textarea', disabled: true }),
        field({ label: t('form.notice.start'), control: 'datetime', disabled: true })),
      el('div', { class: 'form-actions' }, gatedAction(t('action.save'), 'auth', { primary: true })),
    ],
  });
}

function confirmPanel() {
  const open = el('button', { type: 'button', class: 'btn', text: t('settings.confirm.open') });
  open.addEventListener('click', () => openConfirm({
    action: t('confirm.example.action'),
    impact: t('confirm.example.impact'),
    word: lang() === 'en' ? 'HIDE' : 'OCULTAR',
  }));
  return panel({ title: t('settings.confirm.title'), children: [el('p', { class: 'muted', text: t('settings.confirm.intro') }), open] });
}

export async function render(root) {
  const env = await previewEnvironment();
  root.append(el('div', { class: 'panels' }, environmentPanel(env), preferencesPanel(), futureFormPanel(), confirmPanel()));
}
