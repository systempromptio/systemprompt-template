import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';

let selectedEntities = {};

export const getSelectedEntities = () => selectedEntities;
export const setSelectedEntities = (val) => { selectedEntities = val; };

export const clearSelection = () => {
  selectedEntities = {};
  for (const cb of document.querySelectorAll('.acl-entity-checkbox, .acl-select-all')) { cb.checked = false; }
  updateSelectionCount();
};

export const updateSelectionCount = () => {
  const count = Object.keys(selectedEntities).length;
  const el = document.getElementById('acl-selection-count');
  if (el) el.textContent = count;
  const btn = document.getElementById('acl-bulk-assign');
  if (btn) btn.disabled = count === 0;
};

export const closeBulkPanel = () => {
  document.getElementById('acl-bulk-overlay')?.classList.remove('active');
  document.getElementById('acl-bulk-panel')?.classList.remove('open');
};

export const openBulkPanel = (getEntityData) => {
  const count = Object.keys(selectedEntities).length;
  if (count) {
    const firstKey = Object.keys(selectedEntities)[0];
    const [type, id] = firstKey.split(':');
    const data = getEntityData(type, id);
    const body = document.getElementById('acl-bulk-body');
    if (body) {
      body.textContent = '';
      const intro = document.createElement('p');
      intro.className = 'acl-bulk-intro';
      const strong = document.createElement('strong');
      strong.textContent = count;
      intro.append(document.createTextNode('Applying to '));
      intro.append(strong);
      intro.append(document.createTextNode(' selected entities. This will replace existing rules.'));
      body.append(intro);
      const section = document.createElement('div');
      section.className = 'acl-panel-section';
      const h3 = document.createElement('h3');
      h3.className = 'acl-panel-section-title';
      h3.textContent = 'Roles';
      section.append(h3);
      if (data?.roles) {
        for (const role of data.roles) {
          const label = document.createElement('label');
          label.className = 'acl-checkbox-row';
          const input = document.createElement('input');
          input.type = 'checkbox';
          input.name = 'role';
          input.value = role.name;
          const span = document.createElement('span');
          span.className = 'acl-checkbox-label';
          span.textContent = role.name;
          label.append(input);
          label.append(span);
          section.append(label);
        }
      }
      body.append(section);
      document.getElementById('acl-bulk-overlay')?.classList.add('active');
      document.getElementById('acl-bulk-panel')?.classList.add('open');
    }
  }
};

export const applyBulk = async () => {
  const body = document.getElementById('acl-bulk-body');
  if (body) {
    const entities = Object.keys(selectedEntities).map((key) => { const [t, i] = key.split(':'); return { entity_type: t, entity_id: i }; });
    const rules = [];
    for (const cb of body.querySelectorAll('input[name="role"]:checked')) { rules.push({ rule_type: 'role', rule_value: cb.value, access: 'allow', default_included: false }); }
    const btn = document.getElementById('acl-bulk-apply');
    if (btn) { btn.disabled = true; btn.textContent = 'Applying...'; }
    try {
      await apiFetch('/access-control/bulk', { method: 'PUT', body: JSON.stringify({ entities, rules, sync_yaml: entities.some((e) => e.entity_type === 'plugin') }) });
      showToast('Bulk assign complete', 'success'); closeBulkPanel(); window.location.reload();
    } catch (err) {
      showToast(err.message || 'Bulk assign failed', 'error'); if (btn) { btn.disabled = false; btn.textContent = 'Apply to Selected'; }
    }
  }
};
