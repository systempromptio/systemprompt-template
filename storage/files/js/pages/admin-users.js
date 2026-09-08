// The roster's write actions. Filtering, sorting and paging are links the
// server answers, so nothing here touches the table's contents.

import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';
import { on, initDelegation } from '../services/events.js';

const panelIds = ['new-user-id', 'new-user-name', 'new-user-email'];

const setPanelOpen = (open) => {
  const overlay = document.getElementById('create-user-overlay');
  const panel = document.getElementById('create-user-panel');
  overlay?.classList.toggle('is-open', open);
  panel?.classList.toggle('is-open', open);
  if (open) panel?.querySelector('input')?.focus();
};

const resetPanel = () => {
  for (const id of panelIds) {
    const el = document.getElementById(id);
    if (el) el.value = '';
  }
  for (const cb of document.querySelectorAll('#create-user-panel input[name="roles"]')) {
    cb.checked = cb.value === 'user';
  }
};

const createUser = async () => {
  const userId = document.getElementById('new-user-id')?.value.trim();
  if (!userId) {
    showToast('User ID is required', 'error');
    return;
  }
  const email = document.getElementById('new-user-email')?.value.trim();
  if (!email || !document.getElementById('new-user-email').checkValidity()) {
    showToast('A valid email address is required', 'error');
    return;
  }
  const roles = Array.from(
    document.querySelectorAll('#create-user-panel input[name="roles"]:checked'),
    (cb) => cb.value,
  );
  const body = {
    user_id: userId,
    display_name: document.getElementById('new-user-name')?.value.trim() || userId,
    email,
    roles,
  };
  try {
    const created = await apiFetch('/users', { method: 'POST', body: JSON.stringify(body) });
    showToast('User created', 'success');
    setPanelOpen(false);
    resetPanel();
    if (created?.invite_note) showToast(created.invite_note, 'info');
    window.location.reload();
  } catch (err) {
    showToast(err?.message ?? 'Failed to create the user', 'error');
  }
};

const selectedIds = () =>
  Array.from(
    document.querySelectorAll('[data-select-user]:checked'),
    (cb) => cb.dataset.selectUser,
  );

// The bulk editor states the whole target role set, because that is the only
// shape PUT /users/{id}/roles accepts and the only one the server can rule on.
const applyBulkRoles = async (ids, roles) => {
  let changed = 0;
  for (const id of ids) {
    try {
      await apiFetch(`/users/${encodeURIComponent(id)}/roles`, {
        method: 'PUT',
        body: JSON.stringify({ roles }),
      });
      changed += 1;
    } catch (err) {
      showToast(`${id}: ${err?.message ?? 'refused'}`, 'error');
    }
  }
  showToast(`${changed} of ${ids.length} accounts updated`, changed ? 'success' : 'error');
  if (changed) window.location.reload();
};

const bindBulk = () => {
  const button = document.querySelector('[data-action="bulk-roles"]');
  if (!button) return;
  const sync = () => {
    const count = selectedIds().length;
    button.disabled = count === 0;
    button.textContent = count ? `Change roles (${count})` : 'Change roles';
  };
  on('change', '[data-select-user]', sync);
  button.addEventListener('click', () => {
    const ids = selectedIds();
    if (!ids.length) return;
    showConfirmDialog(
      `Set roles on ${ids.length} account${ids.length === 1 ? '' : 's'}`,
      'Every selected account is given exactly the "user" role. Any other grant they hold is removed.',
      'Set roles',
      () => applyBulkRoles(ids, ['user']),
    );
  });
  sync();
};

const init = () => {
  initDelegation();
  on('click', '[data-action="create-user"]', () => setPanelOpen(true));
  on('click', '#create-user-overlay', () => setPanelOpen(false));
  on('click', '#create-user-panel .sp-panel-close', () => setPanelOpen(false));
  on('click', '#create-user-panel [data-action="cancel"]', () => setPanelOpen(false));
  on('click', '#create-user-panel [data-action="save"]', createUser);
  bindBulk();
};

init();
