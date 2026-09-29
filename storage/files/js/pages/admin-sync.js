// The Code sync page and the import preview. Sources are imported in place
// through core's refresh; an archive is staged through the upload endpoint
// and previewed on its own page, where the sync-plane component does the
// applying. Nothing here patches the DOM: every outcome is a reload or a
// navigation so the server re-derives the tables.

import { apiFetch, rawFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { on, initDelegation } from '../services/events.js';
import { initSyncPlane } from '../components/sp-sync-plane.js';
import { initAccessReview } from '../components/sp-access-review.js';

const dialog = (name) => document.querySelector(`dialog[data-dialog="${name}"]`);
const field = (name) => document.querySelector(`[data-sync="${name}"]`);

const closeDialogs = () => {
  for (const el of document.querySelectorAll('dialog[data-dialog]')) el.close();
};

const askRefresh = () => {
  const out = field('refresh-result');
  out.hidden = true;
  out.textContent = '';
  dialog('refresh')?.showModal();
};

// Why: the reply is shown verbatim — `reconciled` says the new composition is
// live, and `restart_recommended` names the one case (a kit shipping hooks)
// an in-process import cannot fully serve.
const runRefresh = async (button) => {
  button.disabled = true;
  const out = field('refresh-result');
  out.hidden = false;
  out.textContent = 'Refreshing…';
  try {
    const result = await apiFetch('/sync/sources/refresh', { method: 'POST' });
    out.textContent = JSON.stringify(result, null, 2);
    let summary = 'Composition unchanged';
    if (result?.changed) {
      summary = result?.restart_recommended
        ? 'Imported and reconciled — a restart would also load the kit hooks'
        : 'Imported and reconciled in place';
    }
    showToast(summary, 'success');
    if (!result?.restarting) window.setTimeout(() => window.location.reload(), 1200);
  } catch (err) {
    out.textContent = err?.message ?? 'Refresh failed';
    showToast(err?.message ?? 'Refresh failed', 'error');
  } finally {
    button.disabled = false;
  }
};

// Why: the archive goes up as the raw body — no multipart — and the reply
// carries the preview URL; the page navigates there instead of rendering.
const stageImport = async (button, event) => {
  event.preventDefault();
  const file = field('import-file')?.files?.[0];
  if (!file) {
    showToast('Choose a .zip archive first', 'error');
    return;
  }
  button.disabled = true;
  try {
    const result = await rawFetch(button.dataset.importUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/zip' },
      body: file,
    });
    showToast(`Staged ${result?.planes?.length ?? 0} plane(s) for preview`, 'success');
    window.location.assign(result.preview_url);
  } catch (err) {
    showToast(err?.message ?? 'Upload failed', 'error');
  } finally {
    button.disabled = false;
  }
};

const discardImport = async (button) => {
  try {
    await rawFetch(button.dataset.discardUrl, { method: 'DELETE' });
    showToast('Staged import discarded', 'success');
    window.location.assign('/admin/sync?tab=archive');
  } catch (err) {
    showToast(err?.message ?? 'Discard failed', 'error');
  }
};

const ACTIONS = {
  'refresh-sources': askRefresh,
  'confirm-refresh': runRefresh,
  'stage-import': stageImport,
  'discard-import': discardImport,
  'close-dialog': closeDialogs,
};

const init = () => {
  initDelegation();
  initSyncPlane();
  initAccessReview();
  on('click', '[data-action]', (event, target) => {
    const handler = ACTIONS[target.dataset.action];
    if (handler) handler(target, event);
  });
};

init();
