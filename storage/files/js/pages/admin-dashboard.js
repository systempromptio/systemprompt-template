import { createSSEClient } from '../services/sse-client.js';
import { initTrafficChart, initCountryChart, initSparklines } from './admin-dashboard-charts.js';
import { rawFetch } from '../services/api.js';

const initReport = () => {
  const btn = document.getElementById('dashboard-report-regenerate');
  if (btn) {
    btn.addEventListener('click', async () => {
      btn.disabled = true;
      const lbl = btn.querySelector('span');
      if (lbl) lbl.textContent = 'Generating...';
      try {
        await rawFetch('/admin/api/generate-traffic-report', { method: 'POST' });
        window.location.reload();
      } catch {
        if (lbl) lbl.textContent = 'Error';
        setTimeout(() => {
          if (lbl) lbl.textContent = 'Regenerate';
          btn.disabled = false;
        }, 2000);
      }
    });
  }

  const printBtn = document.getElementById('dashboard-report-print');
  if (printBtn) printBtn.addEventListener('click', () => window.print());

  const countdownEl = document.getElementById('dashboard-report-countdown-value');
  if (countdownEl) {
    const updateCountdown = () => {
      const now = new Date();
      const utcH = now.getUTCHours();
      const utcM = now.getUTCMinutes();
      let nextH = utcH < 8 ? 8 : utcH < 20 ? 20 : 32;
      let diffMin = (nextH * 60 - (utcH * 60 + utcM));
      if (diffMin <= 0) diffMin += 24 * 60;
      const h = Math.floor(diffMin / 60);
      const m = diffMin % 60;
      countdownEl.textContent = h + 'h ' + m + 'm';
    };
    updateCountdown();
    setInterval(updateCountdown, 60000);
  }

  initSparklines();
};

const initWindowPills = () => {
  const pills = document.querySelectorAll('.window-pill[data-window]');
  if (!pills.length) return;
  for (const pill of pills) {
    pill.addEventListener('click', () => {
      const win = pill.getAttribute('data-window');
      for (const p of pills) {
        const active = p === pill;
        p.classList.toggle('window-pill--active', active);
        p.setAttribute('aria-selected', active ? 'true' : 'false');
      }
      for (const panel of document.querySelectorAll('[data-window-panel]')) {
        panel.hidden = panel.getAttribute('data-window-panel') !== win;
      }
    });
  }
};

initWindowPills();
initTrafficChart();
initCountryChart();
initReport();

if (document.getElementById('sse-indicator')) {
  createSSEClient('/admin/api/sse/dashboard', {
    onConnect: () => document.getElementById('sse-indicator')?.classList.add('live-dot-connected'),
    onDisconnect: () => document.getElementById('sse-indicator')?.classList.remove('live-dot-connected'),
    events: {
      activity: (events) => {
        const feed = document.getElementById('live-feed');
        if (feed) {
          for (const evt of events) {
            const item = document.createElement('div');
            item.className = 'feed-item';
            item.setAttribute('data-ts', evt.created_at || '');
            const catMap = {
              login: 'feed-blue', logout: 'feed-cyan',
              marketplace_connect: 'feed-purple', marketplace_edit: 'feed-green',
              session: 'feed-indigo', session_rated: 'feed-gold',
              user_management: 'feed-teal',
            };
            const catClass = catMap[evt.category] || 'feed-orange';
            const name = evt.display_name || 'Anonymous';
            const desc = evt.description || evt.category || '';
            const iconDiv = document.createElement('div');
            iconDiv.className = 'feed-icon ' + catClass;
            const contentDiv = document.createElement('div');
            contentDiv.className = 'feed-content';
            const textSpan = document.createElement('span');
            textSpan.className = 'feed-text';
            const strong = document.createElement('strong');
            strong.textContent = name;
            textSpan.append(strong);
            textSpan.append(document.createTextNode(' ' + desc));
            const timeSpan = document.createElement('span');
            timeSpan.className = 'feed-time';
            timeSpan.textContent = 'just now';
            contentDiv.append(textSpan);
            contentDiv.append(timeSpan);
            item.append(iconDiv);
            item.append(contentDiv);
            feed.insertBefore(item, feed.firstChild);
          }
        }
      },
      stats: (stats) => {
        const keyMap = {
          events_today: stats.events_today,
          active_users_24h: stats.active_users_24h,
          page_views: stats.page_views,
        };
        for (const [key, val] of Object.entries(keyMap)) {
          const el = document.querySelector('[data-key="' + key + '"] .profile-hero__stat-value');
          if (el && val !== undefined) el.textContent = val;
        }
      },
    },
  });
}
