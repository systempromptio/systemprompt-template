import { escapeHtml } from '../utils/dom.js';
import { formatRelativeTime } from '../utils/format.js';
import { on } from './events.js';

export const initExpandRows = (opts = {}) => {
  const detailAttr = opts.detailAttr || 'data-detail-for';

  on('click', 'tr[data-expandable]', (e, row) => {
    if (!e.target.closest('.actions-menu, [data-action], input, a, button')) {
      const id = row.getAttribute('data-event-id') || row.getAttribute('data-entity-id');
      if (id) {
        const detail = document.querySelector('tr[' + detailAttr + '="' + id + '"]');
        if (detail) {
          const isOpen = detail.classList.contains('visible');
          detail.classList.toggle('visible', !isOpen);
          row.classList.toggle('expanded', !isOpen);
          const indicator = row.querySelector('.expand-indicator');
          if (indicator) indicator.classList.toggle('expanded', !isOpen);
        }
      }
    }
  });
};

export const initSidePanel = (panelId, opts = {}) => {
  const panel = document.getElementById(panelId);
  if (!panel) return null;
  const closeBtn = panel.querySelector('[data-action="close-panel"]');

  const openPanel = () => {
    panel.classList.add('open');
    panel.setAttribute('aria-hidden', 'false');
  };

  const closePanel = () => {
    panel.classList.remove('open');
    panel.setAttribute('aria-hidden', 'true');
  };

  closeBtn?.addEventListener('click', closePanel);

  return { open: openPanel, close: closePanel, panel };
};

export const renderRoleBadges = (roles) => {
  if (!roles?.length) return '<span class="badge badge-gray">No roles</span>';
  return roles.map((r) =>
    '<span class="badge badge-subtle">' + escapeHtml(r) + '</span>'
  ).join(' ');
};

export const renderPluginBadges = (plugins) => {
  if (!plugins?.length) return '<span class="text-tertiary">None</span>';
  return plugins.map((p) => {
    const name = typeof p === 'string' ? p : (p.name || p.id);
    return '<span class="badge badge-subtle">' + escapeHtml(name) + '</span>';
  }).join(' ');
};

export const formatJson = (obj) => {
  if (!obj) return '';
  try {
    return JSON.stringify(obj, null, 2);
  } catch (_e) {
    return String(obj);
  }
};

export const initTimeAgo = (root = document) => {
  for (const el of root.querySelectorAll('[data-time]')) {
    el.textContent = formatRelativeTime(el.getAttribute('data-time'));
  }
};

export const initTableSearch = (inputId, tableSelector, rowSelector = 'tbody tr.clickable-row') => {
  const input = document.getElementById(inputId);
  const table = document.querySelector(tableSelector);
  if (input && table) {
    let timer = null;
    input.addEventListener('input', () => {
      clearTimeout(timer);
      timer = setTimeout(() => {
        const query = input.value.toLowerCase().trim();
        for (const row of table.querySelectorAll(rowSelector)) {
          const text = row.textContent.toLowerCase();
          const matches = !query || text.includes(query);
          row.hidden = !matches;
          const detail = row.nextElementSibling;
          if (detail?.classList.contains('detail-row')) detail.hidden = !matches;
        }
      }, 200);
    });
  }
};

export const initTableFilter = (selectId, tableSelector, dataAttr) => {
  const select = document.getElementById(selectId);
  const table = document.querySelector(tableSelector);
  if (select && table) {
    select.addEventListener('change', () => {
      const value = select.value;
      for (const row of table.querySelectorAll('tbody tr.clickable-row')) {
        const matches = !value || (row.getAttribute(dataAttr) || '') === value;
        row.hidden = !matches;
        const detail = row.nextElementSibling;
        if (detail?.classList.contains('detail-row')) detail.hidden = !matches;
      }
    });
  }
};

export const initFilters = (filterSelector, listSelector) => {
  on('click', filterSelector, (e, btn) => {
    const filterGroup = btn.closest('.filter-group');
    if (filterGroup) {
      for (const b of filterGroup.querySelectorAll(filterSelector)) {
        b.classList.toggle('active', b === btn);
      }
      const value = btn.getAttribute('data-filter');
      for (const item of document.querySelectorAll(listSelector)) {
        if (!value || value === 'all') {
          item.hidden = false;
        } else {
          item.hidden = !item.matches('[data-category="' + value + '"]');
        }
      }
    }
  });
};
