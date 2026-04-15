import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';
import { on } from '../services/events.js';

const updateCounter = (id, delta) => {
  const el = document.getElementById(id);
  if (el) {
    const val = Math.max(0, (parseInt(el.textContent, 10) || 0) + delta);
    el.textContent = val;
    if (id === 'plugin-counter') el.style.display = val > 0 ? '' : 'none';
  }
};

const updateAddedBadge = (delta) => {
  const badge = document.getElementById('browse-added-count');
  if (badge) {
    const match = badge.textContent.match(/(\d+)/);
    badge.textContent = Math.max(0, (match ? parseInt(match[1], 10) : 0) + delta) + ' added';
  }
};

const replaceActions = (btn, pluginId, added) => {
  const actions = btn.closest('.plugin-selection-card')?.querySelector('.plugin-card-actions');
  if (actions) {
    while (actions.firstChild) actions.firstChild.remove();
    if (added) {
      const badge = document.createElement('span');
      badge.className = 'badge badge-green';
      badge.style.padding = 'var(--sp-space-1) var(--sp-space-3)';
      badge.textContent = 'Added';
      const removeBtn = document.createElement('button');
      removeBtn.className = 'btn btn-secondary btn-sm';
      removeBtn.setAttribute('data-remove-plugin', pluginId);
      removeBtn.style.marginLeft = 'var(--sp-space-2)';
      removeBtn.textContent = 'Remove';
      actions.append(badge);
      actions.append(removeBtn);
    } else {
      const addBtn = document.createElement('button');
      addBtn.className = 'btn btn-primary btn-sm';
      addBtn.setAttribute('data-add-plugin', pluginId);
      addBtn.textContent = 'Add';
      actions.append(addBtn);
    }
  }
};

const handleAdd = async (btn) => {
  const pluginId = btn.getAttribute('data-add-plugin');
  btn.disabled = true;
  btn.textContent = 'Adding...';
  try {
    await apiFetch('/user/fork/plugin', {
      method: 'POST',
      body: JSON.stringify({ org_plugin_id: pluginId }),
    });
    showToast('Plugin added to your workspace', 'success');
    replaceActions(btn, pluginId, true);
    updateCounter('plugin-counter', 1);
    updateCounter('plugin-stat-count', 1);
    updateAddedBadge(1);
  } catch (err) {
    showToast(err.message || 'Failed to add plugin', 'error');
    btn.disabled = false;
    btn.textContent = 'Add';
  }
};

const handleRemove = (btn) => {
  const pluginId = btn.getAttribute('data-remove-plugin');
  showConfirmDialog('Remove Plugin', 'Remove this plugin and all its skills, agents, and connectors?', 'Remove', async () => {
    btn.disabled = true;
    btn.textContent = 'Removing...';
    try {
      await apiFetch('/user/plugins/' + encodeURIComponent(pluginId), { method: 'DELETE' });
      showToast('Plugin removed from your workspace', 'success');
      replaceActions(btn, pluginId, false);
      updateCounter('plugin-counter', -1);
      updateCounter('plugin-stat-count', -1);
      updateAddedBadge(-1);
    } catch (err) {
      showToast(err.message || 'Failed to remove plugin', 'error');
      btn.disabled = false;
      btn.textContent = 'Remove';
    }
  });
};

export const initBrowsePluginsPage = () => {
  const grid = document.getElementById('browse-plugins-grid');
  if (grid) {
    on('click', '[data-add-plugin]', (e, btn) => {
      e.preventDefault(); e.stopPropagation(); handleAdd(btn);
    });
    on('click', '[data-remove-plugin]', (e, btn) => {
      e.preventDefault(); e.stopPropagation(); handleRemove(btn);
    });

    const searchInput = document.getElementById('browse-plugins-search');
    if (searchInput) {
      let timer;
      searchInput.addEventListener('input', () => {
        clearTimeout(timer);
        timer = setTimeout(() => {
          const query = searchInput.value.toLowerCase().trim();
          for (const card of grid.querySelectorAll('.plugin-selection-card')) {
            card.style.display = (!query || card.textContent.toLowerCase().includes(query)) ? '' : 'none';
          }
        }, 200);
      });
    }

    const catFilter = document.getElementById('browse-plugins-category-filter');
    if (catFilter) {
      catFilter.addEventListener('change', () => {
        const val = catFilter.value;
        for (const card of grid.querySelectorAll('.plugin-selection-card')) {
          card.style.display = (!val || (card.getAttribute('data-category') || '') === val) ? '' : 'none';
        }
      });
    }
  }
};

initBrowsePluginsPage();
