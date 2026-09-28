// Which columns a download carries, and the picks the dialog remembers for
// the next one. The column list is per dataset, so a dataset change re-reads
// its own inputs; the remembered format and columns are keyed by dataset so
// one dataset's pick never restores into another's.

const STORAGE_PREFIX = 'sp-export:';

export const remember = (key, value) => {
  try { localStorage.setItem(STORAGE_PREFIX + key, JSON.stringify(value)); } catch { /* storage unavailable */ }
};

export const recall = (key) => {
  try { return JSON.parse(localStorage.getItem(STORAGE_PREFIX + key) ?? 'null'); } catch { return null; }
};

export const pageKey = () => `page:${window.location.pathname}`;

export const columnInputs = (dialog, dataset) =>
  [...dialog.querySelectorAll(`[data-export-columns="${dataset}"] input`)];

export const updateColumnCount = (dialog, picked) => {
  const count = dialog.querySelector('[data-export-columns-count]');
  if (count) count.textContent = `${picked.columns.length} of ${columnInputs(dialog, picked.dataset).length}`;
};

export const applyRemembered = (dialog, dataset) => {
  const saved = recall(dataset);
  if (!saved) return;
  const format = dialog.querySelector(`input[name="format"][value="${saved.format}"]`);
  if (format) format.checked = true;
  if (Array.isArray(saved.columns) && saved.columns.length) {
    for (const input of columnInputs(dialog, dataset)) input.checked = saved.columns.includes(input.value);
  }
};

export const filterColumns = (dialog, query, setHidden) => {
  const needle = query.trim().toLowerCase();
  for (const group of dialog.querySelectorAll('[data-export-group]')) {
    let visible = 0;
    for (const column of group.querySelectorAll('[data-export-column]')) {
      const match = needle === '' || (column.dataset.label ?? '').toLowerCase().includes(needle);
      setHidden(column, !match);
      if (match) visible += 1;
    }
    setHidden(group, visible === 0);
  }
};
