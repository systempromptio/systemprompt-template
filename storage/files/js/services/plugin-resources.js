import { apiFetch } from './api.js';
import { showToast } from './toast.js';
import { loadAvailableItems, showAddItemsDialog } from './plugin-resources-helpers.js';

const getPluginIds = (data, resourceType) => {
  if (resourceType === 'skills') return (data.skills || []).map((s) => s.id);
  if (resourceType === 'agents') return (data.agents || []).map((a) => a.id);
  if (resourceType === 'mcp_servers') return data.mcp_servers || [];
  if (resourceType === 'hooks') return (data.hooks || []).map((h) => h.id);
  return [];
};

const updatePluginData = (data, resourceType, itemId, updatedIds) => {
  if (resourceType === 'skills') data.skills = data.skills.filter((s) => s.id !== itemId);
  else if (resourceType === 'agents') data.agents = data.agents.filter((a) => a.id !== itemId);
  else if (resourceType === 'mcp_servers') data.mcp_servers = updatedIds;
  else if (resourceType === 'hooks') data.hooks = data.hooks.filter((h) => h.id !== itemId);
};

const parsePluginDetail = (pluginId) => {
  const el = document.querySelector('[data-plugin-detail="' + pluginId + '"]');
  if (!el) return null;
  try { return { el, data: JSON.parse(el.textContent) }; } catch (_e) { return null; }
};

export const handleRemoveFromPlugin = async (btn) => {
  const itemId = btn.getAttribute('data-remove-from-plugin');
  const resourceType = btn.getAttribute('data-resource-type');
  const pluginId = btn.getAttribute('data-plugin-id');
  if (pluginId && pluginId !== 'custom') {
    const detail = parsePluginDetail(pluginId);
    if (detail) {
      const apiField = resourceType === 'mcp_servers' ? 'mcp_servers' : resourceType;
      const currentIds = getPluginIds(detail.data, resourceType);
      if (currentIds) {
        const updatedIds = currentIds.filter((id) => id !== itemId);
        const body = {}; body[apiField] = updatedIds; btn.disabled = true;
        try {
          await apiFetch('/plugins/' + encodeURIComponent(pluginId), { method: 'PUT', body: JSON.stringify(body) });
          const row = btn.closest('tr'); if (row) row.remove();
          updatePluginData(detail.data, resourceType, itemId, updatedIds);
          detail.el.textContent = JSON.stringify(detail.data);
          showToast('Removed from plugin', 'success');
        } catch (err) {
          btn.disabled = false;
          showToast(err.message || 'Failed to remove', 'error');
        }
      }
    }
  }
};

export const handleAddToPlugin = async (btn) => {
  const resourceType = btn.getAttribute('data-add-to-plugin');
  const pluginId = btn.getAttribute('data-plugin-id');
  if (pluginId && pluginId !== 'custom') {
    const detail = parsePluginDetail(pluginId);
    if (detail) {
      const currentIds = getPluginIds(detail.data, resourceType);
      const currentSet = {};
      for (const id of currentIds) currentSet[id] = true;
      const available = await loadAvailableItems(btn, resourceType, currentSet);
      if (available) showAddItemsDialog(available, resourceType, pluginId, currentIds);
    }
  }
};
