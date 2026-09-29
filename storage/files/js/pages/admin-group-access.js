// The group detail page's Access tab: one select per entity that writes or
// removes this group's own rule. Allow and deny ask for a reason before they
// save — a shared band with no stated why is how the ledger went blank — and
// inherit removes exactly this group's rule by id.

import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { on, initDelegation } from '../services/events.js';
import { validUntilFrom } from '../services/validity.js';

const RULE_TYPE = 'group';

const groupId = () => document.querySelector('[data-group-id]')?.dataset.groupId ?? '';

const rulesPath = (entityType, entityId) =>
  '/access-control/entity/' + encodeURIComponent(entityType) + '/' + encodeURIComponent(entityId) + '/rules';

const readRules = async (entityType, entityId) => {
  const query = '?entity_type=' + encodeURIComponent(entityType) + '&entity_id=' + encodeURIComponent(entityId);
  const body = await apiFetch('/access-control' + query);
  return body?.rules ?? [];
};

const ownRule = (rules, group) =>
  rules.find((rule) => rule.rule_type === RULE_TYPE && rule.rule_value === group);

const reasonBox = (select) => select.closest('[data-group-rule]')?.querySelector('[data-group-reason]');
const reasonInput = (select) => reasonBox(select)?.querySelector('[data-group-reason-input]');
const untilInput = (select) => reasonBox(select)?.querySelector('[data-group-until-input]');

const restore = (select) => {
  select.value = select.dataset.previous ?? 'inherit';
  const box = reasonBox(select);
  if (box) box.hidden = true;
};

const removeOwn = async (select) => {
  const { entityType, entityId } = select.dataset;
  const group = groupId();
  const rules = await readRules(entityType, entityId);
  const mine = ownRule(rules, group);
  if (mine) {
    await apiFetch(rulesPath(entityType, entityId) + '/' + encodeURIComponent(mine.id), { method: 'DELETE' });
  }
};

const save = async (select) => {
  const { entityType, entityId } = select.dataset;
  const group = groupId();
  const justification = reasonInput(select)?.value.trim() ?? '';
  if (!justification) {
    showToast('A reason is required for a group rule', 'error');
    reasonInput(select)?.focus();
    return;
  }
  try {
    await apiFetch(rulesPath(entityType, entityId), {
      method: 'POST',
      body: JSON.stringify({
        rule_type: RULE_TYPE,
        rule_value: group,
        access: select.value,
        justification,
        valid_until: validUntilFrom(untilInput(select)),
      }),
    });
    select.dataset.previous = select.value;
    showToast('Rule saved. Reload to see the resolved effect.', 'success');
    const box = reasonBox(select);
    if (box) box.hidden = true;
  } catch (err) {
    restore(select);
    showToast(err.message || 'Failed to save the rule', 'error');
  }
};

// Why: inherit saves at once — there is nothing to explain about removing an
// override. Allow and deny open the reason field and wait for Save.
const onChange = async (select) => {
  if (!groupId()) return;
  if (select.value === 'inherit') {
    try {
      await removeOwn(select);
      select.dataset.previous = 'inherit';
      showToast('Rule removed. Reload to see the resolved effect.', 'success');
    } catch (err) {
      restore(select);
      showToast(err.message || 'Failed to remove the rule', 'error');
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
  initDelegation();
  for (const select of document.querySelectorAll('[data-action="set-group-rule"]')) {
    select.dataset.previous = select.value;
  }
  on('change', '[data-action="set-group-rule"]', (e, select) => onChange(select));
  on('click', '[data-action="save-group-rule"]', (e, button) => {
    const select = button.closest('[data-group-rule]')?.querySelector('[data-action="set-group-rule"]');
    if (select) save(select);
  });
  on('keydown', '[data-group-reason-input]', (e, input) => {
    if (e.key !== 'Enter') return;
    e.preventDefault();
    const select = input.closest('[data-group-rule]')?.querySelector('[data-action="set-group-rule"]');
    if (select) save(select);
  });
};

init();
