// The group's Marketplaces tab.
// The endpoint reconciles rather than diffs: it takes the whole set the group
// should reach and deletes this group's rules on everything else. So the save
// sends every ticked box, not the ones that changed, and an unticked row is
// how a rule is removed.
import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { on, initDelegation } from '../services/events.js';

const groupId = () => document.querySelector('[data-group-id]')?.dataset.groupId ?? '';

const ticked = () =>
  Array.from(document.querySelectorAll('[data-field="marketplace"]:checked')).map((el) => el.value);

const save = async () => {
  const id = groupId();
  if (!id) return;
  try {
    await apiFetch(`/groups/${encodeURIComponent(id)}/marketplaces`, {
      method: 'PUT',
      body: JSON.stringify({ marketplace_ids: ticked() }),
    });
    showToast('Entitlement saved', 'success');
    window.location.reload();
  } catch (err) {
    showToast(err.message || 'Failed to save the entitlement', 'error');
  }
};

export const init = () => {
  initDelegation();
  on('click', '[data-action="save-marketplaces"]', () => save());
};

init();
