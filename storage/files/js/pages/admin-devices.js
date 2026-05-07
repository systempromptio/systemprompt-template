import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { on } from '../services/events.js';

const openCreatePanel = () => {
  const overlay = document.getElementById('create-device-overlay');
  const panel = document.getElementById('create-device-panel');
  if (overlay && panel) {
    overlay.classList.add('open');
    panel.classList.add('open');
    const first = panel.querySelector('input, select');
    if (first) setTimeout(() => first.focus(), 350);
  }
};

const closeCreatePanel = () => {
  const overlay = document.getElementById('create-device-overlay');
  const panel = document.getElementById('create-device-panel');
  if (panel) panel.classList.remove('open');
  if (overlay) overlay.classList.remove('open');
};

const resetForm = () => {
  for (const id of ['new-device-name', 'new-device-hostname', 'new-device-secret']) {
    const el = document.getElementById(id);
    if (el) el.value = '';
  }
  const userSel = document.getElementById('new-device-user');
  if (userSel) userSel.value = '';
  const platformSel = document.getElementById('new-device-platform');
  if (platformSel) platformSel.value = 'macos';
  const secretGroup = document.getElementById('new-device-secret-group');
  if (secretGroup) secretGroup.hidden = true;
};

const bindCreatePanel = () => {
  on('click', '#create-device-overlay', () => { closeCreatePanel(); });
  on('click', '#create-device-panel .panel-close', () => { closeCreatePanel(); });
  on('click', '#create-device-panel [data-action="cancel"]', () => {
    closeCreatePanel();
    resetForm();
    window.location.reload();
  });
  on('click', '#create-device-panel [data-action="save"]', async () => {
    const name = document.getElementById('new-device-name').value.trim();
    const userId = document.getElementById('new-device-user').value;
    const platform = document.getElementById('new-device-platform').value;
    const hostname = document.getElementById('new-device-hostname').value.trim();
    if (!name) { showToast('Device name is required', 'error'); return; }
    if (!userId) { showToast('Owner is required', 'error'); return; }
    try {
      const result = await apiFetch('/management/devices', {
        method: 'POST',
        body: JSON.stringify({ user_id: userId, name, platform, hostname })
      });
      showToast('Device enrolled', 'success');
      if (result && result.secret) {
        const secretEl = document.getElementById('new-device-secret');
        const secretGroup = document.getElementById('new-device-secret-group');
        if (secretEl) secretEl.value = result.secret;
        if (secretGroup) secretGroup.hidden = false;
      } else {
        closeCreatePanel();
        resetForm();
        window.location.reload();
      }
    } catch (err) {
      showToast(err.message || 'Failed to enroll device', 'error');
    }
  });
};

const bindFilters = () => {
  const search = document.getElementById('device-search');
  const platform = document.getElementById('device-platform-filter');
  const apply = () => {
    const q = (search?.value || '').toLowerCase();
    const pf = platform?.value || '';
    document.querySelectorAll('tr[data-search]').forEach((r) => {
      const matchQ = !q || r.getAttribute('data-search').includes(q);
      const matchP = !pf || r.getAttribute('data-platform') === pf;
      r.style.display = matchQ && matchP ? '' : 'none';
    });
  };
  search?.addEventListener('input', apply);
  platform?.addEventListener('change', apply);
};

export const initDevicesPage = () => {
  const page = document.querySelector('[data-page="devices"]');
  if (page) {
    bindCreatePanel();
    bindFilters();
    const createBtn = document.querySelector('[data-action="create-device"]');
    if (createBtn) createBtn.addEventListener('click', openCreatePanel);
  }
};

initDevicesPage();
