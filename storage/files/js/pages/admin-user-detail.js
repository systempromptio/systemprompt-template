// The user detail page's write actions. Every tab is a server-rendered link,
// so this file only submits forms and revokes credentials.

import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';
import { on, initDelegation } from '../services/events.js';

const setStatus = (name, text) => {
  const el = document.querySelector(`[data-status="${name}"]`);
  if (el) el.textContent = text;
};

const reloadSoon = () => window.setTimeout(() => window.location.reload(), 600);

const userIdOf = (el) => el.closest('[data-user-id]')?.dataset.userId;

const submit = async (name, userId, path, body) => {
  setStatus(name, 'Saving…');
  try {
    await apiFetch(`/users/${encodeURIComponent(userId)}${path}`, {
      method: 'PUT',
      body: JSON.stringify(body),
    });
    setStatus(name, 'Saved');
    reloadSoon();
  } catch (err) {
    setStatus(name, '');
    showToast(err?.message ?? 'The change was refused', 'error');
  }
};

const formHandlers = {
  'user-profile': (form, userId) => {
    const data = new FormData(form);
    return submit('user-profile', userId, '', {
      display_name: String(data.get('display_name') ?? ''),
      email: String(data.get('email') ?? ''),
      is_active: form.elements.namedItem('is_active').checked,
    });
  },
  'user-roles': (form, userId) =>
    submit('user-roles', userId, '/roles', {
      roles: [...Array.from(form.querySelectorAll('input[name="roles"]:checked'), (cb) => cb.value), ...String(new FormData(form).get("additional_roles") ?? "").split(",").map((r) => r.trim()).filter(Boolean)],
    }),
  // An empty select means "no primary container": the person's spend then lands
  // in the unattributed bucket rather than in a group they are not in.
  'scope-defaults': (form, userId) => {
    const data = new FormData(form);
    return submit('scope-defaults', userId, '/scope-defaults', {
      primary_group_id: String(data.get('primary_group_id') ?? '') || null,
      primary_project_id: String(data.get('primary_project_id') ?? '') || null,
    });
  },
};

const bindForms = () => {
  for (const form of document.querySelectorAll('form[data-form]')) {
    form.addEventListener('submit', (ev) => {
      ev.preventDefault();
      const handler = formHandlers[form.dataset.form];
      if (handler) handler(form, form.dataset.userId);
    });
  }
};

const membershipPath = (kind, id, userId, adding) => {
  const base = `/${kind === 'group' ? 'groups' : 'projects'}/${encodeURIComponent(id)}/members`;
  return adding ? base : `${base}/${encodeURIComponent(userId)}`;
};

const toggleMembership = async (input) => {
  const card = input.closest('[data-membership]');
  const { userId } = card.dataset;
  const { membershipKind: kind, membershipId: id } = input.dataset;
  const adding = input.checked;
  setStatus('membership', 'Saving…');
  try {
    await apiFetch(membershipPath(kind, id, userId, adding), {
      method: adding ? 'POST' : 'DELETE',
      body: adding ? JSON.stringify({ user_id: userId }) : undefined,
    });
    setStatus('membership', 'Saved');
    reloadSoon();
  } catch (err) {
    input.checked = !adding;
    setStatus('membership', '');
    showToast(err?.message ?? 'Failed to change the membership', 'error');
  }
};

const revoke = async (path, label) => {
  try {
    await apiFetch(path, { method: 'DELETE' });
    showToast(`${label} revoked`, 'success');
    reloadSoon();
  } catch (err) {
    showToast(err?.message ?? `Failed to revoke the ${label}`, 'error');
  }
};

const issueShareToken = async (button) => {
  const userId = userIdOf(button);
  try {
    await apiFetch(`/users/${encodeURIComponent(userId)}/share-token`, { method: 'POST' });
    showToast('Share token issued; every earlier one is now void', 'success');
    reloadSoon();
  } catch (err) {
    showToast(err?.message ?? 'Failed to issue a share token', 'error');
  }
};

const bindActions = () => {
  on('change', 'input[data-membership-kind]', (_ev, input) => toggleMembership(input));

  on('click', '[data-revoke-device]', (_ev, button) => {
    const collection = button.dataset.kind === 'cert' ? 'certs' : 'pats';
    showConfirmDialog(
      'Revoke credential',
      'The device stops working on its next request.',
      'Revoke',
      () => revoke(`/devices/${collection}/${encodeURIComponent(button.dataset.revokeDevice)}`, 'credential'),
    );
  });

  on('click', '[data-revoke-session]', (_ev, button) => {
    const userId = userIdOf(button);
    showConfirmDialog('Revoke session', 'The session ends on its next request.', 'Revoke', () =>
      revoke(
        `/users/${encodeURIComponent(userId)}/sessions/${encodeURIComponent(button.dataset.revokeSession)}`,
        'session',
      ),
    );
  });

  on('click', '[data-revoke-all]', (_ev, button) => {
    showConfirmDialog(
      'Sign out everywhere',
      'Every live session for this account ends on its next request.',
      'Sign out',
      () => revoke(`/users/${encodeURIComponent(button.dataset.revokeAll)}/sessions`, 'sessions'),
    );
  });

  on('click', '[data-unlink]', (_ev, button) => {
    const userId = userIdOf(button);
    const kind = button.dataset.unlink;
    revoke(`/users/${encodeURIComponent(userId)}/${kind}-identity`, `${kind} identity`);
  });

  on('click', '[data-issue-share-token]', (_ev, button) => issueShareToken(button));
};

const init = () => {
  initDelegation();
  bindForms();
  bindActions();
};

init();
