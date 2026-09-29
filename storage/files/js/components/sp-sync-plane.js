// The sync-plane component's actions, shared by every page that renders
// one: an owner page's Sync tab and the import preview. Everything is
// server-rendered; this file confirms a write in words that name what will
// move, posts it to the URL the button carries, and reloads so the tables
// are re-derived by the server rather than patched here.

import { rawFetch, rawResponse, errorMessage } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { on, initDelegation } from '../services/events.js';

const dialog = (name) => document.querySelector(`dialog[data-dialog="${name}"]`);
const field = (name) => document.querySelector(`[data-sync="${name}"]`);

const closeDialogs = () => {
  for (const el of document.querySelectorAll('dialog[data-dialog^="sync-"]')) el.close();
};

let pending = null;

const CONFIRM = {
  insert_only: (button) => ({
    title: 'Apply all new',
    body: `Add ${button.dataset.count} row(s) the declaration holds and this database lacks. Nothing existing is changed or removed.`,
  }),
  overwrite: (button) => {
    const deleted = Number(button.dataset.count ?? 0);
    const console = Number(button.dataset.dashboard ?? 0);
    const kept = Number(button.dataset.kept ?? 0);
    return {
      title: 'Replace database with code',
      body:
        `Make this database equal the declaration. ${deleted} row(s) will be deleted: ` +
        `${deleted - console} that code removed and ${console} written from the console on entities the code still names. ` +
        `${kept} row(s) written here or by a bundle on entities the code no longer names are kept. ` +
        `Per-person overrides are never touched.`,
    };
  },
};

const askSync = (button) => {
  const words = CONFIRM[button.dataset.mode]?.(button);
  if (!words) return;
  // Why: a card narrowed to one entity applies for that entity alone; the
  // server refuses a participant's apply that names nothing.
  const entities = button.dataset.entities ? [button.dataset.entities] : [];
  pending = { url: button.dataset.applyUrl, plane: button.dataset.plane, mode: button.dataset.mode, entities };
  field('confirm-title').textContent = words.title;
  field('confirm-body').textContent = words.body;
  field('confirm-reason').value = '';
  dialog('sync-confirm')?.showModal();
};

// Why: a plane's own apply takes `{mode, entities}`; a staged import's apply
// also names the plane (or `all`). Sending all three is harmless to either.
const runSync = async () => {
  if (!pending) return;
  const reason = field('confirm-reason')?.value.trim() ?? '';
  if (!reason) {
    showToast('A reason is required', 'error');
    field('confirm-reason')?.focus();
    return;
  }
  const { url, plane, mode, entities } = pending;
  pending = null;
  closeDialogs();
  try {
    const result = await rawFetch(url, {
      method: 'POST',
      body: JSON.stringify({ mode, plane, entities, reason }),
    });
    const applied = result?.applied ?? [result];
    const total = applied.reduce(
      (acc, a) => {
        const o = a?.outcome ?? {};
        return {
          i: acc.i + (o.inserted ?? 0),
          u: acc.u + (o.updated ?? 0),
          d: acc.d + (o.deleted ?? 0),
        };
      },
      { i: 0, u: 0, d: 0 },
    );
    showToast(`Synced: +${total.i} ~${total.u} −${total.d}`, 'success');
    if (result?.stage_closed) {
      window.location.assign('/admin/sync?tab=archive');
    } else {
      window.location.reload();
    }
  } catch (err) {
    showToast(err?.message ?? 'Sync failed', 'error');
  }
};

// Why: the export is the plane's file (YAML), not JSON, so it is read as
// text through the raw response rather than the JSON helper.
const showExport = async (button) => {
  const url = button.dataset.exportUrl;
  const link = field('export-download');
  link.href = url;
  link.setAttribute('download', button.dataset.file?.split('/').pop() ?? 'export.yaml');
  field('export-title').textContent = `This database as ${button.dataset.file}`;
  dialog('sync-export')?.showModal();
  const target = field('export-content');
  target.textContent = 'Loading…';
  try {
    const resp = await rawResponse(url);
    if (!resp.ok) throw new Error(await errorMessage(resp));
    target.textContent = await resp.text();
  } catch (err) {
    target.textContent = err?.message ?? 'Failed to render the export.';
  }
};

const copyExport = async (button) => {
  try {
    await navigator.clipboard.writeText(field('export-content').textContent);
    button.textContent = 'Copied';
  } catch {
    showToast('Copy failed — select the text manually', 'error');
  }
};

const ACTIONS = {
  'sync-apply': askSync,
  'sync-confirm': runSync,
  'sync-export': showExport,
  'sync-copy-export': copyExport,
  'sync-close': closeDialogs,
};

let bound = false;

export const initSyncPlane = () => {
  if (bound) return;
  bound = true;
  initDelegation();
  on('click', '[data-action^="sync-"]', (event, target) => {
    const handler = ACTIONS[target.dataset.action];
    if (handler) handler(target);
  });
};
