import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';
import { initExpandRows, initTableSearch, initTableFilter } from '../services/entity-common.js';
import { initEntityBatch } from '../services/entity-batch.js';
import { on } from '../services/events.js';

const deleteSkill = async (skillId) => {
  try {
    await apiFetch('/user/skills/' + encodeURIComponent(skillId), { method: 'DELETE' });
    showToast('Skill deleted', 'success');
    setTimeout(() => window.location.reload(), 500);
  } catch (err) {
    showToast(err.message || 'Failed to delete skill', 'error');
  }
};

export const initMySkillsPage = () => {
  if (document.getElementById('my-skills-table')) {
    initExpandRows('#my-skills-table');
    initTableSearch('my-skills-search', '#my-skills-table');
    initTableFilter('my-skills-tag-filter', '#my-skills-table', 'data-tags');
    initEntityBatch({
      tableSelector: '#my-skills-table',
      rowSelector: 'tbody tr.clickable-row',
      idExtractor: (row) => row.getAttribute('data-entity-id'),
      deleteEndpoint: '/user/skills/batch-delete',
      entityLabel: 'skill',
    });
    on('click', '[data-delete-entity][data-entity-type="user-skill"]', (e, btn) => {
      e.stopPropagation();
      const skillId = btn.getAttribute('data-delete-entity');
      showConfirmDialog('Delete Skill', 'This action cannot be undone.', 'Delete', () => deleteSkill(skillId));
    });
  }
};

initMySkillsPage();
