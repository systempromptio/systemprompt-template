import { createSSEClient } from '../services/sse-client.js';
import { initTrafficChart, initCountryChart, initSparklines } from './admin-dashboard-charts.js';
import { rawFetch } from '../services/api.js';

const initTabs = () => {
  for (const tab of document.querySelectorAll('.dashboard-tabs .sp-tab')) {
    tab.addEventListener('click', () => {
      const target = tab.getAttribute('data-tab');

      if (target === 'report') {
        const url = new URL(window.location);
        url.searchParams.set('tab', 'report');
        window.location.href = url.toString();
        return;
      }

      for (const t of document.querySelectorAll('.dashboard-tabs .sp-tab')) {
        t.classList.remove('sp-tab--active');
        t.setAttribute('aria-selected', 'false');
      }
      for (const p of document.querySelectorAll('.sp-tab-panel')) p.hidden = true;
      tab.classList.add('sp-tab--active');
      tab.setAttribute('aria-selected', 'true');
      const panel = document.getElementById('tab-' + target);
      if (panel) panel.hidden = false;

      const url = new URL(window.location);
      url.searchParams.set('tab', target);
      history.replaceState(null, '', url);
    });
  }
};

const initReportTab = () => {
  const btn = document.getElementById('dashboard-report-regenerate');
  if (btn) {
    btn.addEventListener('click', async () => {
      btn.disabled = true;
      btn.querySelector('span').textContent = 'Generating...';
      try {
        await rawFetch('/admin/api/generate-traffic-report', { method: 'POST' });
        const url = new URL(window.location);
        url.searchParams.set('tab', 'report');
        window.location.href = url.toString();
      } catch {
        btn.querySelector('span').textContent = 'Error';
        setTimeout(() => { btn.querySelector('span').textContent = 'Regenerate'; btn.disabled = false; }, 2000);
      }
    });
  }

  const printBtn = document.getElementById('dashboard-report-print');
  if (printBtn) {
    printBtn.addEventListener('click', () => window.print());
  }

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

const initFilterAjax = () => {
  document.addEventListener('click', async (e) => {
    const link = e.target.closest('.traffic-period-selector a.chart-range-tab, .chart-range-tabs a.chart-range-tab, .content-period-selector a.chart-range-tab');
    if (!link || link.classList.contains('active')) return;

    e.preventDefault();
    const url = link.href;

    const isTrafficFilter = !!link.closest('.traffic-period-selector') || !!link.closest('.content-period-selector');
    const targetPanelId = isTrafficFilter ? 'tab-traffic' : 'tab-mcp';
    const targetPanel = document.getElementById(targetPanelId);

    if (targetPanel) targetPanel.classList.add('sp-tab-panel--loading');

    const siblingLinks = link.parentElement.querySelectorAll('a.chart-range-tab');
    siblingLinks.forEach(s => s.classList.remove('active'));
    link.classList.add('active');

    try {
      const resp = await fetch(url, { headers: { 'Accept': 'text/html' }, credentials: 'same-origin' });
      if (!resp.ok) throw new Error(resp.statusText);
      const html = await resp.text();
      const doc = new DOMParser().parseFromString(html, 'text/html');

      const newPanel = doc.getElementById(targetPanelId);
      if (newPanel && targetPanel) {
        targetPanel.innerHTML = newPanel.innerHTML;
      }

      if (isTrafficFilter) {
        initTrafficChart();
        initCountryChart();
      }

      history.replaceState(null, '', url);
    } catch (err) {
      window.location.href = url;
    } finally {
      if (targetPanel) targetPanel.classList.remove('sp-tab-panel--loading');
    }
  });
};

initTabs();
initTrafficChart();
initCountryChart();
initReportTab();
initFilterAjax();

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
