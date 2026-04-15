import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';
import { initExpandRows } from '../services/entity-common.js';
import { initEntityBatch } from '../services/entity-batch.js';
import { on } from '../services/events.js';

const initSearch = (table) => {
  const searchInput = document.getElementById('my-secrets-search');
  if (searchInput) {
    let timer;
    searchInput.addEventListener('input', () => {
      clearTimeout(timer);
      timer = setTimeout(() => {
        const query = searchInput.value.toLowerCase().trim();
        for (const row of table.querySelectorAll('tr.clickable-row')) {
          const name = (row.getAttribute('data-name') || '') + ' ' + (row.getAttribute('data-plugin') || '');
          const matches = !query || name.toLowerCase().includes(query);
          row.hidden = !matches;
          const detail = row.nextElementSibling;
          if (detail?.classList.contains('detail-row')) detail.hidden = !matches;
        }
        for (const header of table.querySelectorAll('tr.group-header-row')) {
          let next = header.nextElementSibling;
          let anyVisible = false;
          while (next && !next.classList.contains('group-header-row')) {
            if (next.classList.contains('clickable-row') && !next.hidden) anyVisible = true;
            next = next.nextElementSibling;
          }
          header.hidden = !anyVisible;
        }
      }, 200);
    });
  }
};

const initSecretPanel = () => {
  const panel = document.getElementById('secret-panel');
  const overlay = document.getElementById('secret-panel-overlay');
  const panelTitle = document.getElementById('secret-panel-title');
  const form = document.getElementById('secret-form');
  const pluginSelect = document.getElementById('secret-plugin-id');
  const varNameInput = document.getElementById('secret-var-name');
  const varValueInput = document.getElementById('secret-var-value');
  const isSecretCheckbox = document.getElementById('secret-is-secret');
  let isEditMode = false;

  const openPanel = (title, edit) => {
    isEditMode = !!edit;
    if (panelTitle) panelTitle.textContent = title;
    if (panel) panel.classList.add('open');
    if (overlay) overlay.classList.add('open');
  };

  const closePanel = () => {
    if (panel) panel.classList.remove('open');
    if (overlay) overlay.classList.remove('open');
    if (form) form.reset();
    if (varNameInput) varNameInput.readOnly = false;
    if (pluginSelect) pluginSelect.disabled = false;
    if (isSecretCheckbox) isSecretCheckbox.checked = true;
    isEditMode = false;
  };

  return { openPanel, closePanel, form, pluginSelect, varNameInput, varValueInput, isSecretCheckbox, overlay, getEditMode: () => isEditMode };
};

const initEditSecretButtons = (ctx) => {
  on('click', '.edit-secret-btn', (e, btn) => {
    e.stopPropagation();
    ctx.pluginSelect.value = btn.getAttribute('data-plugin');
    ctx.pluginSelect.disabled = true;
    ctx.varNameInput.value = btn.getAttribute('data-var');
    ctx.varNameInput.readOnly = true;
    ctx.varValueInput.value = '';
    ctx.varValueInput.placeholder = 'Enter new value...';
    ctx.isSecretCheckbox.checked = btn.getAttribute('data-secret') === 'true';
    ctx.openPanel('Edit Secret', true);
  });
};

const initDeleteSecretButtons = () => {
  on('click', '.delete-secret-btn', (e, btn) => {
    e.stopPropagation();
    const pluginId = btn.getAttribute('data-plugin');
    const varName = btn.getAttribute('data-var');
    showConfirmDialog('Delete Secret', 'Delete "' + varName + '"?', 'Delete', async () => {
      try {
        await apiFetch('/user/secrets/' + encodeURIComponent(pluginId) + '/' + encodeURIComponent(varName), { method: 'DELETE' });
        showToast('Secret deleted', 'success'); setTimeout(() => location.reload(), 500);
      } catch (err) { showToast(err.message || 'Failed to delete', 'error'); }
    });
  });
};

const initSecretFormSubmit = (ctx) => {
  ctx.form?.addEventListener('submit', async (e) => {
    e.preventDefault();
    const pluginId = ctx.pluginSelect.value;
    const varName = ctx.varNameInput.value.trim();
    const varValue = ctx.varValueInput.value;
    const isSecret = ctx.isSecretCheckbox.checked;
    if (!pluginId || !varName) {
      showToast('Plugin and variable name are required', 'error');
    } else {
      const isEditMode = ctx.getEditMode();
      const url = isEditMode ? '/user/secrets/' + encodeURIComponent(pluginId) + '/' + encodeURIComponent(varName) : '/user/secrets';
      const method = isEditMode ? 'PUT' : 'POST';
      const body = isEditMode ? { var_value: varValue, is_secret: isSecret } : { plugin_id: pluginId, var_name: varName, var_value: varValue, is_secret: isSecret };
      try {
        await apiFetch(url, { method, body: JSON.stringify(body) });
        showToast(isEditMode ? 'Secret updated' : 'Secret created', 'success'); ctx.closePanel(); setTimeout(() => location.reload(), 500);
      } catch (err) { showToast(err.message || 'Failed to save secret', 'error'); }
    }
  });
};

export const initMySecretsPage = () => {
  const table = document.getElementById('my-secrets-table');
  if (table) {
    initExpandRows();
    initEntityBatch({
      tableSelector: '#my-secrets-table',
      rowSelector: 'tbody tr.clickable-row',
      idExtractor: (row) => row.getAttribute('data-entity-id'),
      deleteEndpoint: '/user/secrets/batch-delete',
      entityLabel: 'secret',
      bodyBuilder: (ids) => ({
        items: ids.map((id) => {
          const sep = id.indexOf('/');
          return { plugin_id: id.substring(0, sep), var_name: id.substring(sep + 1) };
        }),
      }),
    });
    const ctx = initSecretPanel();
    document.getElementById('add-secret-btn')?.addEventListener('click', () => ctx.openPanel('Add Secret', false));
    document.getElementById('secret-panel-close')?.addEventListener('click', ctx.closePanel);
    document.getElementById('secret-cancel-btn')?.addEventListener('click', ctx.closePanel);
    ctx.overlay?.addEventListener('click', ctx.closePanel);
    initEditSecretButtons(ctx);
    initDeleteSecretButtons();
    initSecretFormSubmit(ctx);
    initSearch(table);
  }
};

initMySecretsPage();
