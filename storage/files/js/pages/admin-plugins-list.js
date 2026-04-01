import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { on } from '../services/events.js';
import { closeAllMenus } from '../services/dropdown.js';
import { showConfirmDialog } from '../services/confirm.js';
import { openPluginEnv } from '../services/plugin-env.js';
import { openSkillFiles } from '../services/skill-files.js';
import { handleRemoveFromPlugin, handleAddToPlugin } from '../services/plugin-resources.js';

const pluginEnvValid = {};

const updateGenerateButtons = (pluginId) => {
  const envReady = pluginEnvValid[pluginId] === true;
  for (const btn of document.querySelectorAll('[data-generate-plugin="' + pluginId + '"]')) {
    btn.disabled = !envReady;
    btn.title = !envReady ? (pluginEnvValid[pluginId] === false ? 'Configure required environment variables first' : 'Checking environment variables...') : '';
    btn.style.opacity = !envReady ? '0.4' : '';
    btn.style.cursor = !envReady ? 'not-allowed' : '';
  }
};

const loadJSZip = () => new Promise((resolve, reject) => {
  if (window.JSZip) return resolve(window.JSZip);
  const s = document.createElement('script');
  s.src = 'https://cdnjs.cloudflare.com/ajax/libs/jszip/3.10.1/jszip.min.js';
  s.integrity = 'sha384-+mbV2IY1Zk/X1p/nWllGySJSUN8uMs+gUAN10Or95UBH0fpj6GfKgPmgC5EXieXG';
  s.crossOrigin = 'anonymous'; s.onload = () => resolve(window.JSZip); s.onerror = () => reject(new Error('Failed to load JSZip'));
  document.head.append(s);
});

const handleExport = async (pluginId, btn, platform = 'unix') => {
  if (pluginEnvValid[pluginId] !== true) {
    showToast(pluginEnvValid[pluginId] === false ? 'Configure required environment variables before generating' : 'Still checking environment variables', 'error');
  } else {
    if (btn) { btn.disabled = true; btn.textContent = 'Generating...'; }
    try {
      const data = await apiFetch('/export?plugin=' + encodeURIComponent(pluginId) + '&platform=' + encodeURIComponent(platform));
      const JSZip = await loadJSZip(); const zip = new JSZip();
      const bundle = (data.plugins || data.bundles || []).find((b) => b.id === pluginId || b.plugin_id === pluginId);
      if (!bundle?.files) throw new Error('No files found in export');
      for (const f of bundle.files) zip.file(f.path, f.content, f.executable ? { unixPermissions: '755' } : {});
      const blob = await zip.generateAsync({ type: 'blob', platform: 'UNIX' });
      const url = URL.createObjectURL(blob); const a = document.createElement('a'); a.href = url; a.download = pluginId + '.zip'; a.click(); URL.revokeObjectURL(url);
      showToast('Plugin zip generated', 'success');
    } catch (err) { showToast(err.message || 'Export failed', 'error'); }
    finally { if (btn) { btn.disabled = false; btn.textContent = 'Generate'; } }
  }
};

const toggleDetailRow = (pluginId, section) => {
  const detailRow = document.querySelector('tr[data-detail-for="' + pluginId + '"]');
  if (detailRow) {
    const isVisible = detailRow.classList.contains('visible');
    for (const r of document.querySelectorAll('tr.detail-row.visible')) {
      if (r !== detailRow) { r.classList.remove('visible'); const oi = document.querySelector('[data-expand-for="' + r.getAttribute('data-detail-for') + '"]'); if (oi) oi.classList.remove('expanded'); }
    }
    const indicator = document.querySelector('[data-expand-for="' + pluginId + '"]');
    if (!isVisible) {
      detailRow.classList.add('visible'); if (indicator) indicator.classList.add('expanded');
      if (section) { for (const s of detailRow.querySelectorAll('.detail-section')) s.classList.remove('active'); const t = detailRow.querySelector('[data-section="' + section + '"]'); if (t) { t.classList.add('active'); t.scrollIntoView({ behavior: 'smooth', block: 'nearest' }); } }
    } else { detailRow.classList.remove('visible'); if (indicator) indicator.classList.remove('expanded'); }
  }
};

const applyFilters = () => {
  const sv = (document.getElementById('plugin-search')?.value || '').toLowerCase();
  const cv = document.getElementById('category-filter')?.value.toLowerCase() || '';
  for (const row of document.querySelectorAll('#plugins-table tr.clickable-row')) {
    const match = (!sv || (row.getAttribute('data-name') || '').includes(sv)) && (!cv || (row.getAttribute('data-category') || '').toLowerCase() === cv);
    row.style.display = match ? '' : 'none';
    if (!match) { const did = row.getAttribute('data-entity-id'); if (did) { const dr = document.querySelector('tr[data-detail-for="' + did + '"]'); if (dr) dr.classList.remove('visible'); } }
  }
};

const handleToggleJson = (btn) => {
  const pid = btn.getAttribute('data-toggle-json');
  const jv = document.querySelector('[data-json-for="' + pid + '"]');
  if (jv) {
    if (jv.style.display === 'none') { if (!jv.textContent.trim()) { const el = document.querySelector('[data-plugin-detail="' + pid + '"]'); if (el) try { jv.textContent = JSON.stringify(JSON.parse(el.textContent), null, 2); } catch (ex) {} } jv.style.display = ''; btn.textContent = 'Hide JSON'; }
    else { jv.style.display = 'none'; btn.textContent = 'Show JSON'; }
  }
};

