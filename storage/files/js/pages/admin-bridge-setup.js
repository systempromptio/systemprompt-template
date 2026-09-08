import { initTabs } from '/js/components/sp-tabs.js';

const hasStoredTab = (key) => {
  try { return localStorage.getItem(`sp-tabs:${key}`) !== null; } catch { return false; }
};

const ARTIFACTS = {
  macos: { label: 'macOS build coming soon \u2014 see other platforms' },
  windows: { label: 'Download for Windows', file: 'systemprompt-bridge-windows.exe' },
  'linux-x86_64': { label: 'Download for Linux (x86_64)', file: 'systemprompt-bridge-linux-x86_64.tar.gz' },
  'linux-aarch64': { label: 'Download for Linux (aarch64)', file: 'systemprompt-bridge-linux-aarch64.tar.gz' }
};

const detectPlatform = (ua) => {
  if (/Mac/i.test(ua)) return 'macos';
  if (/Win/i.test(ua)) return 'windows';
  if (/Linux|Android/i.test(ua)) {
    return /aarch64|arm64|armv8/i.test(ua) ? 'linux-aarch64' : 'linux-x86_64';
  }
  return 'windows';
};

const pill = document.getElementById('gateway-pill');
const gateway = pill?.dataset.gatewayUrl || '';
const downloadBase = pill?.dataset.downloadBase || '';

const cta = document.getElementById('sp-download-cta');
if (cta) {
  const artifact = ARTIFACTS[detectPlatform(navigator.userAgent)];
  cta.textContent = artifact.label;
  if (artifact.file) {
    cta.href = `${downloadBase}/${artifact.file}`;
  } else {
    const other = document.querySelector('.sp-download-other');
    if (other) other.open = true;
  }
}

if (pill) {
  fetch(`${gateway}/v1/auth/bridge/capabilities`)
    .then((r) => {
      pill.className = r.ok ? 'sp-pill is-ok' : 'sp-pill is-err';
      pill.textContent = r.ok ? 'Gateway reachable' : `Gateway error ${r.status}`;
    })
    .catch(() => {
      pill.className = 'sp-pill is-err';
      pill.textContent = 'Gateway unreachable';
    });
}

const strip = document.querySelector('[data-tabs="bridge-setup"]');
const tabs = strip ? initTabs(strip) : null;
if (tabs && detectPlatform(navigator.userAgent).startsWith('linux') && !hasStoredTab('bridge-setup')) {
  tabs.select('linux');
}

const wireCopy = (btnId, srcId) => {
  const btn = document.getElementById(btnId);
  const src = document.getElementById(srcId);
  if (!btn || !src) return;
  btn.addEventListener('click', () => {
    navigator.clipboard.writeText(src.innerText).then(() => {
      btn.textContent = 'Copied';
      setTimeout(() => {
        btn.textContent = 'Copy';
      }, 1500);
    });
  });
};

wireCopy('cli-copy-btn', 'cli-snippet');
wireCopy('linux-copy-btn', 'linux-snippet');
wireCopy('toml-copy-btn', 'toml-snippet');
