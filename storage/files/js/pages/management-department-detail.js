// The department detail page's write actions: adding and removing members,
// renaming, and deleting the department.

import { rawFetch } from '../services/api.js';
import { showConfirmDialog } from '../services/confirm.js';
import { showToast } from '../services/toast.js';
import { on, initDelegation } from '../services/events.js';

const DEPARTMENTS_URL = '/admin/management/departments';
const USERS_URL = '/admin/management/users';

const pageEl = document.querySelector('[data-dept-id]');
const deptId = pageEl?.dataset.deptId;
const deptName = pageEl?.dataset.deptName;

const setUserDepartment = (userId, departmentName) =>
  rawFetch(`${USERS_URL}/${encodeURIComponent(userId)}/department`, {
    method: 'PUT',
    body: JSON.stringify({ department_name: departmentName }),
  });

const showInlineError = (form, err) => {
  const el = form.querySelector('[data-error]');
  if (!el) return;
  el.textContent = err?.message ?? 'The request was refused';
  el.hidden = false;
};

const openAddDialog = () => {
  const dlg = document.querySelector('[data-dialog="member-add"]');
  const form = dlg?.querySelector('form');
  if (!dlg || !form) return;
  form.reset();
  const err = form.querySelector('[data-error]');
  if (err) err.hidden = true;
  dlg.showModal();
};

const addMember = async (form) => {
  const userId = String(new FormData(form).get('user_id') ?? '').trim();
  if (!userId) return;
  try {
    await setUserDepartment(userId, deptName);
    window.location.reload();
  } catch (err) {
    showInlineError(form, err);
  }
};

const unassign = (button) => {
  showConfirmDialog('Move member?', 'Move this account back to Default?', 'Move', async () => {
    try {
      await setUserDepartment(button.dataset.unassignUser, 'Default');
      window.location.reload();
    } catch (err) {
      showToast(err?.message ?? 'Failed to move the member', 'error');
    }
  });
};

const saveSettings = async (form) => {
  const fd = new FormData(form);
  const status = form.querySelector('[data-status]');
  if (status) status.textContent = 'Saving…';
  try {
    await rawFetch(`${DEPARTMENTS_URL}/${encodeURIComponent(form.dataset.deptId)}`, {
      method: 'PUT',
      body: JSON.stringify({ name: fd.get('name'), description: fd.get('description') || '' }),
    });
    if (status) status.textContent = 'Saved';
    window.setTimeout(() => window.location.reload(), 600);
  } catch (err) {
    if (status) status.textContent = '';
    showToast(err?.message ?? 'Failed to save the department', 'error');
  }
};

const deleteDepartment = () => {
  showConfirmDialog(
    `Delete "${deptName}"?`,
    'Members are moved to Default and the department-level access rules are removed.',
    'Delete',
    async () => {
      try {
        await rawFetch(`${DEPARTMENTS_URL}/${encodeURIComponent(deptId)}`, { method: 'DELETE' });
        window.location.assign('/admin/departments');
      } catch (err) {
        showToast(err?.message ?? 'Failed to delete the department', 'error');
      }
    },
  );
};

const init = () => {
  initDelegation();
  on('click', '[data-action="add-member"]', openAddDialog);
  on('click', '[data-action="dialog-close"]', (_ev, button) => button.closest('dialog')?.close());
  on('click', '[data-unassign-user]', (_ev, button) => unassign(button));
  on('click', '[data-action="delete-department"]', deleteDepartment);
  document.querySelector('form[data-form="member-add"]')?.addEventListener('submit', (ev) => {
    ev.preventDefault();
    addMember(ev.target);
  });
  document.querySelector('form[data-form="dept-settings"]')?.addEventListener('submit', (ev) => {
    ev.preventDefault();
    saveSettings(ev.target);
  });
};

init();
