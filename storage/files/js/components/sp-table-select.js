// Row selection for tables with bulk actions. The server renders the boxes
// (`components/table-select`) and the bar (`components/bulk-bar`); this keeps
// the count, the select-all state, shift-click ranges and the id list the
// bar's forms and the export dialog read. State lives in the DOM — the ticked
// boxes are the selection.

import { on } from '../services/events.js';

const rows = () => [...document.querySelectorAll('[data-select-row]')];
const selectedIds = () => rows().filter((box) => box.checked).map((box) => box.dataset.selectRow);

let lastClicked = null;

const sync = () => {
  const ids = selectedIds();
  const all = document.querySelector('[data-select-all]');
  if (all) {
    const total = rows().length;
    all.checked = total > 0 && ids.length === total;
    all.indeterminate = ids.length > 0 && ids.length < total;
  }
  for (const bar of document.querySelectorAll('[data-bulk-bar]')) {
    bar.hidden = ids.length === 0;
    const count = bar.querySelector('[data-bulk-count]');
    if (count) count.textContent = ids.length.toLocaleString();
    for (const input of bar.querySelectorAll('[data-bulk-ids]')) input.value = ids.join(',');
  }
  for (const box of rows()) box.closest('tr')?.classList.toggle('is-selected', box.checked);
};

const rangeTo = (box) => {
  const list = rows();
  const from = list.indexOf(lastClicked);
  const to = list.indexOf(box);
  if (from < 0 || to < 0) return;
  const [a, b] = from < to ? [from, to] : [to, from];
  for (const other of list.slice(a, b + 1)) other.checked = box.checked;
};

export const initTableSelect = () => {
  if (!document.querySelector('[data-select-row], [data-select-all]')) return;
  on('click', '[data-select-row]', (e, box) => {
    if (e.shiftKey && lastClicked) rangeTo(box);
    lastClicked = box;
    sync();
  });
  on('change', '[data-select-all]', (e, all) => {
    for (const box of rows()) box.checked = all.checked;
    sync();
  });
  on('click', '[data-action="bulk-clear"]', () => {
    for (const box of rows()) box.checked = false;
    lastClicked = null;
    sync();
  });
  sync();
};

export const selectedRowIds = selectedIds;

initTableSelect();
