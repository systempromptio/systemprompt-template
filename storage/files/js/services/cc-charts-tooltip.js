import {
  SERIES, extractMetric, formatChartValue,
  CHART_W, getActiveRange,
} from './cc-charts-render.js';

export const setupTooltip = (svg, data) => {
  const container = svg.closest('.cc-hourly-chart-container');
  if (!container) return;

  let tooltip = container.querySelector('.cc-chart-tooltip-multi');
  if (!tooltip) {
    tooltip = document.createElement('div');
    tooltip.className = 'cc-chart-tooltip-multi';
    tooltip.hidden = true;
    container.style.position = 'relative';
    container.append(tooltip);
  }

  const n = data.length;
  const guide = svg.getElementById('cc-guide-line');

  svg.addEventListener('mousemove', (e) => {
    const rect = svg.getBoundingClientRect();
    const relX = (e.clientX - rect.left) / rect.width * CHART_W;
    const idx = Math.round(relX / (CHART_W / Math.max(n - 1, 1)));
    const clamped = Math.max(0, Math.min(idx, n - 1));
    const b = data[clamped];

    const pointX = n === 1 ? CHART_W / 2 : (clamped / (n - 1)) * CHART_W;
    if (guide) {
      guide.setAttribute('x1', pointX.toFixed(1));
      guide.setAttribute('x2', pointX.toFixed(1));
      guide.style.display = '';
    }

    const activeRange = getActiveRange();
    const label = activeRange === '24h' ? (b.hour + ':00') : (b.date || '');
    let tipHtml = '<strong>' + label + '</strong>';
    for (const s of SERIES) {
      const val = extractMetric(b, s.key);
      const display = formatChartValue(val, s.key);
      tipHtml += '<div><span style="color:' + s.color + '">\u25CF</span> ' +
        s.key.charAt(0).toUpperCase() + s.key.slice(1) + ': ' + display + '</div>';
    }
    tooltip.innerHTML = tipHtml;
    tooltip.hidden = false;

    const tipX = (e.clientX - container.getBoundingClientRect().left) + 12;
    const tipY = (e.clientY - container.getBoundingClientRect().top) - 20;
    const maxX = container.offsetWidth - tooltip.offsetWidth - 8;
    tooltip.style.left = Math.min(tipX, maxX) + 'px';
    tooltip.style.top = Math.max(tipY, 0) + 'px';
  });

  svg.addEventListener('mouseleave', () => {
    tooltip.hidden = true;
    if (guide) guide.style.display = 'none';
  });
};
