import { apiFetch, apiGet } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { on } from '../services/events.js';

const mk = (tag, props = {}, children = []) => { const node = Object.assign(document.createElement(tag), props); for (const c of children) node.append(typeof c === 'string' ? document.createTextNode(c) : c); return node; };
const fetchPlugins = () => apiGet('/plugins');
const reloadAfter = (msg, api) => { showToast(msg, 'success'); api.close(); setTimeout(() => window.location.reload(), 500); };

const savePlugins = async (panelApi, id) => {
  const ids = [...panelApi.panel.querySelectorAll('input[name="plugin_id"]:checked')].map((cb) => cb.value);
  const btn = document.getElementById('mkt-save-plugins');
  if (btn) { btn.disabled = true; btn.textContent = 'Saving...'; }
  try {
    await apiFetch('/org/marketplaces/' + encodeURIComponent(id) + '/plugins', { method: 'PUT', body: JSON.stringify({ plugin_ids: ids }) });
    reloadAfter('Plugins updated', panelApi);
  } catch (err) { showToast(err.message || 'Failed to update', 'error'); if (btn) { btn.disabled = false; btn.textContent = 'Save'; } }
};

const renderManageBody = (panelApi, plugins, mktData, id) => {
  const current = {};
  for (const pid of (mktData.plugin_ids || [])) current[pid] = true;
  const list = mk('div', { className: 'assign-panel-checklist' });
  if (!plugins.length) { list.append(mk('p', { textContent: 'No plugins available.', className: 'sp-mkt-no-plugins' })); }
  else { for (const p of plugins) { const pid = p.id || p.plugin_id; const inp = mk('input', { type: 'checkbox', name: 'plugin_id', value: pid }); if (current[pid]) inp.checked = true; const lbl = mk('label', { className: 'acl-checkbox-row sp-mkt-checkbox-row' }, [inp, mk('span', { textContent: p.name || pid })]); list.append(lbl); } }
  panelApi.setBody(list);
  const cancel = mk('button', { className: 'btn btn-secondary', textContent: 'Cancel' });
  cancel.setAttribute('data-panel-close', '');
  cancel.addEventListener('click', panelApi.close);
  const save = mk('button', { className: 'btn btn-primary', id: 'mkt-save-plugins', textContent: 'Save' });
  const frag = document.createDocumentFragment();
  frag.append(cancel); frag.append(document.createTextNode(' ')); frag.append(save);
  panelApi.setFooter(frag);
  document.getElementById('mkt-save-plugins')?.addEventListener('click', () => savePlugins(panelApi, id));
};

export const initManagePluginsPanel = (panelApi) => {
  if (panelApi) {
    on('click', '[data-manage-plugins]', async (e, btn) => {
      const id = btn.getAttribute('data-manage-plugins');
      const dataEl = document.querySelector('script[data-marketplace-detail="' + id + '"]');
      if (dataEl) {
        let mktData; try { mktData = JSON.parse(dataEl.textContent); } catch (_e) { return; }
        panelApi.setTitle('Manage Plugins - ' + (mktData.name || id));
        try { renderManageBody(panelApi, await fetchPlugins(), mktData, id); panelApi.open(); }
        catch (err) { showToast('Failed to load plugins', 'error'); }
      }
    });
  }
};

const saveExisting = async (mktId, body, aclRules, panelApi) => {
  await apiFetch('/org/marketplaces/' + encodeURIComponent(mktId), { method: 'PUT', body: JSON.stringify(body) });
  await apiFetch('/access-control/entity/marketplace/' + encodeURIComponent(mktId), { method: 'PUT', body: JSON.stringify({ rules: aclRules, sync_yaml: false }) });
  reloadAfter('Marketplace updated', panelApi);
};

const saveNew = async (form, body, aclRules, panelApi) => {
  const createBody = { ...body, id: form.querySelector('input[name="marketplace_id"]').value };
  const created = await apiFetch('/org/marketplaces', { method: 'POST', body: JSON.stringify(createBody) });
  const cid = created.id || createBody.id;
  if (aclRules.length > 0 && cid) await apiFetch('/access-control/entity/marketplace/' + encodeURIComponent(cid), { method: 'PUT', body: JSON.stringify({ rules: aclRules, sync_yaml: false }) });
  reloadAfter('Marketplace created', panelApi);
};

const handleSave = async (isEdit, mktId, panelApi) => {
  const form = document.getElementById('panel-edit-form');
  if (form) {
    const btn = document.getElementById('mkt-edit-save');
    if (btn) { btn.disabled = true; btn.textContent = 'Saving...'; }
    const pluginIds = [...form.querySelectorAll('input[name="plugin_ids"]:checked')].map((cb) => cb.value);
    const roles = [...form.querySelectorAll('input[name="roles"]:checked')].map((cb) => cb.value);
    const aclRules = roles.map((r) => ({ rule_type: 'role', rule_value: r, access: 'allow', default_included: false }));
    const body = { name: form.querySelector('input[name="name"]').value, description: form.querySelector('textarea[name="description"]').value, plugin_ids: pluginIds };
    try { if (isEdit) await saveExisting(mktId, body, aclRules, panelApi); else await saveNew(form, body, aclRules, panelApi); }
    catch (err) { showToast('Network error', 'error'); }
    if (btn) { btn.disabled = false; btn.textContent = isEdit ? 'Save Changes' : 'Create Marketplace'; }
  }
};

