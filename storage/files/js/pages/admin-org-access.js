import { on } from '../services/events.js';
import {
  getSelectedEntities, setSelectedEntities, clearSelection, updateSelectionCount,
  closeBulkPanel, openBulkPanel, applyBulk
} from './admin-access-bulk.js';
import { getEntityData, openSidePanel, closeSidePanel, savePanelRules } from './admin-org-access-panel.js';

let activeTab = 'plugins';

const debounce = (fn, ms) => {
  let timer;
  return (...args) => { clearTimeout(timer); timer = setTimeout(() => fn(...args), ms); };
};

const filterRows = () => {
  const query = (document.getElementById('acl-search')?.value || '').toLowerCase();
  const roleFilter = document.getElementById('acl-role-filter')?.value || '';
  const panel = document.querySelector('[data-acl-panel="' + activeTab + '"]');
  if (panel) {
    for (const row of panel.querySelectorAll('.acl-entity-row')) {
      const name = row.getAttribute('data-name') || '';
      const matchSearch = !query || name.includes(query);
      let matchRole = true;
      if (roleFilter) {
        const data = getEntityData(row.getAttribute('data-entity-type'), row.getAttribute('data-entity-id'));
        if (data) matchRole = data.roles?.some((r) => r.name === roleFilter && r.assigned);
      }
      row.hidden = !(matchSearch && matchRole);
    }
  }
};

const updateCoverage = () => {
  const panel = document.querySelector('[data-acl-panel="' + activeTab + '"]');
  if (panel) {
    const rows = panel.querySelectorAll('.acl-entity-row');
    let covered = 0;
    for (const r of rows) {
      const ind = r.querySelector('.acl-coverage-indicator');
      if (ind) {
        const parts = ind.textContent.trim().split('/');
        if (parts[0] && parseInt(parts[0], 10) > 0) covered++;
      }
    }
    const el = document.getElementById('acl-coverage-text');
    if (el) {
      el.textContent = covered + ' of ' + rows.length + ' ' + (activeTab === 'mcp' ? 'MCP servers' : activeTab) + ' have role assignments';
    }
  }
};

const bindSelectAll = () => {
  on('change', '.acl-select-all', (e, sa) => {
    const tp = sa.getAttribute('data-acl-tab-target');
    const panel = document.querySelector('[data-acl-panel="' + tp + '"]');
    if (panel) {
      const sel = getSelectedEntities();
      for (const cb of panel.querySelectorAll('.acl-entity-checkbox')) {
        cb.checked = sa.checked;
        const key = cb.getAttribute('data-entity-type') + ':' + cb.getAttribute('data-entity-id');
        if (cb.checked) sel[key] = true; else delete sel[key];
      }
      setSelectedEntities(sel);
    }
    updateSelectionCount();
  });
};

const bindEntityCheckbox = () => {
  on('change', '.acl-entity-checkbox', (e, cb) => {
    const sel = getSelectedEntities();
    const key = cb.getAttribute('data-entity-type') + ':' + cb.getAttribute('data-entity-id');
    if (cb.checked) sel[key] = true; else delete sel[key];
    setSelectedEntities(sel);
    updateSelectionCount();
  });
};

const bindTabBar = (tabBar) => {
  tabBar.addEventListener('click', (e) => {
    const btn = e.target.closest('[data-acl-tab]');
    if (btn) {
    activeTab = btn.getAttribute('data-acl-tab');
    for (const b of tabBar.querySelectorAll('.sp-tab')) b.classList.toggle('sp-tab--active', b === btn);
    for (const p of document.querySelectorAll('[data-acl-panel]')) {
      p.hidden = p.getAttribute('data-acl-panel') !== activeTab;
    }
    clearSelection();
    updateCoverage();
    }
  });
};

const bindBulkButtons = () => {
  document.getElementById('acl-bulk-assign')?.addEventListener('click', () => openBulkPanel(getEntityData));
  document.getElementById('acl-bulk-close')?.addEventListener('click', closeBulkPanel);
  document.getElementById('acl-bulk-cancel')?.addEventListener('click', closeBulkPanel);
  document.getElementById('acl-bulk-overlay')?.addEventListener('click', closeBulkPanel);
  document.getElementById('acl-bulk-apply')?.addEventListener('click', applyBulk);
};

export const initAccessControl = () => {
  const tabBar = document.getElementById('acl-tabs');
  if (tabBar) bindTabBar(tabBar);
  document.getElementById('acl-search')?.addEventListener('input', debounce(filterRows, 200));
  document.getElementById('acl-role-filter')?.addEventListener('change', filterRows);
  on('click', '.acl-entity-row', (e, row) => {
    if (!e.target.closest('[data-no-row-click]') && e.target.tagName !== 'INPUT') {
      openSidePanel(row.getAttribute('data-entity-type'), row.getAttribute('data-entity-id'));
    }
  });
  bindSelectAll();
  bindEntityCheckbox();
  document.getElementById('acl-panel-close')?.addEventListener('click', closeSidePanel);
  document.getElementById('acl-panel-cancel')?.addEventListener('click', closeSidePanel);
  document.getElementById('acl-overlay')?.addEventListener('click', closeSidePanel);
  document.getElementById('acl-panel-save')?.addEventListener('click', savePanelRules);
  bindBulkButtons();
  updateCoverage();
};
