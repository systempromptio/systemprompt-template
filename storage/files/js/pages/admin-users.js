import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { copyLinkPath } from '../services/clipboard.js';
import { on } from '../services/events.js';
import { bindActionsPopup } from './admin-users-actions.js';

const DEFAULT_DEPARTMENT_OPTION = '-- Select --';

const loadDepartments = async () => {
  const select = document.getElementById('new-user-dept');
  if (!select || select.dataset.loaded === 'true') return;
  try {
    const departments = await apiFetch('/management/departments');
    const options = (departments || [])
      .map((d) => `<option value="${d.name}">${d.name}</option>`)
      .join('');
    select.innerHTML = `<option value="">${DEFAULT_DEPARTMENT_OPTION}</option>${options}`;
    select.dataset.loaded = 'true';
  } catch {
  }
};

const openCreatePanel = async () => {
  const overlay = document.getElementById('create-user-overlay');
  const panel = document.getElementById('create-user-panel');
  if (overlay && panel) {
    overlay.classList.add('open');
    panel.classList.add('open');
    const first = panel.querySelector('input');
    if (first) setTimeout(() => first.focus(), 350);
    await loadDepartments();
  }
};

const closeCreatePanel = () => {
  const overlay = document.getElementById('create-user-overlay');
  const panel = document.getElementById('create-user-panel');
  if (panel) panel.classList.remove('open');
  if (overlay) overlay.classList.remove('open');
};

const resetForm = () => {
  for (const id of ['new-user-id', 'new-user-name', 'new-user-email']) {
    const el = document.getElementById(id);
    if (el) el.value = '';
  }
  const dept = document.getElementById('new-user-dept');
  if (dept) dept.value = '';
  for (const cb of document.querySelectorAll('#create-user-panel input[name="roles"]')) {
    cb.checked = cb.value === 'user';
  }
};

const bindCreatePanel = () => {
  on('click', '#create-user-overlay', () => { closeCreatePanel(); });
  on('click', '#create-user-panel .panel-close', () => { closeCreatePanel(); });
  on('click', '#create-user-panel [data-action="cancel"]', () => { closeCreatePanel(); });
  on('click', '#create-user-panel [data-action="save"]', async () => {
    const userId = document.getElementById('new-user-id').value.trim();
    const displayName = document.getElementById('new-user-name').value.trim();
    const email = document.getElementById('new-user-email').value.trim();
    const department = (document.getElementById('new-user-dept')?.value || '').trim();
    const roles = Array.from(document.querySelectorAll('#create-user-panel input[name="roles"]:checked')).map((cb) => cb.value);
    if (!userId) {
      showToast('User ID is required', 'error');
      return;
    }
    const body = { user_id: userId, display_name: displayName || userId, email, roles };
    if (department) body.department = department;
    try {
      const created = await apiFetch('/users', { method: 'POST', body: JSON.stringify(body) });
      showToast('User created', 'success');
      closeCreatePanel();
      resetForm();
      if (created && created.invite_path) {
        await copyLinkPath(created.invite_path, 'User created — sign-in link copied to clipboard');
      } else if (created && created.invite_note) {
        showToast(created.invite_note, 'info');
      }
      window.location.reload();
    } catch (err) {
      showToast(err.message || 'Failed to create user', 'error');
    }
  });
};

export const initUsersPage = () => {
  const page = document.querySelector('[data-page="users"]') || document.getElementById('users-table');
  if (page) {
    bindCreatePanel();
    bindActionsPopup();
    const createBtn = document.querySelector('[data-action="create-user"]');
    if (createBtn) createBtn.addEventListener('click', openCreatePanel);
  }
};

initUsersPage();
