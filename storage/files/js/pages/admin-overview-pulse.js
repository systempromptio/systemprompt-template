// /admin/overview/pulse
import {
  openStream, appendTicker, updateTickerRow, initTickerExpand,
  formatTime, bumpKpi, Sparkline, setStatus,
} from '/js/pages/admin-overview-shared.js';

const url    = document.currentScript?.dataset.sseUrl ?? '/admin/api/sse/overview/pulse';
const ticker = document.getElementById('pulse-ticker');
const status = document.getElementById('pulse-status');
const reqEl  = document.querySelector('[data-kpi="requests"]');
const errEl  = document.querySelector('[data-kpi="error_count"]');
const spark  = new Sparkline(document.getElementById('pulse-spark'), 60);

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
  onOpen:  () => setStatus(status, 'live'),
  onError: () => setStatus(status, 'reconnecting…'),
  onLag:   () => setStatus(status, 'catching up…'),
  events: {
    request: (p) => {
      bumpKpi(reqEl);
      const isError   = p?.severity === 'error';
      const isPending = p?.status === 'pending';
      spark.add(isError);
      if (isError) bumpKpi(errEl);
      const tid = p?.trace_id;
      appendTicker(ticker, {
        time:         formatTime(p?.created_at),
        actor:        actor(p),
        tag:          isError ? 'ERROR' : isPending ? 'PENDING' : 'OK',
        tagClass:     isError ? 'live-ticker__tag--deny' : isPending ? 'live-ticker__tag--pending' : '',
        traceId:      tid,
        userId:       p?.user_id,
        sessionId:    p?.session_id,
        contextId:    p?.context_id,
        requestId:    p?.id,
        displayName:  p?.display_name,
        department:   p?.department,
        errorMessage: isError ? p?.error_message : null,
        errorHref:    isError && tid ? `/admin/analytics/requests?q=${encodeURIComponent(tid)}&preset=7d` : null,
      }, 100);
    },
    decision: (p) => {
      const sev    = p?.severity ?? 'info';
      const isDeny = p?.decision === 'deny';
      const tag    = isDeny ? (sev === 'breach' ? 'BREACH' : 'DENY') : 'ALLOW';
      const tagClass = isDeny
        ? (sev === 'breach' ? 'live-ticker__tag--breach' : 'live-ticker__tag--deny')
        : '';
      // governance_decisions.session_id stores the request trace_id (authz.rs:79)
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
      }, 100);
    },
    tool: (p) => {
      spark.add(false);
      appendTicker(ticker, {
        time:   formatTime(p?.created_at),
        actor:  actor(p),
        target: p?.tool_name ?? '?',
        tag:    'TOOL',
        tagClass: 'live-ticker__tag--tool',
        userId: p?.user_id,
      }, 100);
    },
  },
});
