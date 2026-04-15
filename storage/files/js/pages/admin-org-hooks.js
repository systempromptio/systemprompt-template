import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';
import { escapeHtml } from '../utils/dom.js';
import { on } from '../services/events.js';

const initExpandRows = () => {
  const table = document.querySelector('.data-table');
  if (table) {
    table.addEventListener('click', (e) => {
      if (!e.target.closest('[data-no-row-click],.actions-menu,.btn,a,input,.toggle-switch')) {
        const row = e.target.closest('tr.clickable-row');
        if (row) {
          const detailRow = row.nextElementSibling;
          if (detailRow?.classList.contains('detail-row')) {
            const isVisible = detailRow.classList.contains('visible');
            for (const r of table.querySelectorAll('tr.detail-row.visible')) {
              if (r !== detailRow) { r.classList.remove('visible'); r.previousElementSibling?.querySelector('.expand-indicator')?.classList.remove('expanded'); }
            }
            detailRow.classList.toggle('visible', !isVisible);
            row.querySelector('.expand-indicator')?.classList.toggle('expanded', !isVisible);
          }
        }
      }
    });
  }
};

const handleHookJsonToggle = (toggle) => {
  const hookId = toggle.getAttribute('data-hook-json');
  const container = document.querySelector('[data-hook-json-container="' + hookId + '"]');
  if (container) {
    if (container.style.display === 'none') {
      const script = document.querySelector('script[data-hook-detail="' + hookId + '"]');
      if (script) {
        try {
          container.textContent = '';
          const pre = document.createElement('pre');
          pre.className = 'json-view';
          pre.textContent = JSON.stringify(JSON.parse(script.textContent), null, 2);
          container.append(pre);
        } catch (err) {
          container.textContent = '';
          const span = document.createElement('span');
          span.className = 'text-muted';
          span.textContent = 'Failed to parse JSON';
          container.append(span);
        }
      }
      container.style.display = 'block';
      toggle.textContent = 'Hide JSON';
    } else {
      container.style.display = 'none';
      toggle.textContent = 'Show JSON';
    }
  }
};

const handleDeleteHook = (deleteBtn) => {
  const id = deleteBtn.getAttribute('data-entity-id');
  showConfirmDialog('Delete Hook?', 'Delete this hook? This cannot be undone.', 'Delete', async () => {
    try {
      await apiFetch('/hooks/' + encodeURIComponent(id), { method: 'DELETE' });
      showToast('Hook deleted', 'success');
      const row = document.querySelector('tr[data-entity-id="' + id + '"].clickable-row');
      if (row) { const detail = row.nextElementSibling; if (detail?.classList.contains('detail-row')) detail.remove(); row.remove(); }
    } catch (err) {
      showToast(err.message || 'Failed to delete hook', 'error');
    }
  });
};

const handleHookDetails = (detailsBtn) => {
  const hookId = detailsBtn.getAttribute('data-hook-details');
  const row = document.querySelector('tr[data-entity-id="' + hookId + '"].clickable-row');
  if (row) {
    const detailRow = row.nextElementSibling;
    if (detailRow?.classList.contains('detail-row')) {
      const isVisible = detailRow.classList.contains('visible');
      detailRow.classList.toggle('visible', !isVisible);
      row.querySelector('.expand-indicator')?.classList.toggle('expanded', !isVisible);
    }
  }
};

export const initOrgHooks = () => {
  initExpandRows();

  const searchInput = document.getElementById('hook-search');
  if (searchInput) {
    searchInput.addEventListener('input', () => {
      const query = searchInput.value.toLowerCase();
      for (const row of document.querySelectorAll('.data-table tbody tr.clickable-row')) {
        const name = (row.getAttribute('data-name') || '').toLowerCase();
        const match = !query || name.includes(query);
        row.style.display = match ? '' : 'none';
        const detail = row.nextElementSibling;
        if (detail?.classList.contains('detail-row')) {
          if (!match) { detail.classList.remove('visible'); detail.style.display = 'none'; }
          else detail.style.display = '';
        }
      }
    });
  }

  on('click', '[data-hook-json]', (e, toggle) => {
    handleHookJsonToggle(toggle);
  });

  on('click', '[data-action="delete"][data-entity-type="hook"]', (e, deleteBtn) => {
    handleDeleteHook(deleteBtn);
  });

  on('click', '[data-hook-details]', (e, detailsBtn) => {
    handleHookDetails(detailsBtn);
  });
};
