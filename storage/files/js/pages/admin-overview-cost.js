// /admin/overview/cost
import {
  openStream, appendTicker, formatTime, bumpKpi,
} from '/js/pages/admin-overview-shared.js';

const url = document.currentScript?.dataset.sseUrl ?? '/admin/api/sse/overview/cost';
const ticker = document.getElementById('cost-ticker');
const errEl = document.querySelector('[data-kpi="errors"]');

openStream(url, {
  events: {
    request: (p) => {
      if (p?.severity === 'error') bumpKpi(errEl);
      appendTicker(ticker, {
        time: formatTime(p?.created_at),
        msg: `${p?.model ?? '?'}  user=${p?.user_id ?? '—'}`,
        tag: p?.severity === 'error' ? 'ERROR' : 'REQ',
        tagClass: p?.severity === 'error' ? 'live-ticker__tag--deny' : '',
      }, 50);
    },
  },
});
