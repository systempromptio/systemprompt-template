import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';
import { initExpandRows, initTableSearch } from '../services/entity-common.js';
import { initEntityBatch } from '../services/entity-batch.js';
import { on } from '../services/events.js';

const deleteAgent = async (agentId) => {
  try {
    await apiFetch('/user/agents/' + encodeURIComponent(agentId), { method: 'DELETE' });
    showToast('Agent deleted', 'success');
    setTimeout(() => window.location.reload(), 500);
  } catch (err) {
    showToast(err.message || 'Failed to delete agent', 'error');
  }
};

export const initMyAgentsPage = () => {
  if (document.getElementById('my-agents-table')) {
    initExpandRows('#my-agents-table');
    initTableSearch('my-agents-search', '#my-agents-table');
    initEntityBatch({
      tableSelector: '#my-agents-table',
      rowSelector: 'tbody tr.clickable-row',
      idExtractor: (row) => row.getAttribute('data-entity-id'),
      deleteEndpoint: '/user/agents/batch-delete',
      entityLabel: 'agent',
    });
    on('click', '[data-delete-entity][data-entity-type="user-agent"]', (e, btn) => {
      e.stopPropagation();
      const agentId = btn.getAttribute('data-delete-entity');
      showConfirmDialog('Delete Agent', 'This action cannot be undone.', 'Delete', () => deleteAgent(agentId));
    });
  }
};

initMyAgentsPage();
