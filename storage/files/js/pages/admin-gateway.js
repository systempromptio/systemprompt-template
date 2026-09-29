import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';
import { on, initDelegation } from '../services/events.js';

const dialog = () => document.getElementById('gateway-route-dialog');
const form = () => document.getElementById('gateway-route-form');
const errorEl = () => document.querySelector('[data-route-error]');

const rowIndexes = () =>
  Array.from(document.querySelectorAll('tr[data-route-index]')).map((tr) =>
    Number(tr.dataset.routeIndex),
  );

const setError = (message) => {
  const el = errorEl();
  if (el) el.textContent = message || '';
};

const openDialog = (index) => {
  const f = form();
  if (!f) return;
  const row = index === null ? null : document.querySelector('tr[data-route-index="' + index + '"]');
  f.dataset.index = index === null ? '' : String(index);
  const d = row ? row.dataset : {};
  f.elements.id.value = d.routeId || '';
  f.elements.name.value = d.name || '';
  f.elements.description.value = d.description || '';
  f.elements.model_pattern.value = d.modelPattern || '';
  f.elements.provider.value = d.provider || '';
  f.elements.upstream_model.value = d.upstream || '';
  setError('');
  dialog()?.showModal();
};

const body = () => {
  const f = form();
  const upstream = f.elements.upstream_model.value.trim();
  const name = f.elements.name.value.trim();
  const description = f.elements.description.value.trim();
  return {
    id: f.elements.id.value.trim(),
    name: name ? name : null,
    description: description ? description : null,
    model_pattern: f.elements.model_pattern.value.trim(),
    provider: f.elements.provider.value.trim(),
    upstream_model: upstream ? upstream : null,
  };
};

const saveRoute = async (event) => {
  event.preventDefault();
  const f = form();
  const index = f.dataset.index;
  const path = index ? '/gateway/routes/' + encodeURIComponent(index) : '/gateway/routes';
  try {
    await apiFetch(path, {
      method: index ? 'PATCH' : 'POST',
      body: JSON.stringify(body()),
    });
    window.location.reload();
  } catch (err) {
    setError(err.message || 'The gateway refused that route.');
  }
};

const deleteRoute = (index, routeId) => {
  showConfirmDialog(
    'Delete route ' + routeId,
    'Requests matching this pattern will fall through to the next route that matches.',
    'Delete',
    async () => {
      try {
        await apiFetch('/gateway/routes/' + encodeURIComponent(index), { method: 'DELETE' });
        window.location.reload();
      } catch (err) {
        showToast(err.message || 'Failed to delete the route', 'error');
      }
    },
  );
};

// Reordering sends the whole permutation, because route order is the policy
// and the endpoint validates that nothing was dropped or duplicated.
const move = async (index, offset) => {
  const order = rowIndexes();
  const at = order.indexOf(index);
  const to = at + offset;
  if (at < 0 || to < 0 || to >= order.length) return;
  order.splice(to, 0, order.splice(at, 1)[0]);
  try {
    await apiFetch('/gateway/routes/reorder', {
      method: 'POST',
      body: JSON.stringify({ order }),
    });
    window.location.reload();
  } catch (err) {
    showToast(err.message || 'Failed to reorder the routes', 'error');
  }
};

const saveSettings = async (event) => {
  event.preventDefault();
  const f = event.target;
  const status = f.querySelector('[data-settings-status]');
  try {
    await apiFetch('/gateway', {
      method: 'PATCH',
      body: JSON.stringify({
        enabled: f.elements.enabled.value === 'true',
        auth_scheme: f.elements.auth_scheme.value.trim(),
        inference_path_prefix: f.elements.inference_path_prefix.value.trim(),
      }),
    });
    if (status) status.textContent = 'Saved.';
    showToast('Gateway settings saved', 'success');
  } catch (err) {
    if (status) status.textContent = err.message || 'Save failed.';
  }
};

const probeCell = (text, mono) => {
  const td = document.createElement('td');
  if (mono) td.className = 'sp-table__cell--mono';
  td.textContent = text;
  return td;
};

const probeRow = (r) => {
  const tr = document.createElement('tr');
  tr.append(
    probeCell(r.name || r.id, false),
    probeCell(r.model_pattern, true),
    probeCell(r.provider, false),
    probeCell(r.upstream_model || '—', true),
  );
  return tr;
};

const renderProbe = (target, routes) => {
  if (!routes.length) {
    target.textContent = 'No route is visible to this account.';
    return;
  }
  const table = document.createElement('table');
  table.className = 'sp-table__el sp-table__el--nested';
  const head = document.createElement('thead');
  const headRow = document.createElement('tr');
  ['Route', 'Pattern', 'Provider', 'Upstream'].forEach((label) => {
    const th = document.createElement('th');
    th.textContent = label;
    headRow.append(th);
  });
  head.append(headRow);
  const body = document.createElement('tbody');
  body.append(...routes.map(probeRow));
  table.append(head, body);
  const wrap = document.createElement('div');
  wrap.className = 'sp-table sp-p-gateway__probe';
  wrap.append(table);
  target.replaceChildren(wrap);
};

const probe = async (event) => {
  event.preventDefault();
  const target = document.querySelector('[data-probe-result]');
  const userId = event.target.elements.user_id.value;
  if (!target || !userId) return;
  target.textContent = 'Resolving…';
  try {
    const data = await apiFetch('/gateway/catalog/for-user/' + encodeURIComponent(userId));
    renderProbe(target, data?.routes || []);
  } catch (err) {
    target.textContent = err.message || 'The catalog could not be resolved.';
  }
};

export const init = () => {
  initDelegation();
  on('click', '[data-action="route-new"]', () => openDialog(null));
  on('click', '[data-action="route-edit"]', (e, el) => openDialog(el.dataset.index));
  on('click', '[data-action="route-cancel"]', () => dialog()?.close());
  on('click', '[data-action="route-delete"]', (e, el) =>
    deleteRoute(el.dataset.index, el.dataset.routeId),
  );
  on('click', '[data-action="route-up"]', (e, el) => move(Number(el.dataset.index), -1));
  on('click', '[data-action="route-down"]', (e, el) => move(Number(el.dataset.index), 1));
  form()?.addEventListener('submit', saveRoute);
  document.getElementById('gateway-settings')?.addEventListener('submit', saveSettings);
  document.getElementById('gateway-probe')?.addEventListener('submit', probe);
};

init();
