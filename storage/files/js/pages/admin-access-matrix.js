import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';

const NEXT_STATE = { blank: 'allow', allow: 'deny', deny: 'blank' };
const SYMBOL = { allow: '✓', deny: '✕', blank: '·' };

const entityType = () => {
  const table = document.getElementById('gw-matrix');
  return (table && table.dataset.entityType) || 'gateway_route';
};

const entityUrl = (rowId) =>
  '/access-control/entity/' + encodeURIComponent(entityType()) + '/' + encodeURIComponent(rowId);

const setCellState = (cell, state, ruleId) => {
  cell.classList.remove('gw-matrix__cell--allow', 'gw-matrix__cell--deny', 'gw-matrix__cell--blank');
  cell.classList.add('gw-matrix__cell--' + state);
  cell.dataset.state = state;
  cell.dataset.ruleId = ruleId || '';
  cell.textContent = SYMBOL[state] || '·';
};

const cycleCell = async (cell) => {
  const rowId = cell.dataset.rowId;
  const kind = cell.dataset.kind;
  const value = cell.dataset.value;
  const current = cell.dataset.state || 'blank';
  const next = NEXT_STATE[current];
  if (!rowId || !value) return;

  cell.style.opacity = '0.5';
  try {
    if (next === 'blank') {
      const ruleId = cell.dataset.ruleId;
      if (ruleId) {
        await apiFetch(entityUrl(rowId) + '/rules/' + encodeURIComponent(ruleId), {
          method: 'DELETE',
        });
      }
      setCellState(cell, 'blank', null);
    } else {
      const data = await apiFetch(entityUrl(rowId) + '/rules', {
        method: 'POST',
        body: JSON.stringify({ rule_type: kind, rule_value: value, access: next }),
      });
      const ruleId = data && data.rule && data.rule.id;
      setCellState(cell, next, ruleId);
    }
  } catch (e) {
    // toast already shown
  } finally {
    cell.style.opacity = '';
  }
};

const toggleDefault = async (cb) => {
  const rowId = cb.dataset.defaultToggle;
  cb.disabled = true;
  try {
    await apiFetch(entityUrl(rowId) + '/default', {
      method: 'PATCH',
      body: JSON.stringify({ default_included: cb.checked }),
    });
    showToast('Default updated', 'success');
  } catch (e) {
    cb.checked = !cb.checked;
  } finally {
    cb.disabled = false;
  }
};

const bulkAssign = async (kind, value, access) => {
  if (!confirm('Apply ' + access + ' for ' + kind + '=' + value + ' across ALL rows?')) return;
  const cells = document.querySelectorAll(
    'td[data-cell="1"][data-kind="' + CSS.escape(kind) + '"][data-value="' + CSS.escape(value) + '"]',
  );
  let ok = 0;
  let fail = 0;
  for (const cell of cells) {
    const rowId = cell.dataset.rowId;
    try {
      const data = await apiFetch(entityUrl(rowId) + '/rules', {
        method: 'POST',
        body: JSON.stringify({ rule_type: kind, rule_value: value, access }),
      });
      const ruleId = data && data.rule && data.rule.id;
      setCellState(cell, access, ruleId);
      ok += 1;
    } catch (e) {
      fail += 1;
    }
  }
  showToast('Bulk: ' + ok + ' updated, ' + fail + ' failed', fail ? 'error' : 'success');
};

const applyTemplate = async () => {
  const dialog = document.getElementById('apply-template-dialog');
  if (!dialog) return;
  const subjectType = document.getElementById('apply-template-subject-type').value;
  const subjectValue = document.getElementById('apply-template-subject-value').value.trim();
  const action = document.getElementById('apply-template-action').value;
  if (!subjectValue) {
    showToast('Subject value required', 'error');
    return;
  }
  try {
    const resp = await apiFetch('/access-control/bulk-template', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        entity_type: entityType(),
        subject_type: subjectType,
        subject_value: subjectValue,
        action,
      }),
    });
    showToast(`Applied to ${resp.applied} of ${resp.entity_count} entities`, 'success');
    dialog.close();
    setTimeout(() => window.location.reload(), 600);
  } catch (err) {
    showToast(err.message || 'Apply failed', 'error');
  }
};

export const initEntityAccessMatrix = () => {
  for (const cell of document.querySelectorAll('td[data-cell="1"]')) {
    cell.addEventListener('click', () => cycleCell(cell));
  }
  for (const cb of document.querySelectorAll('input[data-default-toggle]')) {
    cb.addEventListener('change', () => toggleDefault(cb));
  }
  for (const btn of document.querySelectorAll('button[data-bulk-kind]')) {
    btn.addEventListener('click', () => {
      bulkAssign(btn.dataset.bulkKind, btn.dataset.bulkValue, btn.dataset.bulkAccess);
    });
  }

  const dialog = document.getElementById('apply-template-dialog');
  const openBtn = document.getElementById('apply-template-btn');
  if (dialog && openBtn) {
    openBtn.addEventListener('click', () => {
      if (typeof dialog.showModal === 'function') dialog.showModal();
      else dialog.setAttribute('open', '');
    });
    const cancel = document.getElementById('apply-template-cancel');
    if (cancel) cancel.addEventListener('click', () => dialog.close());
    const confirm = document.getElementById('apply-template-confirm');
    if (confirm) confirm.addEventListener('click', applyTemplate);
  }
};
