// What an export covers besides its rows: the page filters it carries, which
// the reader may drop for one download, and the window the server actually
// resolved, which can be narrower than the one picked when a contract caps it.

import { droppedFilters } from './export-url.js';

const chips = (dialog) => [...dialog.querySelectorAll('[data-export-filter]')];

const updateCount = (dialog) => {
  const count = dialog.querySelector('[data-export-filters-count]');
  if (!count) return;
  const all = chips(dialog).length;
  const kept = all - droppedFilters(dialog).length;
  count.textContent = kept === all ? `${all}` : `${kept} of ${all}`;
};

const toggle = (el, hidden) => {
  if (!el) return;
  el.hidden = hidden;
  el.classList.toggle('is-hidden', hidden);
};

export const resetFilters = (dialog) => {
  delete dialog.dataset.exportDropped;
  for (const chip of chips(dialog)) toggle(chip, false);
  toggle(dialog.querySelector('[data-export-filters-note]'), true);
  updateCount(dialog);
};

export const dropFilter = (dialog, key) => {
  if (!key) return;
  const dropped = new Set(droppedFilters(dialog));
  dropped.add(key);
  dialog.dataset.exportDropped = [...dropped].join(',');
  for (const chip of chips(dialog)) {
    if (chip.dataset.exportFilter === key) toggle(chip, true);
  }
  toggle(dialog.querySelector('[data-export-filters-note]'), false);
  updateCount(dialog);
};

const when = (iso) => new Date(iso).toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' });

// The dates the file covers, as the server resolved them; a clamped range
// says so, because the file then holds less than the reader picked.
export const renderWindowLine = (el, preview) => {
  if (!el) return;
  if (!preview?.from || !preview?.to) {
    toggle(el, true);
    return;
  }
  el.textContent = `${when(preview.from)} → ${when(preview.to)}`;
  if (preview.clamped) {
    el.dataset.tone = 'warn';
    el.append(' — narrowed to the widest window this export allows.');
  } else {
    delete el.dataset.tone;
  }
  toggle(el, false);
};
