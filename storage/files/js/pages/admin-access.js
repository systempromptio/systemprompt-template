import { on } from '../services/events.js';
import { clearSelection, updateSelectionCount, getSelectedEntities, setSelectedEntities, closeBulkPanel, openBulkPanel, applyBulk } from './admin-access-bulk.js';
import { openSidePanel, closeSidePanel, savePanelRules, getEntityData } from './admin-access-panel.js';

let activeTab = 'plugins';

const debounce = (fn, ms) => {
  let timer;
  return (...args) => { clearTimeout(timer); timer = setTimeout(() => fn(...args), ms); };
};

const filterRows = () => {
  const query = (document.getElementById('acl-search').value || '').toLowerCase();
  const roleFilter = document.getElementById('acl-role-filter').value;
  const panel = document.querySelector('[data-acl-panel="' + activeTab + '"]');
  if (panel) {
    for (const row of panel.querySelectorAll('.acl-entity-row')) {
      const name = row.getAttribute('data-name') || '';
      const matchesSearch = !query || name.includes(query);
      let matchesRole = true;
      if (roleFilter) {
        const data = getEntityData(row.getAttribute('data-entity-type'), row.getAttribute('data-entity-id'));
        if (data) matchesRole = data.roles && data.roles.some((r) => r.name === roleFilter && r.assigned);
      }
      row.style.display = (matchesSearch && matchesRole) ? '' : 'none';
    }
  }
};

const updateCoverage = () => {
  const panel = document.querySelector('[data-acl-panel="' + activeTab + '"]');
  if (panel) {
    const rows = panel.querySelectorAll('.acl-entity-row');
    let covered = 0;
    for (const r of rows) {
      const indicator = r.querySelector('.acl-coverage-indicator');
      if (indicator) {
        const parts = indicator.textContent.trim().split('/');
        if (parts[0] && parseInt(parts[0], 10) > 0) covered++;
      }
    }
    const el = document.getElementById('acl-coverage-text');
    if (el) {
      const label = activeTab === 'mcp' ? 'MCP servers' : activeTab;
      el.textContent = covered + ' of ' + rows.length + ' ' + label + ' have role assignments';
    }
  }
};

const bindTabBar = (tabBar) => {
  tabBar.addEventListener('click', (e) => {
    const btn = e.target.closest('[data-acl-tab]');
    if (btn) {
      activeTab = btn.getAttribute('data-acl-tab');
      for (const b of tabBar.querySelectorAll('.tab-btn')) b.classList.toggle('active', b === btn);
      for (const p of document.querySelectorAll('[data-acl-panel]')) {
        p.style.display = p.getAttribute('data-acl-panel') === activeTab ? '' : 'none';
      }
      clearSelection();
      updateCoverage();
    }
  });
};

const bindCheckboxes = () => {
  on('change', '.acl-select-all', (e, selectAll) => {
    const sel = getSelectedEntities();
    const tabTarget = selectAll.getAttribute('data-acl-tab-target');
    const panel = document.querySelector('[data-acl-panel="' + tabTarget + '"]');
    if (panel) {
      for (const cb of panel.querySelectorAll('.acl-entity-checkbox')) {
        cb.checked = selectAll.checked;
        const key = cb.getAttribute('data-entity-type') + ':' + cb.getAttribute('data-entity-id');
        if (cb.checked) sel[key] = true; else delete sel[key];
      }
      setSelectedEntities(sel);
      updateSelectionCount();
    }
  });
  on('change', '.acl-entity-checkbox', (e, cb) => {
    const sel = getSelectedEntities();
    const key = cb.getAttribute('data-entity-type') + ':' + cb.getAttribute('data-entity-id');
    if (cb.checked) sel[key] = true; else delete sel[key];
    setSelectedEntities(sel);
    updateSelectionCount();
  });
};

const bindPanelButtons = () => {
  const closeBtn = document.getElementById('acl-panel-close');
  const cancelBtn = document.getElementById('acl-panel-cancel');
  const overlay = document.getElementById('acl-overlay');
  const saveBtn = document.getElementById('acl-panel-save');
  if (closeBtn) closeBtn.addEventListener('click', closeSidePanel);
  if (cancelBtn) cancelBtn.addEventListener('click', closeSidePanel);
  if (overlay) overlay.addEventListener('click', closeSidePanel);
  if (saveBtn) saveBtn.addEventListener('click', savePanelRules);
};

const bindBulkButtons = () => {
  const bulkClose = document.getElementById('acl-bulk-close');
  const bulkCancel = document.getElementById('acl-bulk-cancel');
  const bulkOverlay = document.getElementById('acl-bulk-overlay');
  const bulkOpen = document.getElementById('acl-bulk-assign');
  const bulkApplyBtn = document.getElementById('acl-bulk-apply');
  if (bulkClose) bulkClose.addEventListener('click', closeBulkPanel);
  if (bulkCancel) bulkCancel.addEventListener('click', closeBulkPanel);
  if (bulkOverlay) bulkOverlay.addEventListener('click', closeBulkPanel);
  if (bulkOpen) bulkOpen.addEventListener('click', () => openBulkPanel(getEntityData));
  if (bulkApplyBtn) bulkApplyBtn.addEventListener('click', applyBulk);
};

export const initAccessPage = () => {
  if (document.querySelector('[data-page="access"]') || document.getElementById('acl-tabs')) {
    const tabBar = document.getElementById('acl-tabs');
    if (tabBar) bindTabBar(tabBar);
    const searchEl = document.getElementById('acl-search');
    if (searchEl) searchEl.addEventListener('input', debounce(filterRows, 200));
    const roleFilter = document.getElementById('acl-role-filter');
    if (roleFilter) roleFilter.addEventListener('change', filterRows);
    on('click', '.acl-entity-row', (e, row) => {
      if (!e.target.closest('[data-no-row-click]') && e.target.tagName !== 'INPUT') {
        openSidePanel(row.getAttribute('data-entity-type'), row.getAttribute('data-entity-id'));
      }
    });
    bindCheckboxes();
    bindPanelButtons();
    bindBulkButtons();
    updateCoverage();
  }
};
