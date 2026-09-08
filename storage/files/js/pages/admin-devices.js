import { rawFetch } from '../services/api.js';
import { showConfirmDialog } from '../services/confirm.js';
import { showToast } from '../services/toast.js';
import { on } from '../services/events.js';

const LABELS = { pats: 'access token', certs: 'device certificate' };

const revoke = async (button) => {
  const kind = button.dataset.revokeKind;
  const id = button.dataset.revokeId;
  button.disabled = true;
  try {
    await rawFetch(`/api/public/admin/devices/${kind}/${encodeURIComponent(id)}`, {
      method: 'DELETE',
    });
    showToast(`Revoked ${button.dataset.revokeName}.`, 'success');
    // Why: the reload is deferred so the confirmation is actually seen. Calling
    // it straight after the toast tears the page down in the same frame, and
    // the only feedback a destructive action gives is gone before it is painted.
    window.setTimeout(() => window.location.reload(), 900);
  } catch (err) {
    button.disabled = false;
    showToast(err.message || 'Could not revoke that credential.', 'error');
  }
};

on('click', '[data-revoke-id]', (event, button) => {
  const noun = LABELS[button.dataset.revokeKind] || 'credential';
  showConfirmDialog(
    `Revoke this ${noun}?`,
    `${button.dataset.revokeName} stops working immediately and the machine holding it must enrol again.`,
    'Revoke',
    () => revoke(button),
  );
});

for (const btn of document.querySelectorAll('.sp-table__row-toggle')) {
  btn.addEventListener('click', () => {
    const target = document.getElementById(btn.getAttribute('aria-controls') || '');
    if (!target) return;
    const expanded = btn.getAttribute('aria-expanded') === 'true';
    btn.setAttribute('aria-expanded', expanded ? 'false' : 'true');
    target.hidden = expanded;
  });
}
