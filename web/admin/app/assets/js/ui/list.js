// Data table: column definitions render each cell as text or a node; an empty result shows a state.
import { el } from '../../shared/dom.js';
import { empty } from './states.js';

/**
 * @param {{ caption: string, columns: { label: string, cell: (row) => Node|string, className?: string }[],
 *           rows: object[], emptyText: string }} options
 */
export function table({ caption, columns, rows, emptyText }) {
  if (rows.length === 0) return empty(emptyText);
  return el('div', { class: 'table-wrap' },
    el('table', { class: 'data-table' },
      el('caption', { class: 'sr-only', text: caption }),
      el('thead', {}, el('tr', {}, ...columns.map(col => el('th', { scope: 'col', class: col.className, text: col.label })))),
      el('tbody', {}, ...rows.map(row => el('tr', {}, ...columns.map(col => el('td', { class: col.className }, col.cell(row))))))));
}
