import { apiFetch } from './api.js';
import { showToast } from './toast.js';
import { validateContent, buildFileListEl, buildEditorEl } from './skill-files-editor.js';
import { buildModalEl } from './skill-files-modal.js';

let overlay = null;
let currentSkillId = null;
let currentSkillName = '';
let files = [];
let selectedFile = null;

const runValidation = () => {
  if (overlay && selectedFile) {
    const editor = overlay.querySelector('#skill-file-editor');
    const badge = overlay.querySelector('#skill-file-validation');
    if (editor && badge) {
      const err = validateContent(editor.value, selectedFile.language);
      badge.textContent = err || '';
      badge.classList.toggle('sp-skill-files-validation-error', !!err);
    }
  }
};

const handleSave = async () => {
  if (selectedFile) {
    const editor = overlay?.querySelector('#skill-file-editor');
    if (editor) {
      const content = editor.value;
      const err = validateContent(content, selectedFile.language);
      if (err) { showToast('Fix validation error before saving: ' + err, 'error'); return; }
      const saveBtn = overlay.querySelector('#skill-file-save');
      if (saveBtn) { saveBtn.disabled = true; saveBtn.textContent = 'Saving...'; }
      try {
        await apiFetch('/skills/' + encodeURIComponent(currentSkillId) + '/files/' + selectedFile.file_path, {
          method: 'PUT',
          body: JSON.stringify({ content }),
          headers: { 'Content-Type': 'application/json' },
        });
        selectedFile.content = content;
        selectedFile.size_bytes = new Blob([content]).size;
        showToast('File saved', 'success');
      } catch (saveErr) {
        showToast(saveErr.message || 'Save failed', 'error');
      } finally {
        if (saveBtn) { saveBtn.disabled = false; saveBtn.textContent = 'Save'; }
      }
    }
  }
};

const refreshUI = () => {
  if (overlay) {
    const listEl = overlay.querySelector('#skill-files-list');
    const editorEl = overlay.querySelector('#skill-files-editor');
    if (listEl) { listEl.textContent = ''; listEl.append(buildFileListEl(files, selectedFile)); }
    if (editorEl) { editorEl.textContent = ''; editorEl.append(buildEditorEl(selectedFile)); }
    bindFileItems();
    const saveBtn = overlay.querySelector('#skill-file-save');
    if (saveBtn) saveBtn.addEventListener('click', handleSave);
    const editorInput = overlay.querySelector('#skill-file-editor');
    if (editorInput) { editorInput.addEventListener('input', runValidation); runValidation(); }
  }
};

const bindFileItems = () => {
  if (overlay) {
    for (const item of overlay.querySelectorAll('.skill-file-item')) {
      item.addEventListener('click', () => {
        selectedFile = files.find((f) => f.id === item.getAttribute('data-file-id')) || null;
        refreshUI();
      });
    }
  }
};

const loadFiles = async () => {
  try {
    files = await apiFetch('/skills/' + encodeURIComponent(currentSkillId) + '/files');
    if (!Array.isArray(files)) files = [];
  } catch (err) { files = []; showToast(err.message || 'Failed to load files', 'error'); }
};

const handleSync = async () => {
  const syncBtn = overlay?.querySelector('#skill-files-sync');
  if (syncBtn) { syncBtn.disabled = true; syncBtn.textContent = 'Syncing...'; }
  try {
    const result = await apiFetch('/skills/sync-files', { method: 'POST' });
    showToast('Synced: ' + (result.created || 0) + ' created, ' + (result.updated || 0) + ' updated', 'success');
    await loadFiles();
    refreshUI();
  } catch (err) {
    showToast(err.message || 'Sync failed', 'error');
    if (syncBtn) { syncBtn.disabled = false; syncBtn.textContent = 'Sync Files'; }
  }
};

const bindEvents = () => {
  if (overlay) {
    overlay.querySelector('#skill-files-close')?.addEventListener('click', closeModal);
    overlay.querySelector('#skill-files-sync')?.addEventListener('click', handleSync);
    overlay.querySelector('#skill-file-save')?.addEventListener('click', handleSave);
    bindFileItems();
    const editorInput = overlay.querySelector('#skill-file-editor');
    if (editorInput) { editorInput.addEventListener('input', runValidation); runValidation(); }
  }
};

const closeModal = () => {
  if (overlay) { overlay.remove(); overlay = null; }
  currentSkillId = null; currentSkillName = ''; files = []; selectedFile = null;
};

export const openSkillFiles = async (skillId, skillName) => {
  closeModal();
  currentSkillId = skillId;
  currentSkillName = skillName || skillId;
  overlay = document.createElement('div');
  overlay.className = 'sp-skill-files-overlay';
  const panel = document.createElement('div');
  panel.className = 'sp-skill-files-panel';
  const loading = document.createElement('div');
  loading.className = 'sp-skill-files-loading';
  loading.textContent = 'Loading files...';
  panel.append(loading);
  overlay.append(panel);
  document.body.append(overlay);
  overlay.addEventListener('click', (e) => { if (e.target === overlay) closeModal(); });
  await loadFiles();
  const p = overlay?.querySelector('.sp-skill-files-panel');
  if (p) { p.textContent = ''; p.append(buildModalEl(currentSkillName, files, selectedFile)); }
  bindEvents();
};

export const closeSkillFiles = closeModal;
