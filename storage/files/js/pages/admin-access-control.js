// The access-control page. The entities table and the audience grid are
// server-rendered and filtered by form submission; this file remembers whether
// the "how a decision is made" strip is collapsed and creates a group.

import { apiFetch } from '../services/api.js';
import { initSyncPlane } from '../components/sp-sync-plane.js';

const EXPLAIN_KEY = 'sp-ac-explain-collapsed';

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
  'new-group': () => openDialog('new-group'),
  'close-dialog': closeDialogs,
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

// Why: the strip is open on first visit so the model is read once, and stays
// closed afterwards for the person who has. Per browser, never per account.
const rememberExplain = () => {
  const strip = field('explain');
  if (!strip) return;
  try {
    if (window.localStorage.getItem(EXPLAIN_KEY) === '1') strip.open = false;
  } catch {
    // storage unavailable: leave it open
  }
  strip.addEventListener('toggle', () => {
    try {
      window.localStorage.setItem(EXPLAIN_KEY, strip.open ? '0' : '1');
    } catch {
      // storage unavailable: nothing to remember
    }
  });
};

const init = () => {
  const header = document.querySelector('.sp-page-header');
  if (header) bindRoot(header);
  for (const el of document.querySelectorAll('dialog[data-dialog]')) bindRoot(el);
  rememberExplain();
};

init();

// Why: the Sync tab renders the shared sync-plane component; its buttons
// are bound here so the tab works without a second page script.
initSyncPlane();
