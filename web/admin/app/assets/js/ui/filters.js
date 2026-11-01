// Filter bar: labelled search and select controls that report their values on every change.
import { el } from '../../shared/dom.js';
import { t } from '../i18n.js';

let nextId = 0;

function control(field, onChange) {
  const id = `filter-${++nextId}`;
  let input;
  if (field.type === 'search') {
    input = el('input', { id, type: 'search', placeholder: field.placeholder, autocomplete: 'off', value: field.value ?? '' });
    input.addEventListener('input', onChange);
  } else {
    input = el('select', { id },
      el('option', { value: 'all', text: t('filter.all') }),
      ...field.options.map(option => el('option', { value: option.value, text: option.label })));
    input.value = field.value ?? 'all';
    input.addEventListener('change', onChange);
  }
  input.dataset.key = field.key;
  return el('div', { class: 'filter' }, el('label', { for: id, text: field.label }), input);
}

/**
 * @param {{ fields: { key: string, type: 'search'|'select', label: string, placeholder?: string,
 *           options?: { value: string, label: string }[], value?: string }[], onChange: (values) => void }} options
 */
export function filterBar({ fields, onChange }) {
  const form = el('div', { class: 'filter-bar', role: 'search' });
  const values = () => Object.fromEntries([...form.querySelectorAll('[data-key]')].map(input => [input.dataset.key, input.value]));
  const changed = () => onChange(values());
  form.append(...fields.map(field => control(field, changed)));
  const clear = el('button', { type: 'button', class: 'btn btn-quiet', text: t('filter.clear') });
  clear.addEventListener('click', () => {
    for (const input of form.querySelectorAll('[data-key]')) input.value = input.tagName === 'SELECT' ? 'all' : '';
    changed();
  });
  form.append(clear);
  return { element: form, values };
}
