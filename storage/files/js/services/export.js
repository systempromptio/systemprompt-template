// The export dialog. The server renders the dialog (`components/export-dialog`)
// with every dataset's grouped columns, the page's filter query and its
// filters as chips; this module owns its state: which dataset, window, format
// and columns are picked, which page filters the reader dropped, whether a row
// selection or a breakdown bucket replaces the window, the URL they resolve
// to, and the live count fetched from `/preview` so the reader knows how many
// rows a download will hold, and over which dates, before asking.

import { on } from './events.js';
import { rawResponse, errorMessage } from './api.js';
import { selection, buildUrl } from './export-url.js';
import { resetFilters, dropFilter, renderWindowLine } from './export-scope.js';
import { remember, recall, pageKey, columnInputs, updateColumnCount, applyRemembered, filterColumns } from './export-columns.js';
import { renderPreview, documentHref } from './export-preview.js';
import { showFormats } from './export-formats.js';
import { selectedRowIds } from '../components/sp-table-select.js';

const PREVIEW_DEBOUNCE_MS = 250;

// The window a dialog falls back to when the page names none, per kind.
const WINDOW_DEFAULTS = { live_preset: '7d', retained_days: '30', days: '30' };

const dialogEl = () => document.querySelector('[data-export-dialog]');

const setHidden = (el, hidden) => {
  if (!el) return;
  el.hidden = hidden;
  el.classList.toggle('is-hidden', hidden);
};

const datasetSelect = (dialog) => dialog.querySelector('[data-export-dataset]');

const datasetOption = (dialog) => datasetSelect(dialog).selectedOptions[0];

const kindOf = (dialog) => datasetOption(dialog).dataset.kind ?? 'table';

// Only the picked dataset's window kind, formats and column list are
// visible; a selection disables the window entirely because the ids decide
// the rows, and a record export has no columns to pick.
const showWindow = (dialog) => {
  const option = datasetOption(dialog);
  const kind = kindOf(dialog);
  const windowKind = kind === 'document' ? null : option.dataset.window;
  const selected = Boolean(dialog.dataset.exportIds);
  dialog.classList.toggle('sp-export--document', kind !== 'table');
  setHidden(dialog.querySelector('.sp-export__pane--columns'), kind !== 'table');
  showFormats(dialog, kind, setHidden);
  for (const block of dialog.querySelectorAll('[data-export-window]')) {
    setHidden(block, block.dataset.exportWindow !== windowKind);
    block.disabled = selected;
    const checked = block.querySelector('input[type="radio"]:checked');
    const custom = block.querySelector('[data-export-custom]');
    if (custom && checked) setHidden(custom, checked.value !== 'custom');
  }
  for (const block of dialog.querySelectorAll('[data-export-columns]')) {
    setHidden(block, block.dataset.exportColumns !== option.value);
  }
  setHidden(dialog.querySelector('[data-export-filters]'), kind === 'document' || selected);
  const description = dialog.querySelector('[data-export-description]');
  if (description) description.textContent = option.dataset.description ?? '';
};

let previewTimer = null;
let previewSeq = 0;

const refresh = (dialog) => {
  const kind = kindOf(dialog);
  const el = dialog.querySelector('[data-export-preview]');
  const windowLine = dialog.querySelector('[data-export-window-line]');
  previewSeq += 1;
  clearTimeout(previewTimer);
  delete el.dataset.tone;
  if (kind === 'document') {
    dialog.querySelector('[data-export-download]').href = documentHref(dialog, datasetOption);
    el.textContent = 'The whole conversation, one file.';
    renderWindowLine(windowLine, null);
    return;
  }
  const picked = selection(dialog);
  updateColumnCount(dialog, picked);
  remember(picked.dataset, { format: picked.format, columns: picked.columns });
  dialog.querySelector('[data-export-download]').href = buildUrl(dialog, picked, false);
  el.textContent = 'Counting…';
  const seq = previewSeq;
  previewTimer = setTimeout(async () => {
    try {
      const resp = await rawResponse(buildUrl(dialog, picked, true));
      if (seq !== previewSeq) return;
      if (!resp.ok) throw new Error(await errorMessage(resp));
      const preview = await resp.json();
      renderPreview(el, preview, kind, picked.columns.length);
      renderWindowLine(windowLine, preview);
    } catch (err) {
      if (seq !== previewSeq) return;
      el.dataset.tone = 'error';
      el.textContent = err.message || 'Could not count rows';
      renderWindowLine(windowLine, null);
    }
  }, PREVIEW_DEBOUNCE_MS);
};

const setColumns = (dialog, predicate) => {
  const { dataset } = selection(dialog);
  for (const input of columnInputs(dialog, dataset)) input.checked = predicate(input);
  refresh(dialog);
};

// The page's `since` tabs name windows the dialog spells differently; "all"
// is the widest window each contract allows.
const SINCE = { live: { all: '90d' }, retained: { '24h': '1', '7d': '7', '30d': '30', '90d': '90', all: '365' } };

