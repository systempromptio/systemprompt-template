import { showToast } from '/js/services/toast.js';
import { rawResponse, errorMessage } from '/js/services/api.js';

const button = document.getElementById('issue-connect-code');
const output = document.getElementById('connect-code-output');
const error = document.getElementById('connect-code-error');

const showError = (message) => {
  if (!error) return;
  error.textContent = message;
  error.hidden = false;
};

button?.addEventListener('click', async () => {
  button.disabled = true;
  if (error) error.hidden = true;
  try {
    const resp = await rawResponse('/admin/api/profile/bridge-code', { method: 'POST' });
    if (!resp.ok) throw new Error((await errorMessage(resp)) || 'Could not issue a connect code');
    const block = await resp.json();
    for (const node of document.querySelectorAll('[data-connect-field]')) {
      node.textContent = block[node.dataset.connectField] ?? '';
    }
    if (output) output.hidden = false;
    button.textContent = 'Code issued — reload the page for a new one';
    showToast('Connect code issued. It is valid for ten minutes and works once.', 'success');
  } catch (err) {
    button.disabled = false;
    showError(err.message || 'Could not issue a connect code. Please try again.');
  }
});
