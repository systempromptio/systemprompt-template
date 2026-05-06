// /admin/overview/identity
import {
  openStream, appendTicker, updateTickerRow, initTickerExpand, formatTime, bumpKpi,
} from '/js/pages/admin-overview-shared.js';

const url    = document.currentScript?.dataset.sseUrl ?? '/admin/api/sse/overview/identity';
const ticker = document.getElementById('identity-ticker');
const reqEl  = document.querySelector('[data-kpi="requests"]');
const errEl  = document.querySelector('[data-kpi="errors"]');

initTickerExpand(ticker);

function actor(p) {
  const dept = p?.department;
  const name = p?.display_name;
  const uid  = p?.user_id ?? '—';
  if (dept && dept.length) return dept;
  if (name && name.length) return name;
  return uid.length > 8 ? uid.slice(0, 8) + '…' : uid;
}

openStream(url, {
  events: {
    request: (p) => {
      bumpKpi(reqEl);
      const isError = p?.severity === 'error';
      if (isError) bumpKpi(errEl);
      appendTicker(ticker, {
        time:        formatTime(p?.created_at),
        actor:       actor(p),
        target:      p?.model ?? '?',
        tag:         isError ? 'ERROR' : 'REQ',
        tagClass:    isError ? 'live-ticker__tag--deny' : '',
        traceId:     p?.trace_id,
        userId:      p?.user_id,
        sessionId:   p?.session_id,
        requestId:   p?.id,
        displayName: p?.display_name,
        department:  p?.department,
      }, 50);
    },
    decision: (p) => {
      const isDeny   = p?.decision === 'deny';
      const tag      = isDeny ? 'DENY' : 'ALLOW';
      const tagClass = isDeny ? 'live-ticker__tag--deny' : '';
      if (isDeny && updateTickerRow(ticker, p?.session_id, { tag, tagClass })) return;
      appendTicker(ticker, {
        time:        formatTime(p?.created_at),
        actor:       actor(p),
        target:      p?.tool_name ?? '?',
        tag,
        tagClass,
        traceId:     p?.session_id,
        userId:      p?.user_id,
        requestId:   p?.id,
        displayName: p?.display_name,
        department:  p?.department,
        policy:      p?.policy,
        reason:      p?.reason,
      }, 50);
    },
  },
});
