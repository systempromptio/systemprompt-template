import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { on, initDelegation } from '../services/events.js';

const RULE_TYPE = 'group';

const groupId = () => document.querySelector('[data-group-id]')?.dataset.groupId ?? '';

const rulesPath = (entityType, entityId) =>
  '/access-control/entity/' + encodeURIComponent(entityType) + '/' + encodeURIComponent(entityId) + '/rules';

const readRules = async (entityType, entityId) => {
  const query = '?entity_type=' + encodeURIComponent(entityType) + '&entity_id=' + encodeURIComponent(entityId);
  const body = await apiFetch('/access-control' + query);
  return body?.rules ?? [];
};

const withoutOwnBand = (rules, group) =>
  rules
    .filter((rule) => !(rule.rule_type === RULE_TYPE && rule.rule_value === group))
    .map((rule) => ({ rule_type: rule.rule_type, rule_value: rule.rule_value, access: rule.access }));

const nextRules = (rules, group, state) => {
  const kept = withoutOwnBand(rules, group);
  if (state === 'inherit') return kept;
  return [...kept, { rule_type: RULE_TYPE, rule_value: group, access: state }];
};

const applyState = async (select) => {
  const { entityType, entityId } = select.dataset;
  const group = groupId();
  if (!group || !entityType || !entityId) return;
  const previous = select.dataset.previous ?? 'inherit';
  try {
    const rules = await readRules(entityType, entityId);
    await apiFetch(rulesPath(entityType, entityId), {
      method: 'PUT',
      body: JSON.stringify({ rules: nextRules(rules, group, select.value) }),
    });
    select.dataset.previous = select.value;
    showToast('Rule saved. Reload to see the resolved effect.', 'success');
  } catch (err) {
    select.value = previous;
    showToast(err.message || 'Failed to save the rule', 'error');
  }
};

export const init = () => {
  initDelegation();
  for (const select of document.querySelectorAll('[data-action="set-group-rule"]')) {
    select.dataset.previous = select.value;
  }
  on('change', '[data-action="set-group-rule"]', (e, select) => applyState(select));
};

init();
