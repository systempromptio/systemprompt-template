import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { on } from '../services/events.js';
import { bindActionsPopup } from './admin-users-actions.js';

const openCreatePanel = () => {
  const overlay = document.getElementById('create-user-overlay');
  const panel = document.getElementById('create-user-panel');
  if (overlay && panel) {
    overlay.classList.add('open');
    panel.classList.add('open');
    const first = panel.querySelector('input');
    if (first) setTimeout(() => first.focus(), 350);
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
  for (const cb of document.querySelectorAll('#create-user-panel input[name="roles"]')) {
    cb.checked = false;
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
    const roles = Array.from(document.querySelectorAll('#create-user-panel input[name="roles"]:checked')).map((cb) => cb.value);
    if (userId) {
      try {
        await apiFetch('/users', {
          method: 'POST',
          body: JSON.stringify({ user_id: userId, display_name: displayName || userId, email, roles })
        });
        showToast('User created', 'success');
        closeCreatePanel();
        resetForm();
        window.location.reload();
      } catch (err) {
        showToast(err.message || 'Failed to create user', 'error');
      }
    } else {
      showToast('User ID is required', 'error');
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
