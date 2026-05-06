import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';

const KIND_LABELS = {
  department: 'Departments',
  role: 'Roles',
  user: 'Users',
};

const KINDS = ['department', 'role', 'user'];

let rolesCache = null;
let departmentsCache = null;

const fetchRoles = async () => {
  if (rolesCache) return rolesCache;
  const data = await apiFetch('/users/roles');
  rolesCache = (data && data.roles) || [];
  return rolesCache;
};

const fetchDepartments = async () => {
  if (departmentsCache) return departmentsCache;
  const data = await apiFetch('/access-control/departments');
  departmentsCache = Array.isArray(data) ? data.map((d) => d.department || d.name || d).filter(Boolean) : [];
  return departmentsCache;
};

const searchUsers = async (q) => {
  const params = new URLSearchParams();
  if (q) params.set('q', q);
  params.set('limit', '10');
  const data = await apiFetch('/users/search?' + params.toString());
  return (data && data.users) || [];
};

const entityUrl = (panel) => {
  const entityType = panel.dataset.entityType || 'gateway_route';
  const entityId = panel.dataset.entityId;
  return '/access-control/entity/' + encodeURIComponent(entityType) + '/' + encodeURIComponent(entityId);
};

const renderChip = (panel, rule) => {
  const chip = document.createElement('span');
  chip.className = 'gw-acl-chip gw-acl-chip--' + rule.access;
  chip.dataset.ruleId = rule.id;
  const label = document.createElement('span');
  label.className = 'gw-acl-chip__label';
  label.textContent = rule.rule_value;
  chip.append(label);
  const accessBadge = document.createElement('span');
  accessBadge.className = 'gw-acl-chip__access';
  accessBadge.textContent = rule.access;
  chip.append(accessBadge);
  const remove = document.createElement('button');
  remove.type = 'button';
  remove.className = 'gw-acl-chip__remove';
  remove.setAttribute('aria-label', 'Remove ' + rule.rule_value);
  remove.textContent = '×';
  remove.addEventListener('click', async () => {
    try {
      await apiFetch(entityUrl(panel) + '/rules/' + encodeURIComponent(rule.id), {
        method: 'DELETE',
      });
      await loadPanel(panel, true);
      showToast('Rule removed', 'success');
    } catch (e) {
      // toast already shown
    }
  });
  chip.append(remove);
  return chip;
};

const buildAddRow = (panel, kind) => {
  const row = document.createElement('div');
  row.className = 'gw-acl-add';

  const input = document.createElement('input');
  input.type = 'text';
  input.className = 'gw-acl-add__input';
  input.placeholder = kind === 'user' ? 'Search user…' : 'Type to filter…';
  const listId = 'gw-acl-list-' + kind + '-' + panel.dataset.entityId;
  input.setAttribute('list', listId);

  const datalist = document.createElement('datalist');
  datalist.id = listId;
  row.append(datalist);

  const populateList = async () => {
    datalist.textContent = '';
    let values = [];
    if (kind === 'role') values = await fetchRoles();
    else if (kind === 'department') values = await fetchDepartments();
    else if (kind === 'user') values = (await searchUsers(input.value)).map((u) => u.id);
    for (const v of values) {
      const opt = document.createElement('option');
      opt.value = v;
      datalist.append(opt);
    }
  };
  input.addEventListener('focus', populateList);
  if (kind === 'user') input.addEventListener('input', populateList);

  const accessSelect = document.createElement('select');
  accessSelect.className = 'gw-acl-add__access';
  for (const a of ['allow', 'deny']) {
    const opt = document.createElement('option');
    opt.value = a;
    opt.textContent = a;
    accessSelect.append(opt);
  }

  const addBtn = document.createElement('button');
  addBtn.type = 'button';
  addBtn.className = 'btn btn-small';
  addBtn.textContent = 'Add';
  addBtn.addEventListener('click', async () => {
    const value = input.value.trim();
    if (!value) return;
    addBtn.disabled = true;
    try {
      await apiFetch(entityUrl(panel) + '/rules', {
        method: 'POST',
        body: JSON.stringify({
          rule_type: kind,
          rule_value: value,
          access: accessSelect.value,
        }),
      });
      input.value = '';
      await loadPanel(panel, true);
      showToast('Rule added', 'success');
    } catch (e) {
      // toast already shown
    } finally {
      addBtn.disabled = false;
    }
  });

  row.append(input, accessSelect, addBtn);
  return row;
};

const buildSection = (panel, kind, rules) => {
  const section = document.createElement('div');
  section.className = 'gw-acl-section';
  const heading = document.createElement('h4');
  heading.className = 'gw-acl-section__title';
  heading.textContent = KIND_LABELS[kind];
  section.append(heading);

  const chipWrap = document.createElement('div');
  chipWrap.className = 'gw-acl-chips';
  const filtered = rules.filter((r) => r.rule_type === kind);
  if (filtered.length === 0) {
    const empty = document.createElement('span');
    empty.className = 'text-muted';
    empty.textContent = 'None';
    chipWrap.append(empty);
  } else {
    for (const rule of filtered) chipWrap.append(renderChip(panel, rule));
  }
  section.append(chipWrap);
  section.append(buildAddRow(panel, kind));
  return section;
};

const renderPanel = (panel, data) => {
  panel.textContent = '';

  const header = document.createElement('div');
  header.className = 'gw-acl-header';
  const defaultLabel = document.createElement('label');
  defaultLabel.className = 'gw-acl-default';
  const cb = document.createElement('input');
  cb.type = 'checkbox';
  cb.checked = !!data.default_included;
  cb.addEventListener('change', async () => {
    cb.disabled = true;
    try {
      await apiFetch(entityUrl(panel) + '/default', {
        method: 'PATCH',
        body: JSON.stringify({ default_included: cb.checked }),
      });
      showToast('Default updated', 'success');
    } catch (e) {
      cb.checked = !cb.checked;
    } finally {
      cb.disabled = false;
    }
  });
  defaultLabel.append(cb);
  const labelText = document.createElement('span');
  labelText.textContent = ' Default: include all users (allow when no rule matches)';
  defaultLabel.append(labelText);
  header.append(defaultLabel);
  panel.append(header);

  const grid = document.createElement('div');
  grid.className = 'gw-acl-grid';
  for (const kind of KINDS) grid.append(buildSection(panel, kind, data.rules || []));
  panel.append(grid);

  panel.dataset.loaded = 'true';
};

const loadPanel = async (panel, force) => {
  if (panel.dataset.loaded === 'true' && !force) return;
  try {
    const data = await apiFetch(entityUrl(panel) + '/access');
    renderPanel(panel, data || { rules: [], default_included: false });
  } catch (e) {
    panel.textContent = 'Failed to load access rules.';
  }
};

const togglePanel = async (button) => {
  const target = button.dataset.toggleAccess;
  const row = document.querySelector('tr[data-access-row-for="' + CSS.escape(target) + '"]');
  if (!row) return;
  const expanded = button.getAttribute('aria-expanded') === 'true';
  button.setAttribute('aria-expanded', expanded ? 'false' : 'true');
  if (expanded) {
    row.hidden = true;
    return;
  }
  row.hidden = false;
  const panel = row.querySelector('.gw-access-panel');
  if (panel) await loadPanel(panel, false);
};

export const initEntityAccess = () => {
  for (const btn of document.querySelectorAll('[data-toggle-access]')) {
    btn.addEventListener('click', () => togglePanel(btn));
  }
  for (const panel of document.querySelectorAll('.gw-access-panel[data-load-on-init]')) {
    loadPanel(panel, false);
  }
};
