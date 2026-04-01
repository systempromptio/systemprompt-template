import { showToast } from '../services/toast.js';
import { showConfirmDialog, showPromptDialog } from '../services/confirm.js';
import { initExpandRows, initFilters, initTimeAgo } from './admin-org-skills-events.js';
import { openEditPanel, openAssignPanel, getSkillDetail } from './admin-org-skills-panel.js';
import { on } from '../services/events.js';
import { rawFetch } from '../services/api.js';

const reloadAfterSuccess = (msg) => {
  showToast(msg, 'success');
  setTimeout(() => window.location.reload(), 500);
};

const handleDeleteSkill = (skillId) => {
  showConfirmDialog(
    'Delete Skill?',
    'Are you sure you want to delete skill "' + skillId + '"? This cannot be undone.',
    'Delete',
    async () => {
      try {
        await rawFetch('/api/admin/skills/' + encodeURIComponent(skillId), { method: 'DELETE' });
        reloadAfterSuccess('Skill deleted');
      } catch { showToast('Failed to delete skill', 'error'); }
    }
  );
};

const forkSkillPayload = (newId, data, skillId) => ({
  skill_id: newId,
  name: (data.name || skillId) + ' (Custom)',
  description: data.description || '',
  base_skill_id: skillId
});

const handleForkSkill = (skillId) => {
  const data = getSkillDetail(skillId);
  if (data) {
    showPromptDialog('Customize Skill', 'Enter a new ID for the customized skill:', skillId + '-custom', async (newId) => {
      try {
        await rawFetch('/api/admin/skills', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(forkSkillPayload(newId, data, skillId))
        });
        reloadAfterSuccess('Skill customized');
      } catch { showToast('Failed to customize skill', 'error'); }
    });
  }
};

export const initOrgSkills = () => {
  initExpandRows(getSkillDetail);
  initFilters();
  initTimeAgo();
  on('click', '[data-delete-skill]', (e, btn) => {
    handleDeleteSkill(btn.getAttribute('data-delete-skill'));
  });
  on('click', '[data-fork-skill]', (e, btn) => {
    handleForkSkill(btn.getAttribute('data-fork-skill'));
  });
  on('click', '[data-edit-skill]', (e, btn) => {
    const sid = btn.getAttribute('data-edit-skill');
    const data = getSkillDetail(sid);
    if (data) openEditPanel(sid, data);
  });
  on('click', '[data-assign-skill]', (e, btn) => {
    openAssignPanel(
      btn.getAttribute('data-assign-skill'),
      btn.getAttribute('data-skill-name') || btn.getAttribute('data-assign-skill')
    );
  });
};
