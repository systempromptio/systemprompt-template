import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';

const section = document.getElementById('invites-section');
const toggleBtn = document.getElementById('btn-invite-user');
const emailInput = document.getElementById('invite-email');
const orgInput = document.getElementById('invite-org');
const departmentInput = document.getElementById('invite-department');
const createBtn = document.getElementById('btn-create-invite');
const list = document.getElementById('invites-list');

let isPlatformAdmin = false;

const copyLink = async (path) => {
  const url = window.location.origin + path;
  try {
    await navigator.clipboard.writeText(url);
    showToast('Invite link copied to clipboard', 'success');
  } catch {
    window.prompt('Copy the invite link:', url);
  }
};

const renderInvites = (invites) => {
  if (!invites.length) {
    list.innerHTML = '<p class="text-muted">No pending invites.</p>';
    return;
  }
  const rows = invites.map((inv) => `
    <tr>
      <td>${inv.email}</td>
      <td>${inv.org_name}</td>
      <td>${inv.department}</td>
      <td class="col-date">${new Date(inv.expires_at).toLocaleDateString()}</td>
      <td class="col-actions">
        <button type="button" class="btn btn-sm btn-outline" data-revoke="${inv.id}">Revoke</button>
      </td>
    </tr>`).join('');
  list.innerHTML = `
    <table class="data-table">
      <thead><tr>
        <th>Email</th><th>Organization</th><th>Department</th>
        <th class="col-date">Expires</th><th class="col-actions"></th>
      </tr></thead>
      <tbody>${rows}</tbody>
    </table>`;
  list.querySelectorAll('[data-revoke]').forEach((btn) => {
    btn.addEventListener('click', async () => {
      await apiFetch(`/invites/${btn.dataset.revoke}`, { method: 'DELETE' });
      showToast('Invite revoked', 'success');
      await loadInvites();
    });
  });
};

const loadInvites = async () => {
  try {
    const invites = await apiFetch('/invites');
    renderInvites(invites || []);
  } catch {
  }
};

const detectPlatformAdmin = async () => {
  try {
    const orgs = await apiFetch('/management/organizations');
    isPlatformAdmin = Array.isArray(orgs);
    if (isPlatformAdmin) orgInput.hidden = false;
  } catch {
    isPlatformAdmin = false;
  }
};

const createInvite = async () => {
  const email = emailInput.value.trim();
  if (!email) {
    showToast('Enter an email address to invite', 'error');
    return;
  }
  const body = { email };
  const org = orgInput.value.trim();
  if (isPlatformAdmin && org) body.org = org;
  const department = departmentInput.value.trim();
  if (department) body.department = department;

  const created = await apiFetch('/invites', {
    method: 'POST',
    body: JSON.stringify(body),
  });
  emailInput.value = '';
  await copyLink(created.invite_path);
  await loadInvites();
};

toggleBtn?.addEventListener('click', async () => {
  section.hidden = !section.hidden;
  if (!section.hidden) {
    await detectPlatformAdmin();
    await loadInvites();
  }
});
createBtn?.addEventListener('click', createInvite);
