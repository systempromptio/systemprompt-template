import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';
import { on, initDelegation } from '../services/events.js';

const projectId = () => document.querySelector('[data-project-id]')?.dataset.projectId ?? '';

const projectPath = (suffix) => '/projects/' + encodeURIComponent(projectId()) + suffix;

const dialog = () => document.querySelector('dialog[data-dialog="add-member"]');

const addMember = async () => {
  const userId = document.querySelector('[data-field="add-member-user"]')?.value;
  if (!userId) {
    showToast('Choose an account to add', 'error');
    return;
  }
  try {
    await apiFetch(projectPath('/members'), {
      method: 'POST',
      body: JSON.stringify({ user_id: userId }),
    });
    showToast('Member added', 'success');
    window.location.reload();
  } catch (err) {
    showToast(err.message || 'Failed to add member', 'error');
  }
};

const removeMember = (userId) => {
  showConfirmDialog(
    'Remove member?',
    'Removes the manual membership. A directory-sourced membership is written back at the next sign-in.',
    'Remove',
    async () => {
      try {
        await apiFetch(projectPath('/members/' + encodeURIComponent(userId)), { method: 'DELETE' });
        showToast('Member removed', 'success');
        window.location.reload();
      } catch (err) {
        showToast(err.message || 'Failed to remove member', 'error');
      }
    },
  );
};

const addMapping = async () => {
  const adGroup = document.querySelector('[data-field="new-ad-group"]')?.value.trim();
  if (!adGroup) {
    showToast('Enter a directory group name', 'error');
    return;
  }
  try {
    await apiFetch(projectPath('/ad-mappings'), {
      method: 'POST',
      body: JSON.stringify({ ad_group: adGroup }),
    });
    showToast('Mapping added', 'success');
    window.location.reload();
  } catch (err) {
    showToast(err.message || 'Failed to add mapping', 'error');
  }
};

const removeMapping = (adGroup) => {
  showConfirmDialog(
    'Remove mapping?',
    'People attributed by this directory group lose the membership at their next sign-in.',
    'Remove',
    async () => {
      try {
        await apiFetch(projectPath('/ad-mappings/' + encodeURIComponent(adGroup)), { method: 'DELETE' });
        showToast('Mapping removed', 'success');
        window.location.reload();
      } catch (err) {
        showToast(err.message || 'Failed to remove mapping', 'error');
      }
    },
  );
};

const saveSettings = async () => {
  const name = document.querySelector('[data-field="project-name"]')?.value.trim();
  if (!name) {
    showToast('A project needs a name', 'error');
    return;
  }
  const description = document.querySelector('[data-field="project-description"]')?.value.trim();
  try {
    await apiFetch(projectPath(''), {
      method: 'PUT',
      body: JSON.stringify({ name, description: description || null }),
    });
    showToast('Project saved', 'success');
    window.location.reload();
  } catch (err) {
    showToast(err.message || 'Failed to save the project', 'error');
  }
};

const deleteProject = () => {
  showConfirmDialog(
    'Delete project?',
    'Every membership on it is removed. Requests already attributed to it keep their history.',
    'Delete',
    async () => {
      try {
        await apiFetch(projectPath(''), { method: 'DELETE' });
        showToast('Project deleted', 'success');
        window.location.assign('/admin/projects');
      } catch (err) {
        showToast(err.message || 'Failed to delete the project', 'error');
      }
    },
  );
};

export const init = () => {
  initDelegation();
  on('click', '[data-action="save-project"]', () => saveSettings());
  on('click', '[data-action="delete-project"]', () => deleteProject());
  on('click', '[data-action="add-member"]', () => dialog()?.showModal());
  on('click', '[data-action="close-add-member"]', () => dialog()?.close());
  on('click', '[data-action="confirm-add-member"]', () => {
    dialog()?.close();
    addMember();
  });
  on('click', '[data-action="remove-member"]', (e, el) => removeMember(el.dataset.userId));
  on('click', '[data-action="add-mapping"]', () => addMapping());
  on('click', '[data-action="remove-mapping"]', (e, el) => removeMapping(el.dataset.adGroup));
};

init();
