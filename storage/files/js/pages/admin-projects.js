// The projects listing: the one mutation the page offers is creating a project.
// Sorting, filtering and paging are links and a GET form, so they need no
// script at all — this file exists only for the create dialog.
import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { on, initDelegation } from '../services/events.js';

const dialog = () => document.querySelector('dialog[data-dialog="new-project"]');

const field = (name) => document.querySelector(`[data-field="${name}"]`)?.value.trim() ?? '';

const createProject = async () => {
  const id = field('new-project-id');
  const name = field('new-project-name');
  if (!id || !name) {
    showToast('A project needs an identifier and a name', 'error');
    return;
  }
  try {
    await apiFetch('/projects', {
      method: 'POST',
      body: JSON.stringify({ id, name, description: field('new-project-description') || null }),
    });
    showToast('Project created', 'success');
    window.location.assign(`/admin/projects/${encodeURIComponent(id)}`);
  } catch (err) {
    showToast(err.message || 'Failed to create the project', 'error');
  }
};

export const init = () => {
  initDelegation();
  on('click', '[data-action="new-project"]', () => dialog()?.showModal());
  on('click', '[data-action="close-new-project"]', () => dialog()?.close());
  on('click', '[data-action="confirm-new-project"]', () => {
    dialog()?.close();
    createProject();
  });
};

init();
