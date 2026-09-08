import { showToast } from '/js/services/toast.js';
import { rawResponse, errorMessage } from '/js/services/api.js';

const button = document.getElementById('issue-connect-code');
const status = document.getElementById('connect-code-status');
const expiry = document.querySelector('[data-connect-expiry]');
const error = document.getElementById('connect-code-error');
let countdown = null;

const showError = (message) => {
  if (!error) return;
  error.textContent = message;
  error.hidden = false;
};

const formatRemaining = (seconds) => {
  const m = Math.floor(seconds / 60);
  const s = String(seconds % 60).padStart(2, '0');
  return `${m}:${s}`;
};

const expire = () => {
  clearInterval(countdown);
  countdown = null;
  if (status) status.hidden = true;
  for (const node of document.querySelectorAll('[data-connect-field]')) node.textContent = '';
  for (const node of document.querySelectorAll('[data-connect-pending]')) {
    node.textContent = 'Your connection code expired. Create a new code to continue.';
    node.hidden = false;
  }
  button.disabled = false;
  button.textContent = 'Generate a new connect code';
};

const startCountdown = (seconds) => {
  clearInterval(countdown);
  const expiresAt = Date.now() + seconds * 1000;
  const tick = () => {
    const remaining = Math.ceil((expiresAt - Date.now()) / 1000);
    if (remaining <= 0) return expire();
    if (expiry) expiry.textContent = formatRemaining(remaining);

  };
  tick();
  countdown = setInterval(tick, 1000);
};

const fill = (block) => {
  for (const node of document.querySelectorAll('[data-connect-field]')) {
    node.textContent = block[node.dataset.connectField] ?? '';
  }
  for (const node of document.querySelectorAll('[data-connect-pending]')) node.hidden = true;
  if (status) status.hidden = false;
};

button?.addEventListener('click', async () => {
  button.disabled = true;
  button.textContent = 'Creating code…';
  if (error) error.hidden = true;
  try {
    const resp = await rawResponse('/admin/api/profile/bridge-code', { method: 'POST' });
    if (!resp.ok) throw new Error((await errorMessage(resp)) || 'Could not issue a connect code');
    const block = await resp.json();
    fill(block);
    startCountdown(Number(block.expires_in_seconds) || 0);
    button.textContent = 'Code issued';
    showToast('Connect code issued. It is valid for ten minutes and works once.', 'success');
  } catch (err) {
    button.disabled = false;
    button.textContent = 'Try again';
    showError(err.message || 'Could not issue a connect code. Please try again.');
  }
});
