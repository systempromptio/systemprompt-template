// /admin/overview — index. Subscribes to the multiplexed stream and
// increments per-pane summary metrics on the cards.

import { openStream, formatTime } from '/js/pages/admin-overview-shared.js';

const counters = {
  pulse: 0,
  identity: 0,
  cost: 0,
  governance: 0,
  services: 0,
};

function bump(pane) {
  counters[pane] += 1;
  const el = document.querySelector(`[data-pane-metric="${pane}"]`);
  if (!el) return;
  // Append " (+N live)" suffix once we start receiving events.
  el.dataset.live = counters[pane];
  const sub = document.querySelector(`[data-pane-sub="${pane}"]`);
  if (sub) sub.textContent = `+${counters[pane]} live · ${formatTime()}`;
}

openStream('/admin/api/sse/overview/index', {
  events: {
    request: () => { bump('pulse'); bump('identity'); bump('cost'); bump('services'); },
    decision: (p) => {
      bump('governance');
      bump('identity');
      if (p?.decision === 'deny') {
        const card = document.querySelector('[data-pane="governance"]');
        card?.classList.add('overview-card--warn');
      }
    },
    tool: () => { bump('pulse'); bump('services'); },
  },
});
