// The page help modal. The server renders `<dialog data-dialog="help">` with
// the glossary already inside it; this only opens it from the header's `?`
// and closes it from its own button or a backdrop click. Escape is the
// dialog element's native behaviour. Initialised from services/bootstrap.js
// so no page carries its own script tag for it.

import { on } from '../services/events.js';

const dialog = () => document.querySelector('[data-dialog="help"]');

let opener = null;

export const initHelp = () => {
  const el = dialog();
  if (!el) return;
  on('click', '[data-action="help-open"]', (e, trigger) => {
    e.preventDefault();
    opener = trigger;
    el.showModal();
  });
  on('click', '[data-action="help-close"]', () => el.close());
  el.addEventListener('click', (e) => {
    if (e.target === el) el.close();
  });
  el.addEventListener('close', () => opener?.focus());
};

