import { renderAgentExpand } from './admin-org-agents-events-helpers.js';

const collapseOtherRows = (table, detailRow) => {
  for (const r of table.querySelectorAll('tr.detail-row.visible')) {
    if (r === detailRow) continue;
    r.classList.remove('visible');
    r.previousElementSibling?.querySelector('.expand-indicator')?.classList.remove('expanded');
  }
};

const handleRowClick = (e, table, getAgentDetail) => {
  if (!e.target.closest('[data-no-row-click],.actions-menu,.btn,a,input,.toggle-switch')) {
    const row = e.target.closest('tr.clickable-row');
    if (row) {
      const detailRow = row.nextElementSibling;
      if (detailRow?.classList.contains('detail-row')) {
        const isVisible = detailRow.classList.contains('visible');
        collapseOtherRows(table, detailRow);
        detailRow.classList.toggle('visible', !isVisible);
        row.querySelector('.expand-indicator')?.classList.toggle('expanded', !isVisible);
        if (!isVisible) {
          const content = detailRow.querySelector('[data-agent-expand]');
          if (content && !content.hasAttribute('data-loaded')) {
            content.textContent = '';
            content.append(renderAgentExpand(content.getAttribute('data-agent-expand'), getAgentDetail));
            content.setAttribute('data-loaded', 'true');
          }
        }
      }
    }
  }
};

export const initExpandRows = (getAgentDetail) => {
  const table = document.querySelector('.data-table');
  if (table) {
    table.addEventListener('click', (e) => handleRowClick(e, table, getAgentDetail));
  }
};

const setRowVisibility = (row, match) => {
  row.style.display = match ? '' : 'none';
  const detail = row.nextElementSibling;
  if (detail?.classList.contains('detail-row')) {
    if (!match) { detail.style.display = 'none'; detail.classList.remove('visible'); }
    else detail.style.display = '';
  }
};

export const initFilters = () => {
  const table = document.querySelector('.data-table');
  if (table) {
    const apply = () => {
      const q = (document.getElementById('agent-search')?.value || '').toLowerCase().trim();
      const pv = document.getElementById('plugin-filter')?.value || '';
      for (const row of table.querySelectorAll('tbody tr.clickable-row')) {
        const ms = !q || (row.getAttribute('data-name') || '').includes(q) || (row.getAttribute('data-agent-id') || '').toLowerCase().includes(q);
        setRowVisibility(row, ms && (!pv || (row.getAttribute('data-plugins') || '').includes(pv)));
      }
    };
    document.getElementById('plugin-filter')?.addEventListener('change', apply);
    let timer;
    document.getElementById('agent-search')?.addEventListener('input', () => { clearTimeout(timer); timer = setTimeout(apply, 200); });
  }
};

const formatTimeAgo = (diff) => {
  if (diff < 60) return 'just now';
  if (diff < 3600) return Math.floor(diff / 60) + 'm ago';
  if (diff < 86400) return Math.floor(diff / 3600) + 'h ago';
  if (diff < 2592000) return Math.floor(diff / 86400) + 'd ago';
  return null;
};

export const initTimeAgo = () => {
  for (const el of document.querySelectorAll('.metadata-timestamp')) {
    const iso = el.getAttribute('title') || el.textContent.trim();
    if (!iso || iso === '--') continue;
    const d = new Date(iso);
    el.textContent = formatTimeAgo(Math.floor((Date.now() - d.getTime()) / 1000)) || d.toLocaleDateString();
    el.setAttribute('title', d.toLocaleString());
  }
};
