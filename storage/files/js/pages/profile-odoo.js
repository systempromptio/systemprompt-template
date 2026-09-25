import { showToast } from '/js/services/toast.js';
import { rawResponse, errorMessage } from '/js/services/api.js';

const form = document.getElementById('odoo-link-form');
form?.addEventListener('submit', async (e) => {
  e.preventDefault();
  const login = document.getElementById('odoo-login')?.value.trim();
  const apiKey = document.getElementById('odoo-api-key')?.value.trim();
  if (!login || !apiKey) {
    showToast('Enter both your Odoo login and an API key.', 'error');
    return;
  }
  const btn = document.getElementById('odoo-link-btn');
  btn.disabled = true;
  try {
    const resp = await rawResponse('/admin/api/profile/odoo/link', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ login, api_key: apiKey }),
    });
    if (!resp.ok) throw new Error(await errorMessage(resp) || 'Connecting Odoo failed');
    showToast('Odoo account connected.', 'success');
    window.setTimeout(() => window.location.reload(), 800);
  } catch (err) {
    btn.disabled = false;
    showToast(err.message || 'Connecting Odoo failed. Please try again.', 'error');
  }
});

document.getElementById('odoo-unlink-btn')?.addEventListener('click', async (e) => {
  if (!window.confirm('Disconnect your Odoo account? Your AI tools will stop being able to read or write the CRM until you reconnect.')) return;
  const btn = e.currentTarget;
  btn.disabled = true;
  try {
    const resp = await rawResponse('/admin/api/profile/odoo/unlink', { method: 'POST' });
    if (!resp.ok) throw new Error(await errorMessage(resp) || 'Disconnect failed');
    showToast('Odoo account disconnected.', 'success');
    window.setTimeout(() => window.location.reload(), 800);
  } catch (err) {
    btn.disabled = false;
    showToast(err.message || 'Disconnect failed. Please try again.', 'error');
  }
});
