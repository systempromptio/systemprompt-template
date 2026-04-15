import { SERIES, setActiveRange } from './cc-charts-render.js';

const ensureChartHeader = () => {
  const card = document.getElementById('cc-hourly-charts');
  if (!card) return;

  const oldTabs = card.querySelector('.cc-hourly-chart-tabs');
  if (oldTabs) oldTabs.remove();

  const oldMaxLabel = card.querySelector('.cc-chart-max-label');
  if (oldMaxLabel) oldMaxLabel.remove();

  if (!card.querySelector('.cc-chart-header')) {
    const header = document.createElement('div');
    header.className = 'cc-chart-header';
    header.innerHTML =
      '<span class="cc-chart-title">Activity</span>' +
      '<div class="cc-range-tabs">' +
        '<button type="button" class="cc-range-tab cc-range-tab--active" data-range="24h">24h</button>' +
        '<button type="button" class="cc-range-tab" data-range="7d">7d</button>' +
        '<button type="button" class="cc-range-tab" data-range="30d">30d</button>' +
      '</div>' +
      '<div class="cc-chart-legend">' +
        SERIES.map((s) =>
          '<span class="cc-legend-item" style="display:flex;align-items:center;gap:4px;font-size:0.75rem;color:var(--sp-text-secondary)">' +
          '<span style="width:10px;height:10px;border-radius:2px;background:' + s.color + '"></span>' +
          s.key.charAt(0).toUpperCase() + s.key.slice(1) +
          '</span>'
        ).join('') +
      '</div>';
    card.insertBefore(header, card.firstChild);
  }
};

export const initChartSetup = (renderChart) => {
  const initRangeTabs = () => {
    ensureChartHeader();
    const container = document.querySelector('.cc-range-tabs');
    if (container && !container.dataset.init) {
      container.dataset.init = '1';
      container.addEventListener('click', (e) => {
        const tab = e.target.closest('.cc-range-tab');
        if (tab) {
          for (const t of container.querySelectorAll('.cc-range-tab')) t.classList.remove('cc-range-tab--active');
          tab.classList.add('cc-range-tab--active');
          setActiveRange(tab.dataset.range);
          renderChart();
        }
      });
    }
  };

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initRangeTabs);
  } else {
    initRangeTabs();
  }

  const initRegenerateReport = () => {
    const btn = document.getElementById('cc-report-regenerate');
    if (!btn || btn.dataset.init) return;
    btn.dataset.init = '1';
    btn.addEventListener('click', async () => {
      btn.disabled = true;
      const orig = btn.innerHTML;
      btn.innerHTML = '<span>Generating\u2026</span>';
      try {
        const r = await fetch('/control-center/api/generate-report', { method: 'POST', headers: { 'Content-Type': 'application/json' } });
        if (r.status === 403) { const j = await r.json(); throw new Error(j.message || 'Feature not available on your plan'); }
        if (r.status === 503) throw new Error('AI service unavailable');
        if (!r.ok) { const j = await r.json(); throw new Error(j.message || 'Generation failed'); }
        await r.json();
        window.location.reload();
      } catch (err) {
        btn.disabled = false;
        btn.innerHTML = orig;
        const s = btn.querySelector('span:last-child');
        if (s) { s.textContent = err.message; setTimeout(() => { s.textContent = 'Regenerate'; }, 3000); }
      }
    });
  };
  initRegenerateReport();
};
