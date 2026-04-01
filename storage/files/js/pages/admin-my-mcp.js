import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';
import { initExpandRows } from '../services/entity-common.js';
import { initEntityBatch } from '../services/entity-batch.js';
import { on } from '../services/events.js';

const initSearch = () => {
  const searchInput = document.getElementById('my-mcp-search');
  const table = document.getElementById('my-mcp-table');
  if (searchInput && table) {
    let timer;
    searchInput.addEventListener('input', () => {
      clearTimeout(timer);
      timer = setTimeout(() => {
        const query = searchInput.value.toLowerCase().trim();
        for (const row of table.querySelectorAll('tr.clickable-row')) {
          const name = row.getAttribute('data-name') || '';
          const matches = !query || name.toLowerCase().includes(query);
          row.style.display = matches ? '' : 'none';
          const detail = row.nextElementSibling;
          if (detail?.classList.contains('detail-row')) detail.style.display = matches ? '' : 'none';
        }
      }, 200);
    });
  }
};

const initMcpPanel = () => {
  const panel = document.getElementById('mcp-panel');
  const overlay = document.getElementById('mcp-panel-overlay');
  const panelTitle = document.getElementById('mcp-panel-title');
  const form = document.getElementById('mcp-form');
  const serverIdInput = document.getElementById('mcp-server-id');
  const nameInput = document.getElementById('mcp-name');
  const descriptionInput = document.getElementById('mcp-description');
  const endpointInput = document.getElementById('mcp-endpoint');
  let isEditMode = false;

  const openPanel = (title, edit) => {
    isEditMode = !!edit;
    if (panelTitle) panelTitle.textContent = title;
    if (panel) panel.classList.add('open');
    if (overlay) overlay.classList.add('open');
  };

  const closePanel = () => {
    if (panel) panel.classList.remove('open');
    if (overlay) overlay.classList.remove('open');
    if (form) form.reset();
    if (serverIdInput) serverIdInput.readOnly = false;
    isEditMode = false;
  };

  return { openPanel, closePanel, form, serverIdInput, nameInput, descriptionInput, endpointInput, overlay, getEditMode: () => isEditMode };
};

const initEditMcpButtons = (ctx) => {
  on('click', '.edit-mcp-btn', (e, btn) => {
    e.stopPropagation();
    const serverId = btn.getAttribute('data-server-id');
    const row = document.querySelector('tr.clickable-row[data-server-id="' + serverId + '"]');
    if (row) {
      ctx.serverIdInput.value = serverId;
      ctx.serverIdInput.readOnly = true;
      ctx.nameInput.value = row.getAttribute('data-name-val') || '';
      ctx.descriptionInput.value = row.getAttribute('data-description') || '';
      ctx.endpointInput.value = row.getAttribute('data-endpoint') || '';
      ctx.openPanel('Edit MCP Server', true);
    }
  });
};

const initDeleteMcpButtons = () => {
  on('click', '.delete-mcp-btn', (e, btn) => {
    e.stopPropagation();
    const serverId = btn.getAttribute('data-server-id');
    showConfirmDialog('Delete MCP Server', 'Delete "' + serverId + '"?', 'Delete', async () => {
      try {
        await apiFetch('/user/mcp-servers/' + encodeURIComponent(serverId), { method: 'DELETE' });
        showToast('MCP server deleted', 'success'); setTimeout(() => location.reload(), 500);
      } catch (err) { showToast(err.message || 'Failed to delete', 'error'); }
    });
  });
};

const initMcpFormSubmit = (ctx) => {
  ctx.form?.addEventListener('submit', async (e) => {
    e.preventDefault();
    const serverId = ctx.serverIdInput.value.trim();
    const name = ctx.nameInput.value.trim();
    const description = ctx.descriptionInput.value.trim();
    const endpoint = ctx.endpointInput.value.trim();
    if (!serverId || !name) {
      showToast('Server ID and name are required', 'error');
    } else {
      const isEditMode = ctx.getEditMode();
      const url = isEditMode ? '/user/mcp-servers/' + encodeURIComponent(serverId) : '/user/mcp-servers';
      const method = isEditMode ? 'PUT' : 'POST';
      const body = isEditMode ? { name, description, endpoint } : { mcp_server_id: serverId, name, description, endpoint };
      try {
        await apiFetch(url, { method, body: JSON.stringify(body) });
        showToast(isEditMode ? 'MCP server updated' : 'MCP server created', 'success'); ctx.closePanel(); setTimeout(() => location.reload(), 500);
      } catch (err) { showToast(err.message || 'Failed to save MCP server', 'error'); }
    }
  });
};

export const initMyMcpPage = () => {
  if (document.getElementById('my-mcp-table')) {
    initExpandRows('#my-mcp-table');
    initEntityBatch({
      tableSelector: '#my-mcp-table',
      rowSelector: 'tbody tr.clickable-row',
      idExtractor: (row) => row.getAttribute('data-entity-id'),
      deleteEndpoint: '/user/mcp-servers/batch-delete',
      entityLabel: 'MCP server',
    });
    const ctx = initMcpPanel();
    document.getElementById('add-mcp-btn')?.addEventListener('click', () => ctx.openPanel('Add MCP Server', false));
    document.getElementById('mcp-panel-close')?.addEventListener('click', ctx.closePanel);
    document.getElementById('mcp-cancel-btn')?.addEventListener('click', ctx.closePanel);
    ctx.overlay?.addEventListener('click', ctx.closePanel);
    initEditMcpButtons(ctx);
    initDeleteMcpButtons();
    initMcpFormSubmit(ctx);
    initSearch();
  }
};

initMyMcpPage();
