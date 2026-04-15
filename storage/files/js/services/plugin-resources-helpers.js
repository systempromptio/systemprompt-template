import { apiFetch } from './api.js';
import { showToast } from './toast.js';

export const loadAvailableItems = async (btn, resourceType, currentSet) => {
  btn.disabled = true;
  try {
    const endpoint = '/' + resourceType.replace('_', '-');
    const data = await apiFetch(endpoint);
    if (!data) return null;

    const items = Array.isArray(data) ? data : data[resourceType] || data.items || [];
    return items.filter((item) => {
      const id = item.id || item.name || item;
      return !currentSet[id];
    });
  } catch (err) {
    showToast('Failed to load ' + resourceType.replace('_', ' '), 'error');
    return null;
  } finally {
    btn.disabled = false;
  }
};

export const showAddItemsDialog = (available, resourceType, pluginId, currentIds) => {
  if (!available.length) {
    showToast('No ' + resourceType.replace('_', ' ') + ' available to add', 'info');
    return;
  }

  const overlay = document.createElement('div');
  overlay.className = 'sp-add-items-overlay';

  const dialog = document.createElement('div');
  dialog.className = 'sp-add-items-dialog';

  const title = document.createElement('h3');
  title.className = 'sp-add-items-header';
  title.textContent = 'Add ' + resourceType.replace('_', ' ');
  dialog.append(title);

  const list = document.createElement('div');
  list.className = 'sp-add-items-list';
  const selectedItems = new Set();

  for (const item of available) {
    const id = item.id || item.name || item;
    const label = item.name || item.id || item;
    const row = document.createElement('label');
    row.className = 'sp-add-items-row';
    const cb = document.createElement('input');
    cb.type = 'checkbox';
    cb.value = id;
    cb.addEventListener('change', () => {
      if (cb.checked) selectedItems.add(id);
      else selectedItems.delete(id);
    });
    const span = document.createElement('span');
    span.textContent = label;
    row.append(cb, span);
    list.append(row);
  }
  dialog.append(list);

  const actions = document.createElement('div');
  actions.className = 'sp-add-items-actions';

  const cancelBtn = document.createElement('button');
  cancelBtn.type = 'button';
  cancelBtn.className = 'btn btn-secondary';
  cancelBtn.textContent = 'Cancel';
  cancelBtn.addEventListener('click', () => overlay.remove());

  const addBtn = document.createElement('button');
  addBtn.type = 'button';
  addBtn.className = 'btn btn-primary';
  addBtn.textContent = 'Add';
  addBtn.addEventListener('click', async () => {
    if (selectedItems.size === 0) return;
    addBtn.disabled = true;
    addBtn.textContent = 'Adding...';
    const apiField = resourceType === 'mcp_servers' ? 'mcp_servers' : resourceType;
    const updatedIds = [...currentIds, ...selectedItems];
    const body = {};
    body[apiField] = updatedIds;
    try {
      await apiFetch('/plugins/' + encodeURIComponent(pluginId), {
        method: 'PUT',
        body: JSON.stringify(body),
      });
      showToast('Added to plugin', 'success');
      overlay.remove();
      window.location.reload();
    } catch (err) {
      addBtn.disabled = false;
      addBtn.textContent = 'Add';
    }
  });

  actions.append(cancelBtn, addBtn);
  dialog.append(actions);
  overlay.append(dialog);
  overlay.addEventListener('click', (e) => { if (e.target === overlay) overlay.remove(); });
  document.body.append(overlay);
};