// The dialog opens on the window the page is showing, so "Export" with no
// further clicks is the table on screen. Every radio is reset first, so a
// previous open's pick never leaks into this one.
const syncFromPage = (dialog) => {
  const params = new URLSearchParams(dialog.dataset.exportQuery);
  const check = (name, value) => {
    const input = dialog.querySelector(`input[name="${name}"][value="${value}"]`);
    if (input) input.checked = true;
    return Boolean(input);
  };
  for (const [name, value] of Object.entries(WINDOW_DEFAULTS)) check(name, value);
  const since = params.get('since');
  const preset = params.get('preset') ?? (since ? SINCE.live[since] ?? since : null);
  if (preset) check('live_preset', preset);
  if (params.get('from') && params.get('to')) {
    check('live_preset', 'custom');
    for (const edge of ['from', 'to']) {
      dialog.querySelector(`input[name="live_${edge}"]`).value = params.get(edge).slice(0, 16);
    }
  }
  const days = params.get('days') ?? (since ? SINCE.retained[since] : null);
  if (days) {
    check('retained_days', days);
    check('days', days);
  }
  if (params.get('start') && params.get('end')) {
    check('retained_days', 'custom');
    dialog.querySelector('input[name="retained_start"]').value = params.get('start').slice(0, 10);
    dialog.querySelector('input[name="retained_end"]').value = params.get('end').slice(0, 10);
  }
  const month = dialog.querySelector('input[name="month"]');
  if (month) month.value = params.get('month') ?? '';
};

const hasOption = (select, value) => [...select.options].some((o) => o.value === value);

// A trigger may narrow what the dialog exports: `data-export-selected`
// names the dataset the ticked rows belong to, `data-export-group-label`
// names a breakdown bucket (its `data-export-query` carrying that bucket's
// filters), and either shows in the scope strip in place of the window. A
// selection for a dataset this page does not offer is ignored rather than
// sent to whichever dataset happens to be showing.
const applyScope = (dialog, trigger) => {
  const scope = dialog.querySelector('[data-export-scope]');
  const select = datasetSelect(dialog);
  const selected = trigger?.dataset.exportSelected;
  const offered = Boolean(selected) && hasOption(select, selected);
  const ids = offered ? selectedRowIds() : [];
  dialog.dataset.exportIds = ids.join(',');
  if (trigger?.dataset.exportQuery !== undefined) dialog.dataset.exportQuery = trigger.dataset.exportQuery;
  if (offered) {
    select.value = selected;
  } else {
    const saved = recall(pageKey());
    if (saved && hasOption(select, saved)) select.value = saved;
  }
  const group = trigger?.dataset.exportGroupLabel;
  if (ids.length) {
    scope.textContent = `Exporting ${ids.length.toLocaleString()} selected ${trigger.dataset.exportNoun ?? 'rows'}`;
  } else if (group) {
    scope.textContent = `Exporting the bucket “${group}”`;
  }
  setHidden(scope, !ids.length && !group);
};

let opener = null;
let pageQuery = null;

const open = (trigger) => {
  const dialog = dialogEl();
  if (!dialog) return false;
  opener = trigger;
  pageQuery = dialog.dataset.exportQuery;
  resetFilters(dialog);
  applyScope(dialog, trigger);
  syncFromPage(dialog);
  applyRemembered(dialog, datasetOption(dialog).value);
  showWindow(dialog);
  dialog.showModal();
  refresh(dialog);
  return true;
};

const close = () => {
  const dialog = dialogEl();
  if (dialog?.open) dialog.close();
};

export const initExport = () => {
  const dialog = dialogEl();
  if (!dialog) return;
  on('click', '[data-export-open]', (e, trigger) => {
    if (open(trigger)) e.preventDefault();
  });
  on('click', '[data-action="export-close"]', close);
  on('click', '[data-action="export-columns-all"]', () => setColumns(dialog, () => true));
  on('click', '[data-action="export-columns-default"]', () =>
    setColumns(dialog, (input) => input.dataset.default === 'true'));
  on('click', '[data-action="export-columns-none"]', () => setColumns(dialog, () => false));
  on('click', '[data-action="export-filter-drop"]', (e, button) => {
    dropFilter(dialog, button.dataset.key);
    refresh(dialog);
  });
  on('click', '[data-export-download]', () => setTimeout(close, 0));
  on('input', '[data-export-column-filter]', (e, input) => filterColumns(dialog, input.value, setHidden));
  dialog.addEventListener('change', (e) => {
    if (e.target.matches('[data-export-dataset]')) {
      remember(pageKey(), e.target.value);
      applyRemembered(dialog, datasetOption(dialog).value);
    }
    showWindow(dialog);
    refresh(dialog);
  });
  dialog.addEventListener('close', () => {
    if (pageQuery !== null) dialog.dataset.exportQuery = pageQuery;
    delete dialog.dataset.exportIds;
    resetFilters(dialog);
    opener?.focus();
  });
};
