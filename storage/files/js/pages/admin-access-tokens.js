// The access-token console's two write actions: issuing a token, then showing
// the secret once, and revoking one from its row. Filtering is a link the
// server answers.

import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { on, initDelegation } from '../services/events.js';
import { showConfirmDialog } from '../services/confirm.js';

const setupSnippet = ({ pat, origin }) => [
  '# 1. Install Pi and the gateway provider config on the target machine:',
  'examples/pi/setup.sh',
  '',
  '# 2. Point Pi at this token instead of a locally minted one:',
  'mkdir -p ~/.config/systemprompt-pi',
  `printf '%s' '${pat}' > ~/.config/systemprompt-pi/token`,
  `printf '%s' '${origin}' > ~/.config/systemprompt-pi/base-url`,
  'chmod 600 ~/.config/systemprompt-pi/token',
  '',
  '# 3. Verify the token reaches the gateway:',
  `curl -fsS -X POST '${origin}/api/public/gateway/sessions' -H 'x-api-key: ${pat}'`,
].join('\n');

const panel = () => document.getElementById('create-token-panel');

const setPanelOpen = (open) => {
  document.getElementById('create-token-overlay')?.classList.toggle('is-open', open);
  panel()?.classList.toggle('is-open', open);
  if (open) panel()?.querySelector('input, select')?.focus();
};

const setPanelState = (state, ctx = {}) => {
  for (const el of panel()?.querySelectorAll('[data-panel-state]') ?? []) {
    el.hidden = el.dataset.panelState !== state;
  }
  if (state === 'success') {
    const secretEl = document.getElementById('new-token-secret');
    if (secretEl) secretEl.value = ctx.secret ?? '';
    const snippetEl = document.getElementById('new-token-setup-snippet');
    if (snippetEl) {
      snippetEl.textContent = setupSnippet({ pat: ctx.secret ?? '', origin: window.location.origin });
    }
  }
};

const resetForm = () => {
  for (const id of ['new-token-name', 'new-token-expires', 'new-token-secret', 'new-token-user']) {
    const el = document.getElementById(id);
    if (el) el.value = '';
  }
  const snippetEl = document.getElementById('new-token-setup-snippet');
  if (snippetEl) snippetEl.textContent = '';
  setPanelState('form');
};

const copySnippet = async () => {
  const text = document.getElementById('new-token-setup-snippet')?.textContent ?? '';
  if (!text) {
    showToast('Nothing to copy yet', 'error');
    return;
  }
  try {
    await navigator.clipboard.writeText(text);
    showToast('Setup snippet copied', 'success');
  } catch {
    showToast('Copy failed', 'error');
  }
};

const issueToken = async () => {
  const name = document.getElementById('new-token-name')?.value.trim();
  const userId = document.getElementById('new-token-user')?.value;
  const expiresAt = document.getElementById('new-token-expires')?.value.trim();
  if (!name) {
    showToast('Token name is required', 'error');
    return;
  }
  if (!userId) {
    showToast('Owner is required', 'error');
    return;
  }
  const body = expiresAt ? { name, expires_at: expiresAt } : { name };
  try {
    const result = await apiFetch(`/users/${encodeURIComponent(userId)}/pats`, {
      method: 'POST',
      body: JSON.stringify(body),
    });
    showToast('Access token issued', 'success');
    if (result?.secret) {
      setPanelState('success', { secret: result.secret });
    } else {
      setPanelOpen(false);
      resetForm();
      window.location.reload();
    }
  } catch (err) {
    showToast(err?.message ?? 'Failed to issue the token', 'error');
  }
};

const revokeToken = async (button) => {
  const { revokeToken: id, tokenUser: userId, tokenName: name } = button.dataset;
  if (!id || !userId) return;
  await showConfirmDialog(
    'Revoke access token',
    `"${name}" stops authenticating the moment it is revoked. This cannot be undone.`,
    'Revoke',
    async () => {
      try {
        await apiFetch(`/users/${encodeURIComponent(userId)}/pats/${encodeURIComponent(id)}`, {
          method: 'DELETE',
        });
        showToast('Access token revoked', 'success');
        window.location.reload();
      } catch (err) {
        showToast(err?.message ?? 'Failed to revoke the token', 'error');
      }
    },
  );
};

const init = () => {
  initDelegation();
  on('click', '[data-revoke-token]', (event, target) => revokeToken(target));
  on('click', '[data-action="create-token"]', () => setPanelOpen(true));
  on('click', '#create-token-overlay', () => setPanelOpen(false));
  on('click', '#create-token-panel .sp-panel-close', () => setPanelOpen(false));
  on('click', '#create-token-panel [data-action="cancel"]', () => {
    setPanelOpen(false);
    resetForm();
  });
  on('click', '#create-token-panel [data-action="done"]', () => {
    setPanelOpen(false);
    resetForm();
    window.location.reload();
  });
  on('click', '#create-token-panel [data-action="copy-snippet"]', copySnippet);
  on('click', '#create-token-panel [data-action="save"]', issueToken);
};

init();