const closePanel = () => { document.getElementById('config-overlay')?.classList.remove('open'); document.getElementById('config-detail-panel')?.classList.remove('open'); };

const initPluginEnvChecks = async () => {
  for (const row of document.querySelectorAll('#plugins-table tr[data-entity-type="plugin"]')) {
    const pid = row.getAttribute('data-entity-id'); if (!pid || pid === 'custom') continue;
    updateGenerateButtons(pid);
    try {
      const d = await apiFetch('/plugins/' + encodeURIComponent(pid) + '/env');
      pluginEnvValid[pid] = d.valid !== false; updateGenerateButtons(pid);
    } catch (err) { showToast(err.message || 'Failed to check plugin environment', 'error'); }
  }
};

const initPluginFilters = () => {
  const si = document.getElementById('plugin-search');
  if (si) { let t; si.addEventListener('input', () => { clearTimeout(t); t = setTimeout(applyFilters, 200); }); }
  document.getElementById('category-filter')?.addEventListener('change', applyFilters);
};

const initPluginEventHandlers = () => {
  on('click', '[data-remove-from-plugin]', (e, btn) => { e.stopPropagation(); handleRemoveFromPlugin(btn); });
  on('click', '[data-add-to-plugin]', (e, btn) => { e.stopPropagation(); handleAddToPlugin(btn); });
  on('click', '[data-expand-section]', (e, btn) => { e.stopPropagation(); toggleDetailRow(btn.getAttribute('data-plugin-id'), btn.getAttribute('data-expand-section')); });
  on('click', '[data-browse-skill]', (e, el) => { e.stopPropagation(); e.preventDefault(); openSkillFiles(el.getAttribute('data-browse-skill'), el.getAttribute('data-skill-name') || el.getAttribute('data-browse-skill')); });
  on('click', '[data-toggle-json]', (e, btn) => { e.stopPropagation(); handleToggleJson(btn); });
  on('click', 'tr.clickable-row', (e, row) => { if (!e.target.closest('[data-no-row-click],[data-action="toggle"],.actions-menu,.btn,a,input')) { toggleDetailRow(row.getAttribute('data-entity-id')); } });
  on('click', '[data-open-env]', (e, btn) => { e.stopPropagation(); openPluginEnv(btn.getAttribute('data-open-env'), btn.getAttribute('data-plugin-name') || btn.getAttribute('data-open-env')); });
  on('click', '[data-generate-plugin]', (e, btn) => { e.stopPropagation(); handleExport(btn.getAttribute('data-generate-plugin'), btn, btn.getAttribute('data-platform') || 'unix'); });
  on('click', '[data-delete-plugin]', (e, btn) => { e.stopPropagation(); closeAllMenus(); showConfirmDialog('Delete Plugin?', 'This will remove the plugin and all its configuration. This action cannot be undone.', 'Delete Plugin', async () => { try { await apiFetch('/plugins/' + encodeURIComponent(btn.getAttribute('data-delete-plugin')), { method: 'DELETE' }); showToast('Plugin deleted', 'success'); window.location.reload(); } catch (err) { showToast(err.message || 'Failed to delete', 'error'); } }); });
};

const initPluginExportAndPanel = () => {
  document.getElementById('panel-close')?.addEventListener('click', closePanel);
  document.getElementById('config-overlay')?.addEventListener('click', closePanel);
  on('click', '#export-marketplace-btn', async (e, btn) => {
    btn.disabled = true; btn.textContent = 'Generating...';
    try { const data = await apiFetch('/export?platform=unix'); const JSZip = await loadJSZip(); const zip = new JSZip(); if (data.marketplace?.content) zip.file(data.marketplace.path, data.marketplace.content); for (const b of (data.plugins || [])) { for (const f of b.files) zip.file('plugins/' + b.id + '/' + f.path, f.content, f.executable ? { unixPermissions: '755' } : {}); } const blob = await zip.generateAsync({ type: 'blob', platform: 'UNIX' }); const url = URL.createObjectURL(blob); const a = document.createElement('a'); a.href = url; a.download = 'systemprompt-marketplace.zip'; a.click(); URL.revokeObjectURL(url); showToast('Marketplace zip generated', 'success'); }
    catch (err) { showToast(err.message || 'Export failed', 'error'); } finally { btn.disabled = false; btn.textContent = 'Export'; }
  });
  window.addEventListener('env-saved', async (e) => { const pid = e.detail?.pluginId; if (pid) { try { const d = await apiFetch('/plugins/' + encodeURIComponent(pid) + '/env'); pluginEnvValid[pid] = d.valid !== false; updateGenerateButtons(pid); } catch (err) { showToast(err.message || 'Failed to refresh plugin environment', 'error'); } } });
};

export const initPluginsConfig = () => {
  initPluginEnvChecks();
  initPluginFilters();
  initPluginEventHandlers();
  initPluginExportAndPanel();
};

export const initPluginsList = initPluginsConfig;
