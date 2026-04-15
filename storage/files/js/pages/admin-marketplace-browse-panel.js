import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';

let mktPlugins = [];
const setMktPlugins = (data) => { mktPlugins = data; };

const handleSaveVisibility = async (overlay, modalRules, pluginId) => {
  const saveBtn = overlay.querySelector('#vis-save');
  saveBtn.disabled = true;
  saveBtn.textContent = 'Saving...';
  try {
    await apiFetch('/marketplace-plugins/' + encodeURIComponent(pluginId) + '/visibility', {
      method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ rules: modalRules })
    });
    showToast('Visibility updated', 'success');
    overlay.remove();
    window.location.reload();
  } catch (err) {
    showToast(err.message || 'Failed to save visibility', 'error');
    saveBtn.disabled = false;
    saveBtn.textContent = 'Save';
  }
};

const handleModalClick = (overlay, modalRules, pluginId, refresh) => async (e) => {
  if (e.target === overlay || e.target.closest('[data-confirm-cancel]')) {
    overlay.remove();
  } else {
    const rb = e.target.closest('[data-remove-rule]');
    if (rb) {
      modalRules.splice(parseInt(rb.getAttribute('data-remove-rule'), 10), 1);
      refresh();
    } else if (e.target.closest('#vis-add-rule')) {
      handleAddRule(overlay, modalRules, refresh);
    } else if (e.target.closest('#vis-save')) {
      await handleSaveVisibility(overlay, modalRules, pluginId);
    }
  }
};

const showVisibilityModal = (pluginId) => {
  const plugin = mktPlugins.find((p) => p.id === pluginId);
  if (plugin) {
    const modalRules = (plugin.visibility_rules || []).slice();
    const overlay = document.createElement('div');
    overlay.className = 'sp-visibility-overlay';
    overlay.id = 'visibility-modal';
    const refresh = () => {
      const c = overlay.querySelector('#visibility-rules-list');
      if (c) { c.textContent = ''; c.append(renderRulesList(modalRules)); }
    };
    overlay.append(buildModalDialog(plugin, modalRules));
    document.body.append(overlay);
    overlay.addEventListener('click', handleModalClick(overlay, modalRules, pluginId, refresh));
  }
};
