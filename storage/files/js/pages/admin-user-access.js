// The user detail page's Access tab: one select per entity that writes or
// removes this person's own rule, then reloads so the effective column is
// re-resolved by the server rather than guessed at here.

import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';

const RULE_TYPE = 'user';

const userId = () => document.querySelector('[data-access][data-user-id]')?.dataset.userId ?? '';

const rulesPath = (entityType, entityId) =>
  '/access-control/entity/' + encodeURIComponent(entityType) + '/' + encodeURIComponent(entityId) + '/rules';

// Why: inherit removes exactly this person's rule on this entity by id. The
// bulk "clear" endpoint clears every entity of the kind at once, which is how
// the old editor wiped a whole section when one cell was set back to inherit.
const applyState = async (select) => {
  const row = select.closest('tr');
  const { entityType, entityId, ruleId } = row?.dataset ?? {};
  const user = userId();
  if (!user || !entityType || !entityId) return;
  const previous = select.dataset.previous ?? 'inherit';
  try {
    if (select.value === 'inherit') {
      if (ruleId) {
        await apiFetch(rulesPath(entityType, entityId) + '/' + encodeURIComponent(ruleId), {
          method: 'DELETE',
        });
      }
    } else {
      await apiFetch(rulesPath(entityType, entityId), {
        method: 'POST',
        body: JSON.stringify({ rule_type: RULE_TYPE, rule_value: user, access: select.value }),
      });
    }
    select.dataset.previous = select.value;
    showToast('Rule saved', 'success');
    window.location.reload();
  } catch (err) {
    select.value = previous;
    showToast(err?.message ?? 'Failed to save the rule', 'error');
  }
};

export const init = () => {
  for (const select of document.querySelectorAll('[data-action="set-user-rule"]')) {
    select.addEventListener('change', () => applyState(select));
  }
};

init();
