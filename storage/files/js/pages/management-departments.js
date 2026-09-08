// The departments listing's write actions: creating and deleting a
// department. Search is a link the server answers.

import { rawFetch } from '../services/api.js';
import { showConfirmDialog } from '../services/confirm.js';
import { showToast } from '../services/toast.js';
import { on, initDelegation } from '../services/events.js';

const DEPARTMENTS_URL = '/admin/management/departments';

const showInlineError = (form, err) => {
  const el = form.querySelector('[data-error]');
  if (!el) return;
  el.textContent = err?.message ?? 'The request was refused';
  el.hidden = false;
};

const openCreateDialog = () => {
  const dlg = document.querySelector('[data-dialog="dept-create"]');
  const form = dlg?.querySelector('form');
  if (!dlg || !form) return;
  form.reset();
  const err = form.querySelector('[data-error]');
  if (err) err.hidden = true;
  dlg.showModal();
};

const createDepartment = async (form) => {
  const fd = new FormData(form);
  try {
    await rawFetch(DEPARTMENTS_URL, {
      method: 'POST',
      body: JSON.stringify({ name: fd.get('name'), description: fd.get('description') || '' }),
    });
    window.location.reload();
  } catch (err) {
    showInlineError(form, err);
  }
};

const deleteDepartment = (button) => {
  const { deleteDept: id, deptName: name } = button.dataset;
  showConfirmDialog(
    `Delete department "${name}"?`,
    'Members are moved to Default and the department-level access rules are removed.',
    'Delete',
    async () => {
      try {
        await rawFetch(`${DEPARTMENTS_URL}/${encodeURIComponent(id)}`, { method: 'DELETE' });
        window.location.reload();
      } catch (err) {
        showToast(err?.message ?? 'Failed to delete the department', 'error');
      }
    },
  );
};

const init = () => {
  initDelegation();
  on('click', '[data-action="new-department"]', openCreateDialog);
  on('click', '[data-action="dialog-close"]', (_ev, button) => button.closest('dialog')?.close());
  on('click', '[data-delete-dept]', (_ev, button) => deleteDepartment(button));
  document.querySelector('form[data-form="dept-create"]')?.addEventListener('submit', (ev) => {
    ev.preventDefault();
    createDepartment(ev.target);
  });
};

init();
