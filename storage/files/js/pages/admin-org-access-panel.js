import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';

let currentPanelEntity = null;

export const getEntityData = (entityType, entityId) => {
  const el = document.querySelector('[data-acl-entity="' + entityType + '-' + entityId + '"]');
  if (!el) return null;
  try { return JSON.parse(el.textContent); } catch (e) { return null; }
};

const buildEntityInfo = (entity, entityType) => {
  const info = document.createElement('div');
  info.className = 'acl-entity-info';
  const primary = document.createElement('div');
  primary.className = 'cell-primary';
  primary.textContent = entity.name || entity.id;
  info.append(primary);
  if (entity.description) {
    const secondary = document.createElement('div');
    secondary.className = 'cell-secondary';
    secondary.textContent = entity.description;
    info.append(secondary);
  }
  const badgeRow = document.createElement('div');
  badgeRow.className = 'sp-acl-badge-row';
  const typeBadge = document.createElement('span');
  typeBadge.className = 'badge badge-blue';
  typeBadge.textContent = entityType.replace('_', ' ');
  badgeRow.append(typeBadge);
  badgeRow.append(document.createTextNode(' '));
  const statusBadge = document.createElement('span');
  statusBadge.className = entity.enabled ? 'badge badge-green' : 'badge badge-gray';
  statusBadge.textContent = entity.enabled ? 'Active' : 'Disabled';
  badgeRow.append(statusBadge);
  info.append(badgeRow);
  return info;
};

const buildRoleCheckbox = (role) => {
  const label = document.createElement('label');
  label.className = 'acl-checkbox-row';
  const input = document.createElement('input');
  input.type = 'checkbox';
  input.name = 'role';
  input.value = role.name;
  if (role.assigned) input.checked = true;
  const span = document.createElement('span');
  span.className = 'acl-checkbox-label';
  span.textContent = role.name;
  label.append(input);
  label.append(span);
  return label;
};

const buildRolesSection = (entity) => {
  const section = document.createElement('div');
  section.className = 'acl-panel-section';
  const h3 = document.createElement('h3');
  h3.className = 'acl-panel-section-title';
  h3.textContent = 'Roles';
  section.append(h3);
  const desc = document.createElement('p');
  desc.className = 'acl-panel-section-desc';
  desc.textContent = 'Select which roles can access this entity. Empty means accessible to all.';
  section.append(desc);
  if (entity.roles?.length) {
    for (const role of entity.roles) section.append(buildRoleCheckbox(role));
  } else {
    const noRoles = document.createElement('p');
    noRoles.className = 'sp-acl-no-roles';
    noRoles.textContent = 'No roles defined.';
    section.append(noRoles);
  }
  return section;
};

export const openSidePanel = (entityType, entityId) => {
  const data = getEntityData(entityType, entityId);
  if (data) {
    currentPanelEntity = { type: entityType, id: entityId };
    const title = document.getElementById('acl-panel-title');
    if (title) title.textContent = data.name || data.id;
    const body = document.getElementById('acl-panel-body');
    if (body) {
      body.textContent = '';
      const frag = document.createDocumentFragment();
      frag.append(buildEntityInfo(data, entityType));
      frag.append(buildRolesSection(data));
      body.append(frag);
    }
    document.getElementById('acl-overlay')?.classList.add('active');
    document.getElementById('acl-detail-panel')?.classList.add('open');
  }
};

export const closeSidePanel = () => {
  currentPanelEntity = null;
  document.getElementById('acl-overlay')?.classList.remove('active');
  document.getElementById('acl-detail-panel')?.classList.remove('open');
};

export const savePanelRules = async () => {
  if (currentPanelEntity) {
    const body = document.getElementById('acl-panel-body');
    if (body) {
      const rules = [];
      for (const cb of body.querySelectorAll('input[name="role"]:checked')) {
        rules.push({ rule_type: 'role', rule_value: cb.value, access: 'allow', default_included: false });
      }
      const saveBtn = document.getElementById('acl-panel-save');
      if (saveBtn) { saveBtn.disabled = true; saveBtn.textContent = 'Saving...'; }
      try {
        const url = '/access-control/entity/' + encodeURIComponent(currentPanelEntity.type) + '/' + encodeURIComponent(currentPanelEntity.id);
        await apiFetch(url, { method: 'PUT', body: JSON.stringify({ rules, sync_yaml: currentPanelEntity.type === 'plugin' }) });
        showToast('Access rules updated', 'success');
        closeSidePanel();
        window.location.reload();
      } catch (err) {
        showToast(err.message || 'Failed to save rules', 'error');
        if (saveBtn) { saveBtn.disabled = false; saveBtn.textContent = 'Save Changes'; }
      }
    }
  }
};
