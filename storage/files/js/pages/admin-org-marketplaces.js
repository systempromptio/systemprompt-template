import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';
import { on } from '../services/events.js';
import { initManagePluginsPanel, initEditPanel } from './admin-org-marketplaces-panel.js';

const showDeleteConfirm = (marketplaceId) => {
  showConfirmDialog('Delete Marketplace?', 'You are about to delete ' + marketplaceId + '. This will remove the marketplace and all plugin associations. This action cannot be undone.', 'Delete Marketplace', async () => {
    try {
      await apiFetch('/org/marketplaces/' + encodeURIComponent(marketplaceId), { method: 'DELETE' });
      showToast('Marketplace deleted', 'success');
      setTimeout(() => window.location.reload(), 500);
    } catch (err) {}
  });
};

const initSidePanel = (panelId) => {
  const panel = document.getElementById(panelId);
  if (panel) {
    const overlayId = panel.getAttribute('data-overlay') || (panelId + '-overlay');
    const overlay = document.getElementById(overlayId);
    const closeBtn = panel.querySelector('[data-panel-close]');
    const api = {
      open: () => { panel.classList.add('open'); overlay?.classList.add('active'); },
      close: () => { panel.classList.remove('open'); overlay?.classList.remove('active'); },
      setTitle: (t) => { const el = panel.querySelector('[data-panel-title]'); if (el) el.textContent = t; },
      setBody: (node) => { const el = panel.querySelector('[data-panel-body]'); if (el) { el.textContent = ''; el.append(node); } },
      setFooter: (node) => { const el = panel.querySelector('[data-panel-footer]'); if (el) { el.textContent = ''; el.append(node); } },
      panel
    };
    if (closeBtn) closeBtn.addEventListener('click', api.close);
    if (overlay) overlay.addEventListener('click', api.close);
    return api;
  }
  return null;
};

const initTableExpand = (table) => {
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

const initSearch = (searchInput, table) => {
  if (searchInput && table) {
    let timer;
    searchInput.addEventListener('input', () => {
      clearTimeout(timer);
      timer = setTimeout(() => {
        const q = searchInput.value.toLowerCase();
        for (const row of table.querySelectorAll('tbody tr.clickable-row')) {
          const name = row.getAttribute('data-name') || '';
          const match = !q || name.includes(q);
          row.style.display = match ? '' : 'none';
          if (!match) {
            const eid = row.getAttribute('data-entity-id') || '';
            const dr = table.querySelector('tr[data-detail-for="' + eid + '"]');
            if (dr) { dr.classList.remove('visible'); dr.style.display = 'none'; }
          }
        }
      }, 200);
    });
  }
};

const initJsonToggle = () => {
  on('click', '[data-toggle-json]', (e, btn) => {
    const id = btn.getAttribute('data-toggle-json');
    const container = document.querySelector('[data-json-container="' + id + '"]');
    if (container) {
      if (container.style.display === 'none') {
        const dataEl = document.querySelector('script[data-marketplace-detail="' + id + '"]');
        if (dataEl) {
          container.textContent = '';
          try {
            const pre = document.createElement('pre');
            pre.className = 'json-view';
            pre.textContent = JSON.stringify(JSON.parse(dataEl.textContent), null, 2);
            container.append(pre);
          } catch (err) {
            const p = document.createElement('p');
            p.textContent = 'Error parsing JSON';
            container.append(p);
          }
        }
        container.style.display = 'block';
        btn.textContent = 'Hide JSON';
      } else {
        container.style.display = 'none';
        btn.textContent = 'Show JSON';
      }
    }
  });
};

export const initOrgMarketplaces = () => {
  const table = document.getElementById('mkt-table');
  initTableExpand(table);
  initSearch(document.getElementById('mkt-search'), table);
  initJsonToggle();
  on('click', '[data-delete-marketplace]', (e, btn) => showDeleteConfirm(btn.getAttribute('data-delete-marketplace')));
  initManagePluginsPanel(initSidePanel('mkt-panel'));
  initEditPanel(initSidePanel('mkt-edit-panel'), showDeleteConfirm);
};
