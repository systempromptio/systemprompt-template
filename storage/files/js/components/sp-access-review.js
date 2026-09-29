// The access-control review's actions, shared by an entity's "Who gets
// this" panel and the review list on Code sync: apply code to one entity,
// or keep the database's version. Both ask for a reason first, through the
// one reason dialog, and both reload so the server re-derives the diff.

import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { on, initDelegation } from '../services/events.js';

const APPLY_PATH = '/sync/planes/access_control/apply';
const KEEP_PATH = '/sync/access-control/keep';

const dialog = () => document.querySelector('dialog[data-dialog="access-reason"]');
const part = (name) => dialog()?.querySelector(`[data-access-reason="${name}"]`);

let pending = null;

// Why: the one confirmation every access write goes through, so no write
// leaves the page without a stated reason.
export const askReason = (title, body, run) => {
  const box = dialog();
  if (!box) return;
  pending = run;
  part('title').textContent = title;
  part('body').textContent = body;
  part('input').value = '';
  box.showModal();
  part('input').focus();
};

const closeReason = () => {
  pending = null;
  dialog()?.close();
};

const submitReason = async (event) => {
  event.preventDefault();
  const reason = part('input')?.value.trim() ?? '';
  if (!reason) {
    showToast('A reason is required', 'error');
    part('input')?.focus();
    return;
  }
  const run = pending;
  closeReason();
  if (!run) return;
  try {
    await run(reason);
    window.location.reload();
  } catch (err) {
    showToast(err?.message ?? 'The change failed', 'error');
  }
};

const applyCode = (button) => {
  const { entity } = button.closest('[data-access-review]')?.dataset ?? {};
  if (!entity) return;
  askReason(
    `Apply code to ${entity}`,
    'Make this entity match rules.yaml: add what code declares, correct what differs, and delete what code no longer names on it. Per-person overrides are kept.',
    (reason) => apiFetch(APPLY_PATH, {
      method: 'POST',
      body: JSON.stringify({ mode: 'overwrite', entities: [entity], reason }),
    }),
  );
};

const keepDatabase = (button) => {
  const { entity, fingerprint } = button.closest('[data-access-review]')?.dataset ?? {};
  if (!entity) return;
  askReason(
    `Keep the database version of ${entity}`,
    'Nothing is written. The difference leaves the to-do list until code or this database changes again; export to code to make it permanent.',
    (reason) => apiFetch(KEEP_PATH, {
      method: 'POST',
      body: JSON.stringify({ entity, fingerprint, reason }),
    }),
  );
};

let bound = false;

export const initAccessReview = () => {
  if (bound) return;
  bound = true;
  initDelegation();
  on('click', '[data-action="access-apply-code"]', (e, el) => applyCode(el));
  on('click', '[data-action="access-keep-db"]', (e, el) => keepDatabase(el));
  on('click', '[data-action="access-reason-close"]', closeReason);
  dialog()?.querySelector('[data-access-reason-form]')?.addEventListener('submit', submitReason);
};
