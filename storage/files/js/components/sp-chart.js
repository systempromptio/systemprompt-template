// Live charts. The server renders every `[data-chart]` figure complete — a
// stretched unit-space SVG with three-label gutters — so the page reads
// without scripts. This module replaces that plot with one drawn at real
// pixel size (proper axes, hairline grid, round dots, columns) and keeps it
// in step with the panel through a ResizeObserver; hover, keyboard, legend
// toggles and a table view come from the sibling modules. Series values are
// read from the server's `data-values`, never re-derived from geometry.

import { render } from './sp-chart-draw.js';
import { formatValue, sum } from './sp-chart-scale.js';
import { attachHover, buildLegend, buildTable } from './sp-chart-tooltip.js';

const parseValues = (raw) => raw.split(',').map((v) => Number.parseInt(v, 10) || 0);

const parseLabels = (raw) => {
  try {
    const labels = JSON.parse(raw || '[]');
    return Array.isArray(labels) ? labels.map(String) : [];
  } catch {
    return [];
  }
};

const readModel = (figure) => {
  const unit = figure.dataset.unit ?? '';
  const series = [...figure.querySelectorAll('[data-series]')].map((group, index) => {
    const values = parseValues(group.dataset.values ?? '');
    return {
      index,
      label: group.dataset.label ?? `Series ${index + 1}`,
      color: group.dataset.color ?? '--sp-chart-purple',
      values,
      // The server states each series' legend figure (a sum, a peak or a
      // percentile — only it knows which); summing here is the fallback.
      total: group.dataset.total || formatValue(sum(values), unit, true),
      hidden: false,
    };
  });
  if (series.length === 0) return null;
  return {
    kind: figure.dataset.kind ?? 'line',
    unit,
    labels: parseLabels(figure.dataset.labels),
    series,
    get visible() {
      return this.series.filter((s) => !s.hidden);
    },
    width: 0,
    height: 0,
  };
};

const button = (label, pressed) => {
  const el = document.createElement('button');
  el.type = 'button';
  el.className = 'sp-btn sp-btn--xs sp-btn--ghost sp-chart__tool';
  el.textContent = label;
  el.setAttribute('aria-pressed', pressed ? 'true' : 'false');
  return el;
};

export const initChart = (figure) => {
  const model = readModel(figure);
  if (!model) return null;
  for (const stale of figure.querySelectorAll('.sp-timeseries__plot, .sp-timeseries__x-axis, .sp-svgchart__legend, .sp-svgchart__refs')) {
    stale.hidden = true;
  }
  const host = document.createElement('div');
  host.className = 'sp-chart';
  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.setAttribute('class', 'sp-chart__svg');
  svg.setAttribute('role', 'img');
  svg.setAttribute('aria-label', figure.querySelector('[data-chart-plot]')?.getAttribute('aria-label') ?? '');
  const tip = document.createElement('div');
  tip.className = 'sp-chart__tip';
  tip.hidden = true;
  host.append(svg, tip);
  figure.append(host);

  let geo = null;
  const draw = () => {
    model.width = Math.max(host.clientWidth, 120);
    model.height = Math.max(host.clientHeight, 120);
    geo = model.visible.length ? render(svg, model) : null;
    if (!geo) svg.replaceChildren();
  };
  const legend = buildLegend(model, draw);
  if (legend) figure.append(legend);

  const table = buildTable(model);
  table.hidden = true;
  figure.append(table);
  const head = figure.querySelector('.sp-timeseries__head');
  if (head) {
    const tools = document.createElement('span');
    tools.className = 'sp-chart__tools';
    const toggle = button('Table', false);
    toggle.addEventListener('click', () => {
      const showing = table.hidden;
      table.hidden = !showing;
      host.hidden = showing;
      if (legend) legend.hidden = showing;
      toggle.setAttribute('aria-pressed', showing ? 'true' : 'false');
      if (!showing) draw();
    });
    tools.append(toggle);
    head.append(tools);
  }

  attachHover({ host, svg, tip, model, geo: () => geo });
  if (typeof ResizeObserver === 'function') {
    new ResizeObserver(() => {
      if (!host.hidden) draw();
    }).observe(host);
  }
  draw();
  figure.classList.add('sp-svgchart--live');
  return { draw, model };
};

export const initAllCharts = (scope = document) =>
  [...scope.querySelectorAll('[data-chart]')].map(initChart).filter(Boolean);

if (typeof document !== 'undefined') initAllCharts();
