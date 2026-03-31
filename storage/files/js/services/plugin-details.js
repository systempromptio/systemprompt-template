import { apiFetch } from './api.js';
import { showToast } from './toast.js';
import { showEmpty } from '../utils/dom.js';
import { buildEnvItem, buildOverviewRow } from './plugin-details-ui.js';

const textNode = (text) => document.createTextNode(text || '\u2014');

export const loadEnvStatus = async (pluginId, container) => {
  container.textContent = '';
  const loading = document.createElement('div');
  loading.className = 'env-loading';
  loading.textContent = 'Loading variables...';
  container.append(loading);
  try {
    const data = await apiFetch('/plugins/' + encodeURIComponent(pluginId) + '/env');
    const defs = data.definitions || [];
    const stored = data.stored || [];
    container.textContent = '';
    if (!defs.length && !stored.length) {
      showEmpty(container, 'No environment variables defined for this plugin.');
    } else {
      const storedMap = {};
      for (const v of stored) storedMap[v.var_name] = v;
      for (const def of defs) container.append(buildEnvItem(def, storedMap[def.name]));
      const btnWrap = document.createElement('div');
      btnWrap.className = 'env-btn-wrap';
      const btn = document.createElement('button');
      btn.className = 'btn btn-primary btn-sm';
      btn.setAttribute('data-open-env', pluginId);
      btn.setAttribute('data-plugin-name', pluginId);
      btn.textContent = 'Configure';
      btnWrap.append(btn);
      container.append(btnWrap);
    }
  } catch (err) {
    showEmpty(container, 'Failed to load environment variables.');
  }
};

export const buildPluginPanel = (pluginId, { openPanel, loadEnvStatusFn }) => {
  const el = document.querySelector('[data-plugin-detail="' + pluginId + '"]');
  if (el) {
    let data;
    try { data = JSON.parse(el.textContent); } catch (e) { return; }

    document.getElementById('panel-title').textContent = data.name || pluginId;
    const body = document.getElementById('panel-body');
    body.textContent = '';
    const section = document.createElement('div');
    section.className = 'config-panel-section';
    const h4 = document.createElement('h4');
    h4.textContent = 'Overview';
    section.append(h4);
    const grid = document.createElement('div');
    grid.className = 'config-overview-grid';
    const idCode = document.createElement('code');
    idCode.textContent = data.id;
    for (const n of buildOverviewRow('ID', idCode)) grid.append(n);
    const statusBadge = document.createElement('span');
    statusBadge.className = data.enabled ? 'badge badge-green' : 'badge badge-gray';
    statusBadge.textContent = data.enabled ? 'Enabled' : 'Disabled';
    for (const n of buildOverviewRow('Status', statusBadge)) grid.append(n);
    for (const n of buildOverviewRow('Version', textNode(data.version))) grid.append(n);
    for (const n of buildOverviewRow('Category', textNode(data.category))) grid.append(n);
    for (const n of buildOverviewRow('Author', textNode(data.author_name))) grid.append(n);
    for (const n of buildOverviewRow('Description', textNode(data.description))) grid.append(n);
    section.append(grid);
    body.append(section);
    const envSection = document.createElement('div');
    envSection.className = 'config-panel-section';
    const envH4 = document.createElement('h4');
    envH4.textContent = 'Environment';
    envSection.append(envH4);
    const envStatus = document.createElement('div');
    envStatus.id = 'panel-env-status';
    envStatus.textContent = 'Loading...';
    envSection.append(envStatus);
    body.append(envSection);

    const footer = document.getElementById('panel-footer');
    footer.textContent = '';
    if (data.id !== 'custom') {
      const editLink = document.createElement('a');
      editLink.href = '/admin/org/plugins/edit/?id=' + encodeURIComponent(data.id);
      editLink.className = 'btn btn-primary';
      editLink.textContent = 'Edit Plugin';
      footer.append(editLink);
      footer.append(document.createTextNode(' '));
      const envBtn = document.createElement('button');
      envBtn.className = 'btn btn-secondary';
      envBtn.setAttribute('data-open-env', data.id);
      envBtn.setAttribute('data-plugin-name', data.name);
      envBtn.textContent = 'Configure Env';
      footer.append(envBtn);
    }
    openPanel();

    if (data.id !== 'custom') {
      (loadEnvStatusFn || loadEnvStatus)(data.id, document.getElementById('panel-env-status'));
    } else {
      showEmpty(document.getElementById('panel-env-status'), 'N/A');
    }
  }
};
