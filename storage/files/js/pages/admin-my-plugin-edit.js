import { initEditForm } from '../services/list-page.js';
import { apiFetch, BASE } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';

const init = () => {
  initEditForm('my-plugin-form', {
    buildBody: (form, formData) => {
      const keywordsRaw = formData.get('keywords') || '';
      const keywords = keywordsRaw.split(',').map(t => t.trim()).filter(Boolean);
      return {
        plugin_id: formData.get('plugin_id'),
        name: formData.get('name'),
        description: formData.get('description') || '',
        version: formData.get('version') || '1.0.0',
        category: formData.get('category') || '',
        keywords,
        author_name: formData.get('author_name') || '',
      };
    },
  });

  const deleteBtn = document.getElementById('btn-delete-my-plugin');
  if (deleteBtn) {
    deleteBtn.addEventListener('click', () => {
      const pluginId = document.querySelector('[name="plugin_id"]').value;
      showConfirmDialog(
        'Delete Plugin',
        'Delete this plugin? This cannot be undone.',
        'Delete',
        async () => {
          try {
            await apiFetch(
              `/user/plugins/${encodeURIComponent(pluginId)}`,
              { method: 'DELETE' },
            );
            showToast('Plugin deleted', 'success');
            window.location.href = `${BASE}/my/plugins/`;
          } catch {
          }
        },
      );
    });
  }
};

init();
