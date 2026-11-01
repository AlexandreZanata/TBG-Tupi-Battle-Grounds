// Form fields with label, hint and error slots, ready for future authorized forms.
import { el } from '../../shared/dom.js';

let nextId = 0;

/**
 * @param {{ label: string, hint?: string, control: 'text'|'textarea'|'select'|'datetime',
 *           options?: { value: string, label: string }[], disabled?: boolean, maxlength?: number }} spec
 */
export function field(spec) {
  const id = `field-${++nextId}`;
  const hintId = spec.hint ? `${id}-hint` : undefined;
  const common = { id, disabled: spec.disabled, 'aria-describedby': hintId };
  let input;
  if (spec.control === 'textarea') input = el('textarea', { ...common, rows: 4 });
  else if (spec.control === 'select') input = el('select', common, ...(spec.options ?? []).map(o => el('option', { value: o.value, text: o.label })));
  else input = el('input', { ...common, type: spec.control === 'datetime' ? 'datetime-local' : 'text', maxlength: spec.maxlength });
  return el('div', { class: 'field' },
    el('label', { for: id, text: spec.label }),
    input,
    spec.hint ? el('p', { class: 'field-hint', id: hintId, text: spec.hint }) : null,
    el('p', { class: 'field-error', role: 'alert', hidden: true }));
}
