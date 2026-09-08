import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { on, initDelegation } from '../services/events.js';

const rowFor = (userId) => document.querySelector('tr[data-user-id="' + CSS.escape(userId) + '"]');

const selectedGroup = (userId) =>
  document.querySelector('select[data-action="assign-group"][data-user-id="' + CSS.escape(userId) + '"]')?.value;

const assign = async (userId) => {
  const target = selectedGroup(userId);
  if (!target) {
    showToast('Choose a destination group', 'error');
    return;
  }
  try {
    await apiFetch('/groups/' + encodeURIComponent(target) + '/members', {
      method: 'POST',
      body: JSON.stringify({ user_id: userId }),
    });
    rowFor(userId)?.remove();
    showToast('Assigned to ' + target, 'success');
  } catch (err) {
    showToast(err.message || 'Failed to assign', 'error');
  }
};

export const init = () => {
  initDelegation();
  on('click', '[data-action="assign-submit"]', (e, el) => assign(el.dataset.userId));
};

init();
