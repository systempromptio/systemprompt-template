import { on } from './events.js';

// Why: `hidden` is the server's no-JS default. Once JS owns the rows the
// detail row stays in the DOM and CSS animates it open, so the attribute is
// dropped on init and `is-open` carries the state from here on.
const adoptDetailRows = () => {
  for (const row of document.querySelectorAll('[data-expand-row]')) {
    const btn = row.querySelector('[data-action="row-toggle"]');
    const detail = document.getElementById(btn?.getAttribute('aria-controls') || '');
    if (!detail) continue;
    detail.hidden = false;
    detail.classList.toggle('is-open', btn.getAttribute('aria-expanded') === 'true');
  }
};

const setExpanded = (row, isOpen) => {
  const btn = row.querySelector('[data-action="row-toggle"]');
  const detail = document.getElementById(btn?.getAttribute('aria-controls') || '');
  if (!btn || !detail) return;
  btn.setAttribute('aria-expanded', String(isOpen));
  row.classList.toggle('is-expanded', isOpen);
  detail.classList.toggle('is-open', isOpen);
};

const toggleRow = (row) => {
  setExpanded(row, !row.classList.contains('is-expanded'));
};

// Why: the chevron was a 16px target at the far edge of a 2000px row. The
// whole summary row is the trigger now, and anything inside it that is a
// control of its own (a link to the record, a revoke button) keeps its job.
const isOwnControl = (target, row) => {
  const control = target.closest('a, button, input, select, label');
  return control !== null && control !== row && !control.matches('[data-action="row-toggle"]');
};

export const initTableExpand = () => {
  adoptDetailRows();
  on('click', '[data-expand-row]', (e, row) => {
    if (isOwnControl(e.target, row)) return;
    if (window.getSelection()?.toString()) return;
    toggleRow(row);
  });
  on('keydown', '[data-expand-row]', (e, row) => {
    if (e.target !== row || (e.key !== 'Enter' && e.key !== ' ')) return;
    e.preventDefault();
    toggleRow(row);
  });
};
