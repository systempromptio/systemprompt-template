// Turning the export dialog's picks into the query the server reads: the
// page's own filters, minus any the reader dropped for this download, with
// the dialog's window, format and columns in place of whatever the page had.
// A selection replaces the window: when `ids` are present the datasets read
// those rows and ignore the window entirely, so the window keys are dropped
// rather than sent alongside.

const WINDOW_KEYS = ['preset', 'from', 'to', 'days', 'start', 'end', 'month', 'since', 'format', 'columns', 'ids', 'source'];

const windowParams = (form, kind) => {
  const data = new FormData(form);
  if (kind === 'live') {
    const preset = data.get('live_preset');
    if (preset !== 'custom') return { preset };
    return { from: data.get('live_from'), to: data.get('live_to') };
  }
  if (kind === 'retained') {
    const days = data.get('retained_days');
    if (days !== 'custom') return { days };
    return { start: data.get('retained_start'), end: data.get('retained_end') };
  }
  if (kind === 'days') return { days: data.get('days') };
  if (kind === 'month') return { month: data.get('month') };
  return {};
};

export const droppedFilters = (dialog) => (dialog.dataset.exportDropped ?? '').split(',').filter(Boolean);

export const selection = (dialog) => {
  const form = dialog.querySelector('[data-export-form]');
  const option = dialog.querySelector('[data-export-dataset]').selectedOptions[0];
  const dataset = option.value;
  const columns = [...dialog.querySelectorAll(`[data-export-columns="${dataset}"] input:checked`)]
    .map((input) => input.value);
  const ids = (dialog.dataset.exportIds ?? '').split(',').filter(Boolean);
  return {
    dataset,
    source: option.dataset.source ?? null,
    format: new FormData(form).get('format'),
    columns,
    ids,
    window: ids.length ? {} : windowParams(form, option.dataset.window),
  };
};

export const buildUrl = (dialog, { dataset, source, format, columns, ids, window }, preview) => {
  const params = new URLSearchParams(dialog.dataset.exportQuery);
  for (const key of [...WINDOW_KEYS, ...droppedFilters(dialog)]) params.delete(key);
  for (const [key, value] of Object.entries(window)) {
    if (value) params.set(key, value);
  }
  if (ids.length) params.set('ids', ids.join(','));
  if (source) params.set('source', source);
  params.set('format', format);
  if (columns.length) params.set('columns', columns.join(','));
  const suffix = preview ? '/preview' : '';
  return `/admin/export/${encodeURIComponent(dataset)}${suffix}?${params.toString()}`;
};
