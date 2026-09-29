// The user detail page's Access tab: one select per entity that writes or
// removes this person's own rule, then reloads so the effective column is
// re-resolved by the server rather than guessed at here. Allow and deny offer
// an optional reason before saving; inherit saves at once.

import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { validUntilFrom } from '../services/validity.js';

const RULE_TYPE = 'user';

const userId = () => document.querySelector('[data-access][data-user-id]')?.dataset.userId ?? '';

const rulesPath = (entityType, entityId) =>
  '/access-control/entity/' + encodeURIComponent(entityType) + '/' + encodeURIComponent(entityId) + '/rules';

const reasonBox = (select) => select.closest('[data-user-rule]')?.querySelector('[data-user-reason]');
const reasonInput = (select) => reasonBox(select)?.querySelector('[data-user-reason-input]');
const untilInput = (select) => reasonBox(select)?.querySelector('[data-user-until-input]');

const restore = (select) => {
  select.value = select.dataset.previous ?? 'inherit';
  const box = reasonBox(select);
  if (box) box.hidden = true;
};

// Why: inherit removes exactly this person's rule on this entity by id. The
// bulk "clear" endpoint clears every entity of the kind at once, which is how
// the old editor wiped a whole section when one cell was set back to inherit.
const removeOwn = async (select) => {
  const row = select.closest('tr');
  const { entityType, entityId, ruleId } = row?.dataset ?? {};
  if (!ruleId) return;
  await apiFetch(rulesPath(entityType, entityId) + '/' + encodeURIComponent(ruleId), { method: 'DELETE' });
};

const save = async (select) => {
  const row = select.closest('tr');
  const { entityType, entityId } = row?.dataset ?? {};
  const user = userId();
  if (!user || !entityType || !entityId) return;
  const justification = reasonInput(select)?.value.trim() || null;
  try {
    await apiFetch(rulesPath(entityType, entityId), {
      method: 'POST',
      body: JSON.stringify({
        rule_type: RULE_TYPE,
        rule_value: user,
        access: select.value,
        justification,
        valid_until: validUntilFrom(untilInput(select)),
      }),
    });
    select.dataset.previous = select.value;
    showToast('Rule saved', 'success');
    window.location.reload();
  } catch (err) {
    restore(select);
    showToast(err?.message ?? 'Failed to save the rule', 'error');
  }
};

const onChange = async (select) => {
  if (select.value === 'inherit') {
    try {
      await removeOwn(select);
      select.dataset.previous = 'inherit';
      showToast('Override removed', 'success');
      window.location.reload();
    } catch (err) {
      restore(select);
      showToast(err?.message ?? 'Failed to remove the override', 'error');
    }
    return;
  }
  const box = reasonBox(select);
  if (box) {
    box.hidden = false;
    reasonInput(select)?.focus();
  }
};

export const init = () => {
  const pane = document.querySelector('[data-access]');
  const edit = pane?.querySelector('[data-edit-permissions]');
  edit?.addEventListener('click', () => {
    const editing = pane.dataset.editing !== 'true';
    pane.dataset.editing = String(editing);
    edit.setAttribute('aria-expanded', String(editing));
    edit.textContent = editing ? 'Done editing' : 'Edit permissions';
    for (const select of pane.querySelectorAll('[data-can-edit="true"]')) {
      select.disabled = !editing;
    }
  });
  for (const select of document.querySelectorAll('[data-action="set-user-rule"]')) {
    select.addEventListener('change', () => onChange(select));
  }
  for (const button of document.querySelectorAll('[data-action="save-user-rule"]')) {
    button.addEventListener('click', () => {
      const select = button.closest('[data-user-rule]')?.querySelector('[data-action="set-user-rule"]');
      if (select) save(select);
    });
  }
  for (const input of document.querySelectorAll('[data-user-reason-input]')) {
    input.addEventListener('keydown', (e) => {
      if (e.key !== 'Enter') return;
      e.preventDefault();
      const select = input.closest('[data-user-rule]')?.querySelector('[data-action="set-user-rule"]');
      if (select) save(select);
    });
  }
};

init();
