import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { on } from '../services/events.js';

const INSTALL_TEMPLATES = {
  macos: {
    title: 'Install on macOS',
    intro: 'Run the following on the target Mac. The Homebrew tap installs the Systemprompt Bridge app and the systemprompt-bridge CLI; the login command registers this device against the gateway using the PAT above.',
    downloadLabel: 'Download .dmg from GitHub Releases',
    downloadHref: 'https://github.com/systempromptio/systemprompt-template/releases/latest',
    guideHref: '/docs/bridge/install-macos',
    snippet: ({ pat, origin }) => [
      'brew tap systempromptio/tap',
      'brew install --cask bridge',
      `systemprompt-bridge login ${pat} --gateway ${origin}`,
      'open -a "Systemprompt Bridge"'
    ].join('\n'),
  },
  windows: {
    title: 'Install on Windows',
    intro: 'Run the following in PowerShell on the target Windows machine. Scoop installs the systemprompt-bridge CLI; the login command registers this device against the gateway using the PAT above.',
    downloadLabel: 'Download .msi from GitHub Releases',
    downloadHref: 'https://github.com/systempromptio/systemprompt-template/releases/latest',
    guideHref: '/docs/bridge/install-windows',
    snippet: ({ pat, origin }) => [
      'scoop bucket add systemprompt https://github.com/systempromptio/scoop-bucket',
      'scoop install systemprompt-bridge',
      `systemprompt-bridge login ${pat} --gateway ${origin}`
    ].join('\n'),
  },
  linux: {
    title: 'Install on Linux',
    intro: 'Download the prebuilt systemprompt-bridge binary, install it onto your PATH, then register the device with the gateway using the PAT above.',
    downloadLabel: 'Download Linux binary from GitHub Releases',
    downloadHref: 'https://github.com/systempromptio/systemprompt-template/releases/latest',
    guideHref: '/docs/bridge/device-auth',
    snippet: ({ pat, origin }) => [
      'curl -sSL -o systemprompt-bridge \\',
      '  https://github.com/systempromptio/systemprompt-template/releases/latest/download/systemprompt-bridge-x86_64-unknown-linux-gnu',
      'chmod +x systemprompt-bridge',
      'sudo install -m 0755 systemprompt-bridge /usr/local/bin/systemprompt-bridge',
      `systemprompt-bridge login ${pat} --gateway ${origin}`
    ].join('\n'),
  },
};

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

const setPanelState = (state, ctx = {}) => {
  const formState = document.getElementById('new-device-form-state');
  const successState = document.getElementById('new-device-success-state');
  if (formState) formState.hidden = state !== 'form';
  if (successState) successState.hidden = state !== 'success';
  document.querySelectorAll('.panel-footer-state').forEach((el) => {
    el.hidden = el.getAttribute('data-state') !== state;
  });

  if (state === 'success') {
    const tpl = INSTALL_TEMPLATES[ctx.platform] || INSTALL_TEMPLATES.macos;
    const origin = window.location.origin;
    const snippet = tpl.snippet({ pat: ctx.secret || '', origin });
    const secretEl = document.getElementById('new-device-secret');
    if (secretEl) secretEl.value = ctx.secret || '';
    const titleEl = document.getElementById('new-device-install-title');
    if (titleEl) titleEl.textContent = tpl.title;
    const introEl = document.getElementById('new-device-install-intro');
    if (introEl) introEl.textContent = tpl.intro;
    const dlEl = document.getElementById('new-device-download-link');
    if (dlEl) {
      dlEl.textContent = `${tpl.downloadLabel} →`;
      dlEl.href = tpl.downloadHref;
    }
    const guideEl = document.getElementById('new-device-install-guide');
    if (guideEl) guideEl.href = tpl.guideHref;
    const snippetEl = document.getElementById('new-device-install-snippet');
    if (snippetEl) snippetEl.textContent = snippet;
  }
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
  const snippetEl = document.getElementById('new-device-install-snippet');
  if (snippetEl) snippetEl.textContent = '';
  setPanelState('form');
};

const copyCurrentSnippet = async () => {
  const snippetEl = document.getElementById('new-device-install-snippet');
  const text = snippetEl ? snippetEl.textContent : '';
  if (!text) { showToast('Nothing to copy yet', 'error'); return; }
  try {
    await navigator.clipboard.writeText(text);
    showToast('Install snippet copied', 'success');
  } catch {
    showToast('Copy failed', 'error');
  }
};

const bindCreatePanel = () => {
  on('click', '#create-device-overlay', () => { closeCreatePanel(); });
  on('click', '#create-device-panel .panel-close', () => { closeCreatePanel(); });
  on('click', '#create-device-panel [data-action="cancel"]', () => {
    closeCreatePanel();
    resetForm();
  });
  on('click', '#create-device-panel [data-action="done"]', () => {
    closeCreatePanel();
    resetForm();
    window.location.reload();
  });
  on('click', '#create-device-panel [data-action="copy-snippet"]', () => {
    copyCurrentSnippet();
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
        setPanelState('success', { platform, secret: result.secret });
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
