import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';
import { initExpandRows, initTableSearch } from '../services/entity-common.js';
import { initEntityBatch } from '../services/entity-batch.js';
import { on } from '../services/events.js';

const initPanel = () => {
  const panel = document.getElementById('hook-panel');
  const overlay = document.getElementById('hook-panel-overlay');
  const panelTitle = document.getElementById('hook-panel-title');
  const form = document.getElementById('hook-form');
  const hookTypeSelect = document.getElementById('hook-type');
  const urlGroup = document.getElementById('hook-url-group');
  const commandGroup = document.getElementById('hook-command-group');
  let isEditMode = false;

  const updateFields = () => {
    const val = hookTypeSelect?.value;
    if (urlGroup) urlGroup.hidden = val === 'command';
    if (commandGroup) commandGroup.hidden = val === 'http';
  };

  const openPanel = (title, edit) => {
    isEditMode = !!edit;
    if (panelTitle) panelTitle.textContent = title;
    if (panel) panel.classList.add('open');
    if (overlay) overlay.classList.add('open');
  };

  const closePanel = () => {
    if (panel) panel.classList.remove('open');
    if (overlay) overlay.classList.remove('open');
    if (form) { form.reset(); form.removeAttribute('data-editing-id'); }
    isEditMode = false;
    updateFields();
  };

  if (hookTypeSelect) { hookTypeSelect.addEventListener('change', updateFields); updateFields(); }
  document.getElementById('add-hook-btn')?.addEventListener('click', () => {
    if (form) { form.reset(); form.removeAttribute('data-editing-id'); }
    updateFields();
    openPanel('Add Custom Hook', false);
  });
  document.getElementById('hook-panel-close')?.addEventListener('click', closePanel);
  document.getElementById('hook-cancel-btn')?.addEventListener('click', closePanel);
  overlay?.addEventListener('click', closePanel);
  return { openPanel, closePanel, form, hookTypeSelect, getEditMode: () => isEditMode };
};

const initEditButtons = (ctx) => {
  on('click', '.edit-hook-btn', (e, btn) => {
    e.stopPropagation();
    const hookId = btn.getAttribute('data-hook-id');
    const row = document.querySelector('.clickable-row[data-hook-id="' + hookId + '"]');
    if (row && ctx.form) {
      ctx.form.setAttribute('data-editing-id', hookId);
      document.getElementById('hook-name').value = row.getAttribute('data-hook-name') || '';
      document.getElementById('hook-description').value = row.getAttribute('data-description') || '';
      document.getElementById('hook-event-type').value = row.getAttribute('data-event-type') || '';
      ctx.hookTypeSelect.value = row.getAttribute('data-hook-type') || 'http';
      document.getElementById('hook-url').value = row.getAttribute('data-url') || '';
      document.getElementById('hook-command').value = row.getAttribute('data-command') || '';
      document.getElementById('hook-matcher').value = row.getAttribute('data-matcher') || '';
      document.getElementById('hook-timeout').value = row.getAttribute('data-timeout') || '';
      document.getElementById('hook-is-async').checked = row.getAttribute('data-is-async') === 'true';
      const pluginSelect = document.getElementById('hook-plugin');
      if (pluginSelect) pluginSelect.value = row.getAttribute('data-plugin-id') || '';
      ctx.openPanel('Edit Hook', true);
    }
  });
};

const initDeleteButtons = () => {
  on('click', '.delete-hook-btn', (e, btn) => {
    e.stopPropagation();
    const hookId = btn.getAttribute('data-hook-id');
    showConfirmDialog('Delete Hook', 'This cannot be undone.', 'Delete', async () => {
      try {
        await apiFetch('/user/hooks/' + encodeURIComponent(hookId), { method: 'DELETE' });
        showToast('Hook deleted', 'success'); setTimeout(() => location.reload(), 500);
      } catch (err) { showToast(err.message || 'Failed to delete hook', 'error'); }
    });
  });
};

const initToggleButtons = () => {
  on('click', '.toggle-hook-btn', async (e, btn) => {
    e.stopPropagation();
    try {
      await apiFetch('/user/hooks/' + encodeURIComponent(btn.getAttribute('data-hook-id')) + '/toggle', { method: 'PUT' });
      showToast('Hook toggled', 'success'); setTimeout(() => location.reload(), 500);
    } catch (err) { showToast(err.message || 'Failed to toggle hook', 'error'); }
  });
};

const initFormSubmit = (ctx) => {
  ctx.form?.addEventListener('submit', async (e) => {
    e.preventDefault();
    const editId = ctx.form.getAttribute('data-editing-id');
    const pluginVal = document.getElementById('hook-plugin')?.value || '';
    const payload = {
      hook_name: document.getElementById('hook-name').value.trim(),
      description: document.getElementById('hook-description').value.trim(),
      event_type: document.getElementById('hook-event-type').value,
      hook_type: ctx.hookTypeSelect.value,
      url: document.getElementById('hook-url').value.trim(),
      command: document.getElementById('hook-command').value.trim(),
      matcher: document.getElementById('hook-matcher').value.trim(),
      timeout: parseInt(document.getElementById('hook-timeout').value, 10) || 30,
      is_async: document.getElementById('hook-is-async').checked,
      plugin_id: pluginVal || null,
    };
    const url = editId ? '/user/hooks/' + encodeURIComponent(editId) : '/user/hooks';
    try {
      await apiFetch(url, { method: editId ? 'PUT' : 'POST', body: JSON.stringify(payload) });
      showToast(editId ? 'Hook updated' : 'Hook created', 'success'); ctx.closePanel(); setTimeout(() => location.reload(), 500);
    } catch (err) { showToast(err.message || 'Failed to save hook', 'error'); }
  });
};

export const initMyHooksPage = () => {
  if (document.getElementById('my-hooks-table')) {
    initExpandRows();
    initTableSearch('my-hooks-search', '#my-hooks-table', 'tbody tr.clickable-row');
    initEntityBatch({
      tableSelector: '#my-hooks-table',
      rowSelector: 'tbody tr.clickable-row',
      idExtractor: (row) => row.getAttribute('data-entity-id'),
      deleteEndpoint: '/user/hooks/batch-delete',
      entityLabel: 'hook',
    });
    const ctx = initPanel();
    initEditButtons(ctx);
    initDeleteButtons();
    initToggleButtons();
    initFormSubmit(ctx);
  }
};

initMyHooksPage();
