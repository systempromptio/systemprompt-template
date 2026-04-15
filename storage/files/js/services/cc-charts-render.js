import { setupTooltip } from './cc-charts-tooltip.js';

const formatBytes = (bytes) => {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1048576) return (bytes / 1024).toFixed(1) + ' KB';
  return (bytes / 1048576).toFixed(1) + ' MB';
};

let _data24h = [];
let _data7d = [];
let _data30d = [];
let _activeRange = '24h';

export const getActiveRange = () => _activeRange;
export const setActiveRange = (r) => { _activeRange = r; };

const CHART_W = 960;
const CHART_H = 280;
const PADDING_TOP = 20;
const PADDING_BOTTOM = 5;
const DRAW_H = CHART_H - PADDING_TOP - PADDING_BOTTOM;

export const SERIES = [
  { key: 'actions', color: '#8B5CF6', grad: 'cc-grad-actions' },
  { key: 'apm', color: '#10B981', grad: 'cc-grad-apm' },
  { key: 'throughput', color: '#3B82F6', grad: 'cc-grad-throughput' },
  { key: 'concurrency', color: '#6B68FA', grad: 'cc-grad-concurrency' },
  { key: 'tools', color: '#EF4444', grad: 'cc-grad-tools' },
];

export const extractMetric = (b, key) => {
  switch (key) {
    case 'actions': return b.actions || 0;
    case 'apm': return b.sessions > 0 ? (b.actions || 0) / Math.max(b.sessions, 1) : 0;
    case 'throughput': return (b.input_bytes || 0) + (b.output_bytes || 0);
    case 'concurrency': return b.sessions || 0;
    case 'tools': return b.unique_tools || 0;
    default: return 0;
  }
};

export const formatChartValue = (val, metric) => {
  if (metric === 'throughput') return formatBytes(val);
  if (metric === 'apm') return val.toFixed(1) + ' APM';
  return Math.round(val).toString();
};

export { CHART_W, CHART_H, PADDING_BOTTOM, DRAW_H };
const getActiveData = () => {
  if (_activeRange === '7d') return _data7d;
  if (_activeRange === '30d') return _data30d;
  return _data24h;
};
const buildPath = (points, close) => {
  if (!points.length) return '';
  let d = 'M' + points[0][0].toFixed(1) + ',' + points[0][1].toFixed(1);
  for (let i = 1; i < points.length; i++) {
    d += ' L' + points[i][0].toFixed(1) + ',' + points[i][1].toFixed(1);
  }
  if (close) {
    d += ' L' + points[points.length - 1][0].toFixed(1) + ',' + CHART_H;
    d += ' L' + points[0][0].toFixed(1) + ',' + CHART_H + ' Z';
  }
  return d;
};

const DEFS_HTML = '<defs>' +
  SERIES.map((s) =>
    '<linearGradient id="' + s.grad + '" x1="0" y1="0" x2="0" y2="1">' +
    '<stop offset="0%" stop-color="' + s.color + '" stop-opacity="0.45"/>' +
    '<stop offset="100%" stop-color="' + s.color + '" stop-opacity="0.03"/>' +
    '</linearGradient>'
  ).join('') +
  '<filter id="cc-glow"><feGaussianBlur stdDeviation="3" result="blur"/>' +
  '<feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge></filter>' +
  '</defs>';

const renderXLabels = (data) => {
  const labelsEl = document.getElementById('cc-hourly-x-labels');
  if (!labelsEl) return;
  labelsEl.innerHTML = '';
  const n = data.length;
  if (_activeRange === '24h') {
    for (const h of [0, 3, 6, 9, 12, 15, 18, 21]) {
      const span = document.createElement('span');
      span.textContent = h;
      span.style.left = (h / Math.max(n - 1, 1)) * 100 + '%';
      labelsEl.append(span);
    }
  } else {
    const step = _activeRange === '7d' ? 1 : 5;
    const days = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
    for (let i = 0; i < n; i += step) {
      const b = data[i];
      const span = document.createElement('span');
      if (_activeRange === '7d' && b.date) {
        const d = new Date(b.date + 'T00:00:00');
        span.textContent = days[d.getDay()];
      } else if (b.date) {
        span.textContent = b.date.slice(5);
      }
      span.style.left = (i / Math.max(n - 1, 1)) * 100 + '%';
      labelsEl.append(span);
    }
  }
};

export const renderChart = () => {
  const svg = document.getElementById('cc-hourly-svg');
  const data = getActiveData();
  if (!svg || !data.length) return;
  svg.setAttribute('viewBox', '0 0 ' + CHART_W + ' ' + CHART_H);
  svg.setAttribute('preserveAspectRatio', 'none');
  const n = data.length;
  let html = DEFS_HTML;
  for (const frac of [0.25, 0.5, 0.75]) {
    const gy = CHART_H - PADDING_BOTTOM - DRAW_H * frac;
    html += '<line x1="0" y1="' + gy.toFixed(1) + '" x2="' + CHART_W + '" y2="' + gy.toFixed(1) +
      '" stroke="var(--sp-border-subtle, rgba(255,255,255,0.06))" stroke-dasharray="4 3" stroke-width="0.5" />';
  }
  const delays = [0, 0.08, 0.16, 0.24, 0.32];
  for (let si = 0; si < SERIES.length; si++) {
    const s = SERIES[si];
    const values = data.map((b) => extractMetric(b, s.key));
    const maxVal = Math.max(...values, 0.001);
    const points = values.map((v, i) => {
      const x = n === 1 ? CHART_W / 2 : (i / (n - 1)) * CHART_W;
      const y = CHART_H - PADDING_BOTTOM - (v / maxVal) * DRAW_H;
      return [x, y];
    });
    const areaD = buildPath(points, true);
    const lineD = buildPath(points, false);
    html += '<path d="' + areaD + '" fill="url(#' + s.grad + ')" class="cc-area-path" style="animation-delay:' + delays[si] + 's" />';
    html += '<path d="' + lineD + '" fill="none" stroke="' + s.color + '" stroke-width="2" class="cc-line-path" style="animation-delay:' + (0.4 + delays[si]) + 's" />';
  }
  html += '<line id="cc-guide-line" x1="0" y1="0" x2="0" y2="' + CHART_H + '" class="cc-chart-guide" style="display:none" />';
  svg.innerHTML = html;
  setupTooltip(svg, data);
  renderXLabels(data);
};

export const updateHourlyCharts = (data) => {
  if (Array.isArray(data)) { _data24h = data; if (_activeRange === '24h') renderChart(); }
};
export const update7dData = (data) => {
  if (Array.isArray(data)) { _data7d = data; if (_activeRange === '7d') renderChart(); }
};
export const update30dData = (data) => {
  if (Array.isArray(data)) { _data30d = data; if (_activeRange === '30d') renderChart(); }
};
