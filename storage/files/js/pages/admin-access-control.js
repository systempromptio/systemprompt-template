// The access-control page's two dialogs. The ledger itself is server-rendered
// and filtered by form submission, so this file only opens the YAML snapshot
// and creates a group.

import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';

const dialog = (name) => document.querySelector(`dialog[data-dialog="${name}"]`);

const openDialog = (name) => {
  const el = dialog(name);
  if (el) el.showModal();
};

const closeDialogs = () => {
  for (const el of document.querySelectorAll('dialog[data-dialog]')) el.close();
};

const field = (name) => document.querySelector(`[data-ac="${name}"]`);

const showGroupError = (message) => {
  const err = field('new-group-error');
  if (!err) return;
  err.textContent = message;
  err.hidden = !message;
};

const showYaml = async () => {
  openDialog('yaml');
  const target = field('yaml-content');
  target.textContent = 'Loading…';
  try {
    const yaml = await apiFetch('/access-control/yaml-snapshot');
    target.textContent = yaml || 'No band rules are stored in the database yet.';
  } catch {
    target.textContent = 'Failed to load the YAML snapshot.';
  }
};

const copyYaml = async (button) => {
  try {
    await navigator.clipboard.writeText(field('yaml-content').textContent);
    button.textContent = 'Copied';
  } catch {
    showToast('Copy failed — select the text manually', 'error');
  }
};

// Why: a duplicate identifier is the server's call. It answers with a
// conflict whose message names the clash, and that message is shown as-is.
const saveGroup = async () => {
  const id = field('new-group-id').value.trim();
  const name = field('new-group-name').value.trim();
  const description = field('new-group-desc').value.trim();
  showGroupError('');
  if (!id || !name) {
    showGroupError('An identifier and a name are both required.');
    return;
  }
  try {
    await apiFetch('/groups', {
      method: 'POST',
      body: JSON.stringify({ id, name, description }),
    });
    window.location.reload();
  } catch (err) {
    showGroupError(err?.message ?? 'Failed to create the group');
  }
};

const ACTIONS = {
  'show-yaml': showYaml,
  'new-group': () => openDialog('new-group'),
  'close-dialog': closeDialogs,
  'copy-yaml': (target) => copyYaml(target),
  'save-group': saveGroup,
};

const bindRoot = (root) => {
  root.addEventListener('click', (ev) => {
    const target = ev.target.closest('[data-action]');
    const handler = target && ACTIONS[target.dataset.action];
    if (!handler) return;
    handler(target);
  });
};

const init = () => {
  const header = document.querySelector('.sp-page-header');
  if (header) bindRoot(header);
  for (const el of document.querySelectorAll('dialog[data-dialog]')) bindRoot(el);
};

init();