const buildRolesWrap = (allRoles, currentRoles) => {
  const wrap = mk('div', { className: 'sp-mkt-roles-wrap' });
  for (const r of allRoles) { const val = r.value || r; const inp = mk('input', { type: 'checkbox', name: 'roles', value: val }); if (currentRoles[val]) inp.checked = true; wrap.append(mk('label', { className: 'sp-mkt-role-label' }, [inp, ' ' + val])); }
  return mk('div', { className: 'form-group' }, [mk('label', { className: 'field-label', textContent: 'Roles' }), wrap]);
};

const buildPluginsWrap = (allPlugins, curIds) => {
  const g = mk('div', { className: 'form-group' }, [mk('label', { className: 'field-label', textContent: 'Plugins' })]);
  g.append(mk('input', { type: 'text', className: 'field-input sp-mkt-plugin-filter', placeholder: 'Filter plugins...', id: 'panel-plugin-filter' }));
  const c = mk('div', { className: 'checklist-container sp-mkt-checklist-scroll' });
  let pi = 0;
  for (const p of allPlugins) { const pid = p.id || p.plugin_id; const item = mk('div', { className: 'checklist-item' }); item.setAttribute('data-item-name', (p.name || pid).toLowerCase()); const cb = mk('input', { type: 'checkbox', name: 'plugin_ids', value: pid, id: 'panel-plugin-' + pi }); if (curIds[pid]) cb.checked = true; const lbl = mk('label', { textContent: p.name || pid }); lbl.setAttribute('for', 'panel-plugin-' + pi); item.append(cb); item.append(lbl); c.append(item); pi++; }
  g.append(c);
  return g;
};

const buildEditForm = (plugins, mktData, allRoles, isEdit) => {
  const cpIds = {}; for (const pid of (mktData.plugin_ids || [])) cpIds[pid] = true;
  const cRoles = {}; for (const r of (mktData.roles || [])) { if (r.assigned) cRoles[r.name] = true; }
  const form = mk('form', { id: 'panel-edit-form' });
  if (!isEdit) form.append(mk('div', { className: 'form-group' }, [mk('label', { className: 'field-label', textContent: 'Marketplace ID' }), mk('input', { type: 'text', className: 'field-input', name: 'marketplace_id', required: true, placeholder: 'e.g. my-marketplace' })]));
  form.append(mk('div', { className: 'form-group' }, [mk('label', { className: 'field-label', textContent: 'Name' }), mk('input', { type: 'text', className: 'field-input', name: 'name', required: true, value: mktData.name || '' })]));
  form.append(mk('div', { className: 'form-group' }, [mk('label', { className: 'field-label', textContent: 'Description' }), mk('textarea', { className: 'field-input', name: 'description', rows: 3, textContent: mktData.description || '' })]));
  form.append(buildRolesWrap(allRoles, cRoles));
  form.append(buildPluginsWrap(plugins, cpIds));
  return form;
};

const buildEditFooter = (panelApi, isEdit, mktId, showDeleteFn) => {
  const frag = document.createDocumentFragment();
  if (isEdit) { const del = mk('button', { className: 'btn btn-danger sp-mkt-delete-btn', id: 'mkt-edit-delete', textContent: 'Delete' }); frag.append(del); frag.append(document.createTextNode(' ')); }
  const cancel = mk('button', { className: 'btn btn-secondary', textContent: 'Cancel' });
  cancel.setAttribute('data-panel-close', ''); cancel.addEventListener('click', panelApi.close);
  frag.append(cancel); frag.append(document.createTextNode(' '));
  frag.append(mk('button', { className: 'btn btn-primary', id: 'mkt-edit-save', textContent: isEdit ? 'Save Changes' : 'Create Marketplace' }));
  panelApi.setFooter(frag);
  document.getElementById('panel-plugin-filter')?.addEventListener('input', function () { const q = this.value.toLowerCase(); for (const item of panelApi.panel.querySelectorAll('.checklist-item[data-item-name]')) { item.hidden = !(!q || (item.getAttribute('data-item-name') || '').includes(q)); } });
  document.getElementById('mkt-edit-save')?.addEventListener('click', () => handleSave(isEdit, mktId, panelApi));
  document.getElementById('mkt-edit-delete')?.addEventListener('click', () => { panelApi.close(); showDeleteFn(mktId); });
};

export const initEditPanel = (panelApi, showDeleteFn) => {
  if (panelApi) {
    const readJson = (id) => { const e = document.getElementById(id); if (!e) return []; try { return JSON.parse(e.textContent); } catch (_e) { return []; } };
    const allRoles = readJson('mkt-all-roles');
    const openEdit = async (mktId) => {
      const isEdit = !!mktId;
      let mktData = {};
      if (isEdit) { const d = document.querySelector('script[data-marketplace-detail="' + mktId + '"]'); if (d) try { mktData = JSON.parse(d.textContent); } catch (_e) {} }
      panelApi.setTitle(isEdit ? 'Edit Marketplace' : 'Create Marketplace');
      try { panelApi.setBody(buildEditForm(await fetchPlugins(), mktData, allRoles, isEdit)); buildEditFooter(panelApi, isEdit, mktId, showDeleteFn); panelApi.open(); }
      catch (err) { showToast('Failed to load plugins', 'error'); }
    };
    on('click', '[data-edit-marketplace]', (e, btn) => openEdit(btn.getAttribute('data-edit-marketplace')));
    on('click', '[data-create-marketplace]', (e) => { e.preventDefault(); openEdit(null); });
  }
};
