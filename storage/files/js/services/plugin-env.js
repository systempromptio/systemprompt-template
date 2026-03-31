import { apiFetch } from './api.js';
import { showToast } from './toast.js';
import { mergeDefsWithValues, buildVarListEl } from './plugin-env-ui.js';

let overlay = null;
let currentPluginId = null;
let currentPluginName = '';

const buildModalEl = (vars) => {
  const frag = document.createDocumentFragment();
  const h3 = document.createElement('h3');
  h3.className = 'sp-env-title';
  h3.textContent = currentPluginName + ' \u2014 Environment Variables';
  frag.append(h3);
  const scroll = document.createElement('div');
  scroll.className = 'sp-env-scroll';
  scroll.append(buildVarListEl(vars));
  frag.append(scroll);
  const actions = document.createElement('div');
  actions.className = 'form-actions sp-env-actions';
  const closeBtn = document.createElement('button');
  closeBtn.className = 'btn btn-secondary';
  closeBtn.id = 'plugin-env-close';
  closeBtn.textContent = 'Close';
  const saveBtn = document.createElement('button');
  saveBtn.className = 'btn btn-primary';
  saveBtn.id = 'plugin-env-save';
  saveBtn.textContent = 'Save';
  actions.append(closeBtn);
  actions.append(saveBtn);
  frag.append(actions);
  return frag;
};

const handleSave = async () => {
  const saveBtn = overlay?.querySelector('#plugin-env-save');
  if (saveBtn) { saveBtn.disabled = true; saveBtn.textContent = 'Saving...'; }
  try {
    const inputs = overlay.querySelectorAll('.plugin-env-input');
    const payload = [];
    for (const input of inputs) {
      const name = input.getAttribute('data-var-name');
      const isSecret = input.getAttribute('data-is-secret') === '1';
      const value = input.value;
      if (isSecret && value === '\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022') continue;
      payload.push({ var_name: name, var_value: value, is_secret: isSecret });
    }
    await apiFetch('/plugins/' + encodeURIComponent(currentPluginId) + '/env', {
      method: 'PUT',
      body: JSON.stringify({ variables: payload }),
      headers: { 'Content-Type': 'application/json' },
    });
    window.dispatchEvent(new CustomEvent('env-saved', { detail: { pluginId: currentPluginId } }));
    if (saveBtn) {
      saveBtn.textContent = 'Saved';
      saveBtn.classList.add('sp-env-save-success');
    }
    showToast('Environment variables saved', 'success');
    setTimeout(() => closeModal(), 600);
  } catch (err) {
    showToast(err.message || 'Save failed', 'error');
    if (saveBtn) { saveBtn.disabled = false; saveBtn.textContent = 'Save'; }
  }
};

const bindEvents = () => {
  if (overlay) {
    overlay.querySelector('#plugin-env-close')?.addEventListener('click', closeModal);
    overlay.querySelector('#plugin-env-save')?.addEventListener('click', handleSave);
  }
};

const updatePanel = (vars) => {
  const panel = overlay?.querySelector('.confirm-dialog');
  if (panel) { panel.textContent = ''; panel.append(buildModalEl(vars)); }
  bindEvents();
};

const closeModal = () => {
  if (overlay) { overlay.remove(); overlay = null; }
  currentPluginId = null;
  currentPluginName = '';
};

export const openPluginEnv = async (pluginId, pluginName) => {
  closeModal();
  currentPluginId = pluginId;
  currentPluginName = pluginName || pluginId;
  overlay = document.createElement('div');
  overlay.className = 'confirm-overlay';
  const dialog = document.createElement('div');
  dialog.className = 'confirm-dialog sp-env-dialog';
  const loading = document.createElement('div');
  loading.className = 'sp-env-loading';
  loading.textContent = 'Loading...';
  dialog.append(loading);
  overlay.append(dialog);
  document.body.append(overlay);
  overlay.addEventListener('click', (e) => { if (e.target === overlay) closeModal(); });
  try {
    const data = await apiFetch('/plugins/' + encodeURIComponent(currentPluginId) + '/env');
    const merged = mergeDefsWithValues(data.definitions || [], data.stored || []);
    updatePanel(merged);
  } catch (err) {
    showToast(err.message || 'Failed to load env vars', 'error');
    updatePanel([]);
  }
};

export const closePluginEnv = closeModal;
