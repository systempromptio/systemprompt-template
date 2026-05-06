// /admin/overview/governance
import {
  openStream, appendTicker, formatTime, bumpKpi,
} from '/js/pages/admin-overview-shared.js';

const url = document.currentScript?.dataset.sseUrl ?? '/admin/api/sse/overview/governance';
const denialList = document.getElementById('gov-denials');
const totalEl = document.querySelector('[data-kpi="total"]');
const allowedEl = document.querySelector('[data-kpi="allowed"]');
const deniedEl = document.querySelector('[data-kpi="denied"]');

const STAGE_FROM_POLICY = {
  scope: 'scope_check',
  scope_check: 'scope_check',
  secret_scan: 'secret_scan',
  secret_injection: 'secret_scan',
  blocklist: 'blocklist',
  tool_blocklist: 'blocklist',
  rate_limit: 'rate_limit',
};

function flashStage(key) {
  const el = document.querySelector(`[data-stage="${key}"]`);
  if (!el) return;
  const counter = el.querySelector(`[data-stage-count="${key}"]`);
  if (counter) {
    const n = (Number.parseInt(counter.textContent, 10) || 0) + 1;
    counter.textContent = n;
  }
  el.classList.add('is-firing');
  setTimeout(() => el.classList.remove('is-firing'), 800);
}

openStream(url, {
  events: {
    decision: (p) => {
      bumpKpi(totalEl);
      if (p?.decision === 'deny') {
        bumpKpi(deniedEl);
        const stage = STAGE_FROM_POLICY[p?.policy] ?? 'scope_check';
        flashStage(stage);
        const sev = p?.severity ?? 'deny';
        appendTicker(denialList, {
          time: formatTime(p?.created_at),
          msg: `${p?.user_id ?? '—'} → ${p?.tool_name ?? '?'}  policy=${p?.policy ?? ''}`,
          tag: sev === 'breach' ? 'BREACH' : 'DENY',
          tagClass: sev === 'breach' ? 'live-ticker__tag--breach' : 'live-ticker__tag--deny',
        }, 50);
      } else {
        bumpKpi(allowedEl);
      }
    },
  },
});
