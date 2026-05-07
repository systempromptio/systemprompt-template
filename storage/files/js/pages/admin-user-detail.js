import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';

const form = document.getElementById('user-edit-form');
if (form) {
  const status = document.getElementById('user-edit-status');
  form.addEventListener('submit', async (event) => {
    event.preventDefault();
    const userId = form.dataset.userId;
    if (!userId) return;
    const data = new FormData(form);
    const rolesRaw = (data.get('roles') || '').toString();
    const roles = rolesRaw.split(',').map((r) => r.trim()).filter(Boolean);
    const body = {
      display_name: (data.get('display_name') || '').toString(),
      email: (data.get('email') || '').toString(),
      roles,
      is_active: form.elements.namedItem('is_active').checked,
      department: (data.get('department') || '').toString(),
    };
    if (status) status.textContent = 'Saving…';
    try {
      await apiFetch('/users/' + encodeURIComponent(userId), {
        method: 'PUT',
        body: JSON.stringify(body),
      });
      if (status) status.textContent = 'Saved';
      showToast('User updated', 'success');
      setTimeout(() => window.location.reload(), 600);
    } catch (err) {
      const msg = err && err.message ? err.message : 'Failed to update user';
      if (status) status.textContent = '';
      showToast(msg, 'error');
    }
  });
}
