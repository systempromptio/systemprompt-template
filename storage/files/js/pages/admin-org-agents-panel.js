import { escapeHtml } from '../utils/dom.js';

const getAgentDetail = (agentId) => {
  const el = document.querySelector('script[data-agent-detail="' + agentId + '"]');
  if (!el) return null;
  try { return JSON.parse(el.textContent); } catch (e) { return null; }
};

const getAllPlugins = () => {
  const el = document.getElementById('all-plugins-data');
  if (!el) return [];
  try { return JSON.parse(el.textContent) || []; } catch (e) { return []; }
};

const buildFormField = (form, f, val) => {
  const group = document.createElement('div');
  group.className = 'form-group';
  const label = document.createElement('label');
  label.className = 'form-label';
  label.textContent = f.label;
  group.append(label);
  if (f.type === 'textarea') {
    const textarea = document.createElement('textarea');
    textarea.className = 'form-control';
    textarea.name = f.name;
    textarea.rows = f.rows || 10;
    textarea.textContent = val;
    group.append(textarea);
  } else {
    const input = document.createElement('input');
    input.type = 'text';
    input.className = 'form-control';
    input.name = f.name;
    input.value = val;
    if (f.required) input.required = true;
    group.append(input);
  }
  form.append(group);
};

const buildEditForm = (data, fields) => {
  const form = document.createElement('form');
  form.className = 'edit-panel-form';
  for (const f of fields) {
    const val = data[f.name] || '';
    buildFormField(form, f, val);
  }
  return form;
};

const buildPanelFooter = (footer, panel, overlay, saveAttr, entityId) => {
  footer.textContent = '';
  const cancelBtn = document.createElement('button');
  cancelBtn.className = 'btn btn-secondary';
  cancelBtn.setAttribute('data-panel-close', '');
  cancelBtn.textContent = 'Cancel';
  const saveBtn = document.createElement('button');
  saveBtn.className = 'btn btn-primary';
  saveBtn.setAttribute(saveAttr, '');
  if (entityId) saveBtn.setAttribute('data-entity-id', entityId);
  saveBtn.textContent = 'Save';
  footer.append(cancelBtn);
  footer.append(document.createTextNode(' '));
  footer.append(saveBtn);
  cancelBtn.addEventListener('click', () => {
    panel.classList.remove('open');
    overlay?.classList.remove('active');
  });
};

const buildChecklist = (allPlugins, currentSet) => {
  const checklist = document.createElement('div');
  checklist.className = 'assign-panel-checklist';
  if (!allPlugins.length) {
    const p = document.createElement('p');
    p.className = 'checklist-empty';
    p.textContent = 'No plugins available.';
    checklist.append(p);
    return checklist;
  }
  for (const pl of allPlugins) {
    const label = document.createElement('label');
    label.className = 'acl-checkbox-row';
    const input = document.createElement('input');
    input.type = 'checkbox';
    input.name = 'plugin_id';
    input.value = pl.id;
    if (currentSet[pl.id]) input.checked = true;
    const span = document.createElement('span');
    span.className = 'acl-checkbox-label';
    span.textContent = pl.name || pl.id;
    label.append(input);
    label.append(span);
    checklist.append(label);
  }
  return checklist;
};

const EDIT_FIELDS = [
  { name: 'name', label: 'Name', type: 'text', required: true },
  { name: 'description', label: 'Description', type: 'text' },
  { name: 'system_prompt', label: 'System Prompt', type: 'textarea', rows: 15 }
];

export const openEditPanel = (agentId, data) => {
  const panel = document.getElementById('edit-panel');
  if (panel) {
    const overlay = document.getElementById(
      panel.getAttribute('data-overlay') || 'edit-panel-overlay'
    );
    panel.querySelector('[data-panel-title]').textContent = 'Edit ' + escapeHtml(data.name || agentId);
    const body = panel.querySelector('[data-panel-body]');
    body.textContent = '';
    body.append(buildEditForm(data, EDIT_FIELDS));
    const footer = panel.querySelector('[data-panel-footer]');
    buildPanelFooter(footer, panel, overlay, 'data-edit-save');
    panel.classList.add('open');
    overlay?.classList.add('active');
  }
};

export const openAssignPanel = (agentId, agentName) => {
  const allPlugins = getAllPlugins();
  const panel = document.getElementById('assign-panel');
  if (panel) {
    const overlay = document.getElementById(
      panel.getAttribute('data-overlay') || 'assign-panel-overlay'
    );
    panel.querySelector('[data-panel-title]').textContent = 'Assign ' + (agentName || agentId);
    const data = getAgentDetail(agentId);
    const currentSet = {};
    for (const id of (data?.assigned_plugin_ids || [])) {
      currentSet[id] = true;
    }
    const assignBody = panel.querySelector('[data-panel-body]');
    assignBody.textContent = '';
    assignBody.append(buildChecklist(allPlugins, currentSet));
    const footer = panel.querySelector('[data-panel-footer]');
    buildPanelFooter(footer, panel, overlay, 'data-assign-save', agentId);
    panel.classList.add('open');
    overlay?.classList.add('active');
  }
};

export { getAgentDetail };
