import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';
import { on, initDelegation } from '../services/events.js';

const WORDING = {
  approve: {
    title: 'Approve this call?',
    body: 'The parked call runs with exactly the arguments shown. A retry with different arguments is held again.',
    label: 'Approve',
  },
  deny: {
    title: 'Deny this call?',
    body: 'The caller is released with a refusal. The decision is recorded against your account.',
    label: 'Deny',
  },
};

const decide = async (verb, callId) => {
  try {
    await apiFetch('/approvals/' + encodeURIComponent(callId) + '/' + verb, { method: 'POST' });
    showToast('Request ' + verb + 'd', 'success');
    globalThis.location.reload();
  } catch (err) {
    showToast(err.message || 'Could not record the decision', 'error');
  }
};

const ask = (verb, el) => {
  const wording = WORDING[verb];
  const callId = el.dataset.callId;
  showConfirmDialog(
    wording.title,
    wording.body + ' Tool: ' + (el.dataset.tool || callId) + '.',
    wording.label,
    () => decide(verb, callId),
    { btnClass: verb === 'approve' ? 'sp-btn--primary' : '' },
  );
};

export const init = () => {
  initDelegation();
  on('click', '[data-action="approve"]', (event, el) => ask('approve', el));
  on('click', '[data-action="deny"]', (event, el) => ask('deny', el));
};

init();
