import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { on } from '../services/events.js';

const pageSection = () => document.querySelector('[data-page="models"]');
const selectedUser = () => pageSection()?.dataset.userId || '';

const form = document.querySelector('[data-models="form"]');
const userSelect = document.querySelector('[data-models="user"]');
if (form && userSelect) {
  userSelect.addEventListener('change', () => form.requestSubmit());
}

on('click', '[data-action="disable-model"]', async (e, btn) => {
  const userId = selectedUser();
  const routeId = btn.dataset.routeId;
  if (!userId || !routeId) return;
  btn.disabled = true;
  try {
    await apiFetch(`/access-control/entity/gateway_route/${encodeURIComponent(routeId)}/rules`, {
      method: 'POST',
      body: JSON.stringify({
        rule_type: 'user',
        rule_value: userId,
        access: 'deny',
        justification: 'Disabled from the Models page'
      })
    });
    showToast('Model disabled for this user — the next request is denied', 'success');
    window.location.reload();
  } catch {
    btn.disabled = false;
  }
});

on('click', '[data-action="enable-model"]', async (e, btn) => {
  const routeId = btn.dataset.routeId;
  const ruleId = btn.dataset.ruleId;
  if (!routeId || !ruleId) return;
  btn.disabled = true;
  try {
    await apiFetch(
      `/access-control/entity/gateway_route/${encodeURIComponent(routeId)}/rules/${encodeURIComponent(ruleId)}`,
      { method: 'DELETE' }
    );
    showToast('Model re-enabled for this user', 'success');
    window.location.reload();
  } catch {
    btn.disabled = false;
  }
});
