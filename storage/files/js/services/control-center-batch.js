import { on } from './events.js';
import { showConfirmDialog } from './confirm.js';

const selected = new Set();

const updateBatchBar = () => {
  const bar = document.getElementById('cc-batch-bar');
  const count = document.getElementById('cc-batch-count');
  if (bar && count) {
    if (selected.size > 0) {
      bar.hidden = false;
      count.textContent = selected.size + ' selected';
    } else {
      bar.hidden = true;
    }
  }
};

const syncSelectAll = () => {
  const all = document.getElementById('cc-select-all');
  if (all) {
    const boxes = document.querySelectorAll('.cc-session-checkbox');
    const checked = document.querySelectorAll('.cc-session-checkbox:checked');
    all.checked = boxes.length > 0 && checked.length === boxes.length;
    all.indeterminate = checked.length > 0 && checked.length < boxes.length;
  }
};

const batchUpdateStatus = async (ids, status) => {
  try {
    const res = await fetch('/control-center/api/batch-session-status', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ session_ids: [...ids], status }),
    });
    if (!res.ok) throw new Error('Batch update failed');
    for (const id of ids) {
      const card = document.querySelector('.sf-session[data-session-id="' + id + '"]');
      if (card) {
        if (status === 'deleted') {
          card.classList.add('sf-session--leaving');
          setTimeout(() => card.remove(), 400);
        } else {
          card.setAttribute('data-status', status);
        }
      }
    }
    selected.clear();
    for (const cb of document.querySelectorAll('.cc-session-checkbox')) {
      cb.checked = false;
      cb.closest('.sf-session')?.classList.remove('cc-session--selected');
    }
    updateBatchBar();
    syncSelectAll();
  } catch (e) {
    throw e;
  }
};

export const initBatch = () => {
  on('change', '.cc-session-checkbox', (e, cb) => {
    const sid = cb.getAttribute('data-session-id');
    if (cb.checked) {
      selected.add(sid);
      cb.closest('.sf-session')?.classList.add('cc-session--selected');
    } else {
      selected.delete(sid);
      cb.closest('.sf-session')?.classList.remove('cc-session--selected');
    }
    updateBatchBar();
    syncSelectAll();
  });

  on('change', '#cc-select-all', (e, el) => {
    const checked = el.checked;
    for (const box of document.querySelectorAll('.cc-session-checkbox')) {
      box.checked = checked;
      const sid = box.getAttribute('data-session-id');
      if (checked) {
        selected.add(sid);
        box.closest('.sf-session')?.classList.add('cc-session--selected');
      } else {
        selected.delete(sid);
        box.closest('.sf-session')?.classList.remove('cc-session--selected');
      }
    }
    updateBatchBar();
  });

  on('click', '.cc-batch-btn', (e, btn) => {
    if (selected.size > 0) {
      const action = btn.getAttribute('data-batch-action');
      if (action !== 'deleted') {
        batchUpdateStatus(selected, action);
      } else {
        showConfirmDialog('Confirm Delete', 'Delete ' + selected.size + ' sessions? This cannot be undone.', 'Delete', () => {
          batchUpdateStatus(selected, action);
        });
      }
    }
  });
};
