import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';
import { on, initDelegation } from '../services/events.js';

const marketplaceId = () =>
  document.querySelector('[data-marketplace-id]')?.dataset.marketplaceId || '';

// The group endpoint reconciles a whole set rather than accepting a delta, so
// a toggle reads the group's current marketplaces and sends the amended list.
const setMembership = async (groupId, groupName, include) => {
  const marketplace = marketplaceId();
  if (!marketplace) return;
  const path = '/groups/' + encodeURIComponent(groupId) + '/marketplaces';
  try {
    const current = await apiFetch(path);
    const ids = new Set(current?.marketplace_ids || []);
    if (include) {
      ids.add(marketplace);
    } else {
      ids.delete(marketplace);
    }
    await apiFetch(path, {
      method: 'PUT',
      body: JSON.stringify({ marketplace_ids: Array.from(ids) }),
    });
    showToast(
      include ? 'Assigned to ' + groupName : 'Unassigned from ' + groupName,
      'success',
    );
    window.location.reload();
  } catch (err) {
    showToast(err.message || 'Failed to change the entitlement', 'error');
  }
};

export const init = () => {
  initDelegation();
  on('click', '[data-action="marketplace-assign"]', (e, el) =>
    setMembership(el.dataset.groupId, el.dataset.groupName, true),
  );
  on('click', '[data-action="marketplace-unassign"]', (e, el) =>
    showConfirmDialog(
      'Unassign ' + el.dataset.groupName,
      'Members of this group lose access to everything this marketplace ships.',
      'Unassign',
      () => setMembership(el.dataset.groupId, el.dataset.groupName, false),
    ),
  );
};

init();
