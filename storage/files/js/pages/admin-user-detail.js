import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';

const BASELINE_ROLES = ['user', 'admin'];

const buildRoleCheckboxes = async () => {
  const group = document.getElementById('user-roles-group');
  if (!group) return;
  const held = (group.dataset.roles || '')
    .split(',')
    .map((r) => r.trim())
    .filter(Boolean);
  let known = [];
  try {
    const roles = await apiFetch('/users/roles');
    known = Array.isArray(roles) ? roles.map((r) => (typeof r === 'string' ? r : r.role)) : [];
  } catch {
    known = [];
  }
  const all = [...new Set([...BASELINE_ROLES, ...known.filter(Boolean), ...held])].sort();
  group.innerHTML = all
    .map((role) => {
      const checked = held.includes(role) ? ' checked' : '';
      return `<label class="checkbox-label"><input type="checkbox" name="roles" value="${role}"${checked}> ${role}</label>`;
    })
    .join('');
};

const selectedRoles = () =>
  Array.from(document.querySelectorAll('#user-roles-group input[name="roles"]:checked')).map(
    (cb) => cb.value,
  );

const form = document.getElementById('user-edit-form');
if (form) {
  const status = document.getElementById('user-edit-status');
  buildRoleCheckboxes();
  form.addEventListener('submit', async (event) => {
    event.preventDefault();
    const userId = form.dataset.userId;
    if (!userId) return;
    const data = new FormData(form);
    const roles = selectedRoles();
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

const orgSection = document.getElementById('org-membership');
if (orgSection) {
  const orgSelect = document.getElementById('org-membership-org');
  const roleSelect = document.getElementById('org-membership-role');
  const saveBtn = document.getElementById('org-membership-save');
  const orgStatus = document.getElementById('org-membership-status');

  const loadOrgs = async () => {
    try {
      const orgs = await apiFetch('/management/organizations');
      const current = orgSection.dataset.orgSlug || '';
      orgSelect.innerHTML = (orgs || []).map((o) => {
        const seats = o.seat_limit == null ? `${o.seats_used}` : `${o.seats_used}/${o.seat_limit}`;
        const selected = o.slug === current ? ' selected' : '';
        return `<option value="${o.slug}"${selected}>${o.name} (${seats} seats)</option>`;
      }).join('');
      if (current) orgSelect.value = current;
    } catch {
    }
  };

  saveBtn?.addEventListener('click', async () => {
    const userId = orgSection.dataset.userId;
    if (!userId || !orgSelect.value) return;
    if (orgStatus) orgStatus.textContent = 'Saving…';
    try {
      await apiFetch('/management/users/' + encodeURIComponent(userId) + '/organization', {
        method: 'PUT',
        body: JSON.stringify({ org: orgSelect.value, org_role: roleSelect.value }),
      });
      if (orgStatus) orgStatus.textContent = 'Saved';
      showToast('Organization membership updated', 'success');
    } catch (err) {
      if (orgStatus) orgStatus.textContent = '';
    }
  });

  loadOrgs();
}
