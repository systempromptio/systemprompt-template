import { apiFetch, BASE } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { TOTAL_STEPS, renderStep, renderHooks } from './admin-plugin-wizard-steps.js';
import { saveCurrentStepState, buildPluginBody } from './admin-plugin-wizard-review.js';

const state = {
  step: 1,
  form: { plugin_id: '', name: '', description: '', version: '0.1.0', category: '', author_name: '', keywords: '', roles: {} },
  selectedSkills: {}, selectedAgents: {}, selectedMcpServers: {}, hooks: []
};
let root = null;

const validateStep1 = () => {
  const { plugin_id, name } = state.form;
  if (!plugin_id?.trim()) { showToast('Plugin ID is required', 'error'); return false; }
  if (!/^[a-z0-9]+(-[a-z0-9]+)*$/.test(plugin_id.trim())) { showToast('Plugin ID must be kebab-case (e.g. my-plugin)', 'error'); return false; }
  if (!name?.trim()) { showToast('Name is required', 'error'); return false; }
  return true;
};

const submitImport = async () => {
  const urlInput = document.getElementById('import-url');
  const errorEl = document.getElementById('import-error');
  const submitBtn = document.getElementById('import-submit');
  const targetSelect = document.getElementById('import-target');
  if (urlInput && submitBtn) {
    const url = urlInput.value.trim();
    if (!url) {
      if (errorEl) { errorEl.textContent = 'URL is required'; errorEl.style.display = 'block'; }
    } else {
      submitBtn.disabled = true;
      submitBtn.textContent = 'Importing...';
      if (errorEl) errorEl.style.display = 'none';
      try {
        await apiFetch('/plugins/import', { method: 'POST', body: JSON.stringify({ url, import_target: targetSelect ? targetSelect.value : 'site' }) });
        showToast('Plugin imported successfully!', 'success');
        window.location.href = BASE + '/plugins/';
      } catch (err) {
        if (errorEl) { errorEl.textContent = err.message || 'Failed to import plugin'; errorEl.style.display = 'block'; }
        submitBtn.disabled = false;
        submitBtn.textContent = 'Import';
      }
    }
  }
};

const handleCreate = async () => {
  saveCurrentStepState(state, root);
  const btn = root.querySelector('#wizard-create');
  if (btn) { btn.disabled = true; btn.textContent = 'Creating...'; }
  try {
    await apiFetch('/plugins', { method: 'POST', body: JSON.stringify(buildPluginBody(state)) });
    showToast('Plugin created!', 'success');
    window.location.href = BASE + '/plugins/';
  } catch (err) {
    showToast(err.message || 'Failed to create plugin', 'error');
    if (btn) { btn.disabled = false; btn.textContent = 'Create Plugin'; }
  }
};

const handleImportClick = (e) => {
  if (e.target.closest('#btn-import-url')) {
    const m = document.getElementById('import-modal');
    if (m) { m.style.display = 'flex'; m.querySelector('#import-url')?.focus(); }
    return true;
  }
  if (e.target.closest('#import-cancel') || e.target.id === 'import-modal') {
    const m = document.getElementById('import-modal');
    if (m) m.style.display = 'none';
    return true;
  }
  if (e.target.closest('#import-submit')) { submitImport(); return true; }
  return false;
};

const handleNavClick = (e) => {
  if (e.target.closest('#wizard-next')) {
    saveCurrentStepState(state, root);
    if (state.step === 1 && !validateStep1()) return true;
    if (state.step < TOTAL_STEPS) { state.step++; renderStep(state, root); }
    return true;
  }
  if (e.target.closest('#wizard-prev')) {
    saveCurrentStepState(state, root);
    if (state.step > 1) { state.step--; renderStep(state, root); }
    return true;
  }
  if (e.target.closest('#wizard-create')) { handleCreate(); return true; }
  return false;
};

const handleHookClick = (e) => {
  if (e.target.closest('#btn-add-hook')) {
    saveCurrentStepState(state, root);
    state.hooks.push({ event: 'PostToolUse', matcher: '*', command: '', async: false });
    renderHooks(state);
    return true;
  }
  const rb = e.target.closest('[data-remove-hook]');
  if (rb) {
    saveCurrentStepState(state, root);
    const entry = rb.closest('.hook-entry');
    const hl = document.getElementById('hooks-list');
    if (entry && hl) {
      const idx = Array.from(hl.querySelectorAll('.hook-entry')).indexOf(entry);
      if (idx >= 0) state.hooks.splice(idx, 1);
      renderHooks(state);
    }
    return true;
  }
  return false;
};

const handleBulkSelect = (e) => {
  const sa = e.target.closest('[data-select-all]');
  if (sa) {
    const c = root.querySelector('[data-checklist="' + sa.getAttribute('data-select-all') + '"]');
    if (c) for (const cb of c.querySelectorAll('input[type="checkbox"]')) cb.checked = true;
  } else {
    const da = e.target.closest('[data-deselect-all]');
    if (da) {
      const c = root.querySelector('[data-checklist="' + da.getAttribute('data-deselect-all') + '"]');
      if (c) for (const cb of c.querySelectorAll('input[type="checkbox"]')) cb.checked = false;
    }
  }
};

const handleClick = (e) => {
  if (handleImportClick(e)) {
  } else if (handleNavClick(e)) {
  } else if (handleHookClick(e)) {
  } else {
    handleBulkSelect(e);
  }
};

export const initPluginWizard = () => {
  root = document.getElementById('plugin-create-content');
  if (root) {
    renderStep(state, root);
    root.addEventListener('click', handleClick);
    root.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' && e.target.id === 'import-url') { e.preventDefault(); submitImport(); }
      if (e.key === 'Escape') {
        const m = document.getElementById('import-modal');
        if (m) m.style.display = 'none';
      }
    });
  }
};
