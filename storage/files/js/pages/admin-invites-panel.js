import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { copyLinkPath } from '../services/clipboard.js';

const section = document.getElementById('invites-section');
const toggleBtn = document.getElementById('btn-invite-user');
const emailInput = document.getElementById('invite-email');
const orgInput = document.getElementById('invite-org');
const departmentInput = document.getElementById('invite-department');
const createBtn = document.getElementById('btn-create-invite');
const list = document.getElementById('invites-list');

let isPlatformAdmin = false;

const expiryCell = (expiresAt) => {
  const ms = new Date(expiresAt).getTime() - Date.now();
  const hours = Math.round(ms / 3_600_000);
  const urgent = hours < 24;
  const text = hours < 1
    ? 'expires within the hour'
    : (hours < 48 ? `in ${hours}h` : `in ${Math.round(hours / 24)}d`);
  return `<span class="${urgent ? 'text-danger' : ''}">${text}</span>`;
};

const matchesFilter = (inv, term) => {
  if (!term) return true;
  const haystack = `${inv.email} ${inv.org_name} ${inv.department}`.toLowerCase();
  return haystack.includes(term.toLowerCase());
};

let allInvites = [];

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
      <td class="col-date">${expiryCell(inv.expires_at)}</td>
      <td class="col-actions">
        <button type="button" class="btn btn-sm" data-regenerate="${inv.id}">Regenerate link</button>
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
  list.querySelectorAll('[data-regenerate]').forEach((btn) => {
    btn.addEventListener('click', async () => {
      try {
        const fresh = await apiFetch(`/invites/${btn.dataset.regenerate}/regenerate`, {
          method: 'POST',
        });
        await copyLinkPath(fresh.invite_path, 'New invite link copied to clipboard');
        await loadInvites();
      } catch (err) {
        showToast(err.message || 'Failed to regenerate the invite', 'error');
      }
    });
  });
};

const loadInvites = async () => {
  try {
    allInvites = (await apiFetch('/invites')) || [];
    applyFilter();
  } catch {
  }
};

const applyFilter = () => {
  const term = document.getElementById('invite-filter')?.value.trim() || '';
  renderInvites(allInvites.filter((inv) => matchesFilter(inv, term)));
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
  await copyLinkPath(created.invite_path);
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
document.getElementById('invite-filter')?.addEventListener('input', applyFilter);
