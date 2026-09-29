import { showConfirmDialog } from '../services/confirm.js';
import { initSyncPlane } from '../components/sp-sync-plane.js';

// The editor is a plain form and works without this file. Two enhancements:
// a delete asks first instead of going on the click, and a window's custom
// length field only shows when the length is "custom".

const CUSTOM = 'custom';

const syncCustomField = (select) => {
  const row = select.closest('[data-window]');
  if (!row) return;
  const custom = row.querySelector('[data-window-custom]');
  if (custom) custom.hidden = select.value !== CUSTOM;
};

document.querySelectorAll('[data-window-choice]').forEach((select) => {
  syncCustomField(select);
  select.addEventListener('change', () => syncCustomField(select));
});

document.querySelectorAll('form[data-confirm]').forEach((form) => {
  form.addEventListener('submit', (event) => {
    if (form.dataset.confirmed === 'yes') return;
    event.preventDefault();
    showConfirmDialog('Delete policy', form.dataset.confirm, 'Delete', () => {
      form.dataset.confirmed = 'yes';
      form.requestSubmit();
    });
  });
});

// Why: the Sync tab renders the shared sync-plane component; its buttons
// are bound here so the tab works without a second page script.
initSyncPlane();
