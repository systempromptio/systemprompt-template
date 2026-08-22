import { apiFetch, rawFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { on } from '../services/events.js';

const table = document.getElementById('user-sessions-table');
const body = document.getElementById('user-sessions-body');
const userId = table?.dataset.userId;

const text = (value, fallback = '—') =>
  value === null || value === undefined || value === '' ? fallback : String(value);

const stamp = (iso) => {
  if (!iso) return '—';
  const when = new Date(iso);
  return Number.isNaN(when.getTime()) ? '—' : when.toLocaleString();
};

const renderRows = (sessions) => {
  if (!body) return;
  if (!sessions.length) {
    body.innerHTML =
      '<tr><td colspan="7" class="text-tertiary">No sessions recorded for this user.</td></tr>';
    return;
  }
  body.innerHTML = sessions
    .map((s) => {
      const revoked = Boolean(s.revoked_at);
      const status = revoked
        ? '<span class="badge badge-gray">Revoked</span>'
        : '<span class="badge badge-green">Active</span>';
      const action = revoked
        ? ''
        : `<button type="button" class="btn btn-sm btn-danger" data-revoke-session="${s.session_id}">Revoke</button>`;
      return `<tr data-session-id="${s.session_id}">
        <td><code class="code-inline">${s.session_id}</code></td>
        <td>${text(s.session_source)}</td>
        <td>${text(s.ip_address)}</td>
        <td class="numeric col-numeric">${text(s.request_count, '0')}</td>
        <td class="col-date">${stamp(s.last_activity_at)}</td>
        <td class="col-status">${status}</td>
        <td class="col-actions">${action}</td>
      </tr>`;
    })
    .join('');
};

const loadSessions = async () => {
  if (!userId || !body) return;
  try {
    const data = await apiFetch(`/users/${encodeURIComponent(userId)}/sessions`);
    renderRows(data?.sessions ?? []);
  } catch (err) {
    body.innerHTML = `<tr><td colspan="7" class="text-tertiary">Could not load sessions: ${
      err.message || 'request failed'
    }</td></tr>`;
  }
};

on('click', '[data-revoke-session]', async (e, button) => {
  if (!userId) return;
  if (!window.confirm('Revoke this session? The device using it will be signed out.')) return;
  const sessionId = button.dataset.revokeSession;
  button.disabled = true;
  try {
    await apiFetch(
      `/users/${encodeURIComponent(userId)}/sessions/${encodeURIComponent(sessionId)}`,
      { method: 'DELETE' },
    );
    showToast('Session revoked.', 'success');
    await loadSessions();
  } catch (err) {
    button.disabled = false;
    showToast(err.message || 'Could not revoke that session.', 'error');
  }
});

on('click', '[data-revoke-device]', async (e, button) => {
  if (!window.confirm('Revoke this device token? That device will have to link again.')) return;
  button.disabled = true;
  try {
    await rawFetch(`/admin/devices/pats/${encodeURIComponent(button.dataset.revokeDevice)}`, {
      method: 'DELETE',
    });
    showToast('Device token revoked.', 'success');
    window.setTimeout(() => window.location.reload(), 600);
  } catch (err) {
    button.disabled = false;
    showToast(err.message || 'Could not revoke that device token.', 'error');
  }
});

on('click', '#revoke-all-sessions', async (e, button) => {
  if (!userId) return;
  if (!window.confirm('Sign this user out of every device?')) return;
  button.disabled = true;
  try {
    const result = await apiFetch(`/users/${encodeURIComponent(userId)}/sessions`, {
      method: 'DELETE',
    });
    showToast(`Revoked ${result?.revoked ?? 0} session(s).`, 'success');
    await loadSessions();
  } catch (err) {
    showToast(err.message || 'Could not revoke those sessions.', 'error');
  } finally {
    button.disabled = false;
  }
});

loadSessions();
