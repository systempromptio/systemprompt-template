import { showToast } from '../services/toast.js';
import { showConfirmDialog, showPromptDialog } from '../services/confirm.js';
import { initExpandRows, initFilters, initTimeAgo } from './admin-org-agents-events.js';
import { openEditPanel, openAssignPanel, getAgentDetail } from './admin-org-agents-panel.js';
import { on } from '../services/events.js';
import { rawFetch } from '../services/api.js';

const reloadAfterSuccess = (msg) => {
  showToast(msg, 'success');
  setTimeout(() => window.location.reload(), 500);
};

const handleDeleteAgent = (agentId) => {
  showConfirmDialog(
    'Delete Agent?',
    'Are you sure you want to delete agent "' + agentId + '"? This cannot be undone.',
    'Delete',
    async () => {
      try {
        await rawFetch('/api/admin/agents/' + encodeURIComponent(agentId), { method: 'DELETE' });
        reloadAfterSuccess('Agent deleted');
      } catch { showToast('Failed to delete agent', 'error'); }
    }
  );
};

const forkAgentPayload = (newId, data, agentId) => ({
  id: newId,
  name: (data.name || agentId) + ' (Custom)',
  description: data.description || '',
  system_prompt: data.system_prompt || '',
  enabled: true
});

const handleForkAgent = (agentId) => {
  const data = getAgentDetail(agentId);
  if (data) {
    showPromptDialog('Customize Agent', 'Enter a new ID for the customized agent:', agentId + '-custom', async (newId) => {
      try {
        await rawFetch('/api/admin/agents', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(forkAgentPayload(newId, data, agentId))
        });
        reloadAfterSuccess('Agent customized');
      } catch { showToast('Failed to customize agent', 'error'); }
    });
  }
};

export const initOrgAgents = () => {
  initExpandRows(getAgentDetail);
  initFilters();
  initTimeAgo();
  on('click', '[data-delete-agent]', (e, btn) => {
    handleDeleteAgent(btn.getAttribute('data-delete-agent'));
  });
  on('click', '[data-fork-agent]', (e, btn) => {
    handleForkAgent(btn.getAttribute('data-fork-agent'));
  });
  on('click', '[data-edit-agent]', (e, btn) => {
    const aid = btn.getAttribute('data-edit-agent');
    const data = getAgentDetail(aid);
    if (data) openEditPanel(aid, data);
  });
  on('click', '[data-assign-agent]', (e, btn) => {
    openAssignPanel(
      btn.getAttribute('data-assign-agent'),
      btn.getAttribute('data-agent-name') || btn.getAttribute('data-assign-agent')
    );
  });
};
