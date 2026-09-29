// The "Who gets this" panel on a catalog detail page: add a rule at one band
// with a reason, remove one with a reason, and the review actions shared with
// Code sync. Every write reloads, so the headline, the rule list and the
// resolved reach are re-derived by the server rather than patched here.

import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { on, initDelegation } from '../services/events.js';
import { validUntilFrom } from '../services/validity.js';
import { askReason, initAccessReview } from '../components/sp-access-review.js';

const panel = () => document.querySelector('[data-entity-access]');

const rulesPath = () => {
  const { entityType, entityId } = panel()?.dataset ?? {};
  return `/access-control/entity/${encodeURIComponent(entityType)}/${encodeURIComponent(entityId)}/rules`;
};

const showSubject = (band) => {
  for (const field of panel()?.querySelectorAll('[data-access-subject]') ?? []) {
    field.hidden = field.dataset.accessSubject !== band;
  }
};

const addRule = async (event) => {
  event.preventDefault();
  const form = event.currentTarget;
  const data = new FormData(form);
  const band = data.get('band');
  const subject = String(data.get(band) ?? '').trim();
  const reason = String(data.get('reason') ?? '').trim();
  if (!subject || !reason) {
    showToast('Choose a subject and give a reason', 'error');
    return;
  }
  try {
    await apiFetch(rulesPath(), {
      method: 'POST',
      body: JSON.stringify({
        rule_type: band,
        rule_value: subject,
        access: data.get('access'),
        justification: reason,
        valid_until: validUntilFrom(form.querySelector('[name="until"]')),
      }),
    });
    window.location.reload();
  } catch (err) {
    showToast(err?.message ?? 'Failed to add the rule', 'error');
  }
};

const removeRule = (button) => {
  const { ruleId, subject } = button.dataset;
  askReason(
    `Remove the rule for ${subject}`,
    'The subject loses whatever this rule gave them, unless another band still decides for them.',
    (reason) => apiFetch(`${rulesPath()}/${encodeURIComponent(ruleId)}?reason=${encodeURIComponent(reason)}`, {
      method: 'DELETE',
    }),
  );
};

export const init = () => {
  if (!panel()) return;
  initDelegation();
  initAccessReview();
  on('change', '[data-access-band]', (e, select) => showSubject(select.value));
  on('click', '[data-action="access-remove"]', (e, button) => removeRule(button));
  panel().querySelector('[data-access-add]')?.addEventListener('submit', addRule);
};

init();
