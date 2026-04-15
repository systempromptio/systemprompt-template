import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { on } from '../services/events.js';

export const initMyMarketplacePage = () => {
  const grid = document.getElementById('my-marketplace-grid');
  if (grid) {
    const searchInput = document.getElementById('my-marketplace-search');
    if (searchInput) {
      let timer;
      searchInput.addEventListener('input', () => {
        clearTimeout(timer);
        timer = setTimeout(() => {
          const query = searchInput.value.toLowerCase().trim();
          for (const card of grid.querySelectorAll('.marketplace-card')) {
            card.style.display = (!query || card.textContent.toLowerCase().includes(query)) ? '' : 'none';
          }
        }, 200);
      });
    }

    on('click', '[data-customize-plugin]', async (e, btn) => {
      const pluginId = btn.getAttribute('data-customize-plugin');
      btn.disabled = true;
      btn.textContent = 'Customizing...';
      try {
        const data = await apiFetch('/user/fork/plugin', {
          method: 'POST',
          body: JSON.stringify({ org_plugin_id: pluginId }),
        });
        showToast('Plugin customized successfully', 'success');
        if (data.plugin?.plugin_id) {
          setTimeout(() => { window.location.href = '/admin/my/plugins/edit?id=' + encodeURIComponent(data.plugin.plugin_id); }, 500);
        } else {
          setTimeout(() => window.location.reload(), 500);
        }
      } catch (err) {
        showToast(err.message || 'Failed to customize plugin', 'error');
        btn.disabled = false;
        btn.textContent = 'Customize';
      }
    });
  }
};

initMyMarketplacePage();
