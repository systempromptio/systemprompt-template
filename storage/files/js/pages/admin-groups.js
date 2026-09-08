// The groups listing's three header actions: create a group, map a directory
// group into one, and rebuild the attribution keys.
// Sorting, paging and the time window are links the server answers, so
// nothing here re-renders the table. These are the mutations only.
import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';
import { on, initDelegation } from '../services/events.js';

const dialog = (name) => document.querySelector(`dialog[data-dialog="${name}"]`);

const field = (name) => document.querySelector(`[data-field="${name}"]`)?.value.trim() ?? '';

const createGroup = async () => {
  const id = field('new-group-id');
  const name = field('new-group-name');
  if (!id || !name) {
    showToast('A group needs an identifier and a name', 'error');
    return;
  }
  try {
    await apiFetch('/groups', {
      method: 'POST',
      body: JSON.stringify({ id, name, description: field('new-group-description') || null }),
    });
    showToast('Group created', 'success');
    window.location.assign(`/admin/groups/${encodeURIComponent(id)}`);
  } catch (err) {
    showToast(err.message || 'Failed to create the group', 'error');
  }
};

const mapDirectoryGroup = async () => {
  const adGroup = field('map-ad-group');
  const groupId = field('map-target-group');
  if (!adGroup || !groupId) {
    showToast('Name a directory group and the group it places people into', 'error');
    return;
  }
  try {
    await apiFetch(`/groups/${encodeURIComponent(groupId)}/ad-mappings`, {
      method: 'POST',
      body: JSON.stringify({ ad_group: adGroup }),
    });
    showToast('Mapping added', 'success');
    window.location.reload();
  } catch (err) {
    showToast(err.message || 'Failed to add the mapping', 'error');
  }
};

// Rewrites only the keys it decided itself, so a manual override survives.
// It is behind a confirm because it moves spend between groups on every page
// that reads attribution, which is not obvious from the button.
const recomputeDefaults = () => {
  showConfirmDialog(
    'Recompute attribution keys?',
    'Rebuilds every automatic primary group and project from current membership. Manual overrides are left alone.',
    'Recompute',
    async () => {
      try {
        const result = await apiFetch('/scope-defaults/recompute', { method: 'POST' });
        showToast(`Recomputed ${result?.recomputed ?? 0} attribution keys`, 'success');
        window.location.reload();
      } catch (err) {
        showToast(err.message || 'Failed to recompute', 'error');
      }
    },
  );
};

export const init = () => {
  initDelegation();
  on('click', '[data-action="create-group"]', () => dialog('create-group')?.showModal());
  on('click', '[data-action="close-create-group"]', () => dialog('create-group')?.close());
  on('click', '[data-action="confirm-create-group"]', () => {
    dialog('create-group')?.close();
    createGroup();
  });
  on('click', '[data-action="map-directory"]', () => dialog('map-directory')?.showModal());
  on('click', '[data-action="close-map-directory"]', () => dialog('map-directory')?.close());
  on('click', '[data-action="confirm-map-directory"]', () => {
    dialog('map-directory')?.close();
    mapDirectoryGroup();
  });
  on('click', '[data-action="recompute-defaults"]', () => recomputeDefaults());
};

init();
