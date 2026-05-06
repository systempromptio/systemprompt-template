// /admin/overview/services
import {
  openStream, appendTicker, formatTime, bumpKpi,
} from '/js/pages/admin-overview-shared.js';

const url = document.currentScript?.dataset.sseUrl ?? '/admin/api/sse/overview/services';
const ticker = document.getElementById('services-ticker');
const errEl = document.querySelector('[data-kpi="errors"]');

openStream(url, {
  events: {
    request: (p) => {
      if (p?.severity === 'error') bumpKpi(errEl);
      appendTicker(ticker, {
        time: formatTime(p?.created_at),
        msg: `req: ${p?.model ?? '?'}  status=${p?.status ?? ''}`,
        tag: p?.severity === 'error' ? 'ERROR' : 'REQ',
        tagClass: p?.severity === 'error' ? 'live-ticker__tag--deny' : '',
      }, 50);
    },
    tool: (p) => {
      appendTicker(ticker, {
        time: formatTime(p?.created_at),
        msg: `tool: ${p?.tool_name ?? '?'} · ${p?.event_type ?? ''}`,
        tag: 'TOOL',
        tagClass: 'live-ticker__tag--tool',
      }, 50);
    },
  },
});
