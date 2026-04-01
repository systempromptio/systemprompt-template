import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';
import { initExpandRows, initTableSearch, initTableFilter } from '../services/entity-common.js';
import { on } from '../services/events.js';

const deletePlugin = async (pluginId) => {
  try {
    await apiFetch('/user/plugins/' + encodeURIComponent(pluginId), { method: 'DELETE' });
    showToast('Plugin deleted', 'success');
    setTimeout(() => window.location.reload(), 500);
  } catch (err) {
    showToast(err.message || 'Failed to delete plugin', 'error');
  }
};

export const initMyPluginsPage = () => {
  if (document.getElementById('my-plugins-table')) {
    initExpandRows('#my-plugins-table');
    initTableSearch('my-plugins-search', '#my-plugins-table');
    initTableFilter('my-plugins-category-filter', '#my-plugins-table', 'data-category');
    on('click', '[data-delete-plugin]', (e, btn) => {
      e.stopPropagation();
      const pluginId = btn.getAttribute('data-delete-plugin');
      showConfirmDialog('Delete Plugin', 'This action cannot be undone.', 'Delete', () => deletePlugin(pluginId));
    });
  }
};

initMyPluginsPage();
