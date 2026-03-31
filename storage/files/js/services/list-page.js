import { apiFetch, BASE } from './api.js';
import { showToast } from './toast.js';
import { on } from './events.js';
import { closeAllMenus } from './dropdown.js';
import { showDeleteConfirmDialog, closeDeleteConfirm } from './confirm.js';
import { initTableSort } from './table-sort.js';
import { formDataToObject } from '../utils/form.js';

export { initTableSort };

export const initListPage = (entityType, searchInputId, opts = {}) => {
  const searchAttr = opts.searchAttr || 'data-name';
  initTableSort();
  on('input', '#' + searchInputId, (e, el) => {
    clearTimeout(el._debounceTimer);
    el._debounceTimer = setTimeout(() => {
      const q = el.value.toLowerCase().trim();
      for (const row of document.querySelectorAll('.data-table tbody tr')) {
        const searchVal = row.getAttribute(searchAttr) || row.textContent.toLowerCase();
        row.hidden = q && !searchVal.includes(q);
      }
    }, 200);
  });
  on('click', '[data-action="delete"]', (e, deleteBtn) => {
    closeAllMenus();
    const entityId = deleteBtn.getAttribute('data-entity-id');
    const deleteType = deleteBtn.getAttribute('data-entity-type') || entityType;
    const title = 'Delete ' + deleteType.charAt(0).toUpperCase() + deleteType.slice(1) + '?';
    showDeleteConfirmDialog(title, entityId);
  }, { exclusive: true });
  on('click', '[data-confirm-delete]', (e, confirmBtn) => {
    performDelete(entityType, confirmBtn.getAttribute('data-confirm-delete'), confirmBtn, opts);
  }, { exclusive: true });
  on('click', '[data-confirm-cancel]', () => closeDeleteConfirm(), { exclusive: true });
  on('click', '#delete-confirm', (e, el) => { if (e.target === el) closeDeleteConfirm(); }, { exclusive: true });
  on('change', '[data-action="toggle"]', (e, toggle) => handleToggle(toggle, entityType, opts));
};

const performDelete = async (entityType, entityId, confirmBtn, opts) => {
  confirmBtn.disabled = true;
  confirmBtn.textContent = 'Deleting...';
  const apiPath = opts.deleteApiPath
    ? opts.deleteApiPath(entityId)
    : '/' + entityType + 's/' + encodeURIComponent(entityId);
  try {
    await apiFetch(apiPath, { method: 'DELETE' });
    showToast(entityType + ' deleted', 'success');
    closeDeleteConfirm();
    window.location.reload();
  } catch (err) {
    showToast(err.message || 'Failed to delete ' + entityType, 'error');
    confirmBtn.disabled = false;
    confirmBtn.textContent = 'Delete';
  }
};

const updateRowBadge = (row, enabled) => {
  if (row) {
    row.setAttribute('data-enabled', enabled ? 'enabled' : 'disabled');
    const badge = row.querySelector('.badge-green, .badge-gray');
    if (badge && (badge.textContent === 'Active' || badge.textContent === 'Disabled')) {
      badge.className = 'badge ' + (enabled ? 'badge-green' : 'badge-gray');
      badge.textContent = enabled ? 'Active' : 'Disabled';
    }
  }
};

const handleToggle = async (toggle, entityType, opts) => {
  const id = toggle.getAttribute('data-entity-id');
  const toggleType = toggle.getAttribute('data-entity-type') || entityType;
  const enabled = toggle.checked;
  const apiPath = opts.toggleApiPath
    ? opts.toggleApiPath(id)
    : '/' + toggleType + 's/' + encodeURIComponent(id);
  const row = toggle.closest('tr');
  updateRowBadge(row, enabled);
  try {
    await apiFetch(apiPath, { method: 'PUT', body: JSON.stringify({ enabled }) });
    showToast(toggleType + ' ' + (enabled ? 'enabled' : 'disabled'), 'success');
  } catch (err) {
    showToast(err.message || 'Failed to toggle ' + toggleType, 'error');
    toggle.checked = !enabled;
    updateRowBadge(row, !enabled);
  }
};

export const initEditForm = (formId, opts = {}) => {
  const form = document.getElementById(formId);
  if (form) {
    const apiPath = form.getAttribute('data-api-path') || '';
    const entity = form.getAttribute('data-entity') || 'item';
    const idField = form.getAttribute('data-id-field') || 'id';
    const listPath = form.getAttribute('data-list-path') || '/';
    const existingId = form.querySelector('[name="' + idField + '"]');
    const isEdit = existingId?.readOnly && existingId.value;
    form.addEventListener('submit', async (e) => {
      e.preventDefault();
      const formData = new FormData(form);
      const body = opts.buildBody ? opts.buildBody(form, formData) : formDataToObject(formData);
      const url = isEdit ? apiPath + encodeURIComponent(existingId.value) : apiPath.replace(/\/$/, '');
      const method = isEdit ? 'PUT' : 'POST';
      const submitBtn = form.querySelector('[type="submit"]');
      if (submitBtn) { submitBtn.disabled = true; submitBtn.textContent = 'Saving...'; }
      try {
        await apiFetch(url, { method, body: JSON.stringify(body) });
        showToast(entity + ' saved!', 'success');
        window.location.href = BASE + listPath;
      } catch (err) {
        showToast(err.message || 'Failed to save ' + entity, 'error');
        if (submitBtn) {
          submitBtn.disabled = false;
          submitBtn.textContent = isEdit ? 'Save Changes' : 'Create';
        }
      }
    });
  }
};
