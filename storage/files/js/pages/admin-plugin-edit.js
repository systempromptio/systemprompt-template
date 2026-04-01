import { apiFetch, BASE } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';
import { getCheckedValues, attachFilterHandlers } from '../utils/form.js';

export const initPluginEditForm = () => {
  const form = document.getElementById('plugin-edit-form');
  if (form) {
    const pluginIdInput = form.querySelector('input[name="plugin_id"]');
    const pluginId = pluginIdInput ? pluginIdInput.value : '';

    form.addEventListener('submit', async (e) => {
      e.preventDefault();
      const formData = new FormData(form);
      const keywordsRaw = formData.get('keywords') || '';
      const keywords = keywordsRaw.split(',').map((t) => t.trim()).filter(Boolean);
      const body = {
        name: formData.get('name'),
        description: formData.get('description') || '',
        version: formData.get('version') || '0.1.0',
        category: formData.get('category') || '',
        enabled: !!form.querySelector('input[name="enabled"]').checked,
        keywords,
        author: { name: formData.get('author_name') || '' },
        roles: getCheckedValues(form, 'roles'),
        skills: getCheckedValues(form, 'skills'),
        agents: getCheckedValues(form, 'agents'),
        mcp_servers: getCheckedValues(form, 'mcp_servers')
      };
      const submitBtn = form.querySelector('[type="submit"]');
      if (submitBtn) { submitBtn.disabled = true; submitBtn.textContent = 'Saving...'; }
      try {
        await apiFetch('/plugins/' + encodeURIComponent(pluginId), {
          method: 'PUT',
          body: JSON.stringify(body)
        });
        showToast('Plugin saved!', 'success');
        window.location.href = BASE + '/plugins/';
      } catch (err) {
        showToast(err.message || 'Failed to save plugin', 'error');
        if (submitBtn) { submitBtn.disabled = false; submitBtn.textContent = 'Save Changes'; }
      }
    });

    const deleteBtn = document.getElementById('btn-delete-plugin');
    if (deleteBtn) {
      deleteBtn.addEventListener('click', () => {
        showConfirmDialog('Delete Plugin?', 'Are you sure you want to delete this plugin? This cannot be undone.', 'Delete', async () => {
          deleteBtn.disabled = true;
          deleteBtn.textContent = 'Deleting...';
          try {
            await apiFetch('/plugins/' + encodeURIComponent(pluginId), { method: 'DELETE' });
            showToast('Plugin deleted', 'success');
            window.location.href = BASE + '/plugins/';
          } catch (err) {
            showToast(err.message || 'Failed to delete plugin', 'error');
            deleteBtn.disabled = false;
            deleteBtn.textContent = 'Delete';
          }
        });
      });
    }

    attachFilterHandlers(form);
  }
};
