// The hover layer and the table view of a live chart. One crosshair finds the
// bucket; one tooltip lists every visible series at it, value first; the same
// readout follows keyboard focus. Labels and series names are untrusted text
// and only ever reach the DOM through textContent.

import { formatValue } from './sp-chart-scale.js';

const node = (tag, cls, content) => {
  const element = document.createElement(tag);
  if (cls) element.className = cls;
  if (content !== undefined) element.textContent = content;
  return element;
};

const fillTip = (tip, model, index) => {
  tip.replaceChildren(node('span', 'sp-chart__tip-label', model.labels[index] ?? `#${index + 1}`));
  for (const s of model.visible) {
    const row = node('span', 'sp-chart__tip-row');
    const key = node('i');
    key.style.setProperty('--sp-series', `var(${s.color})`);
    row.append(key, node('b', undefined, formatValue(s.values[index] ?? 0, model.unit, true)), node('span', undefined, s.label));
    tip.append(row);
  }
};

const placeTip = (tip, host, x) => {
  const half = tip.offsetWidth / 2;
  tip.style.left = `${Math.min(Math.max(x, half), host.clientWidth - half)}px`;
};

// Wires pointer and keyboard to the SVG the draw module produced. `geo()`
// is read on every event because a resize replaces the geometry.
export const attachHover = ({ host, svg, tip, model, geo }) => {
  let focused = -1;
  const show = (index) => {
    const g = geo();
    if (!g || index < 0 || index >= g.buckets) return;
    focused = index;
    const x = g.bucketX(index);
    const cursor = svg.querySelector('[data-chart-cursor]');
    if (cursor) {
      cursor.setAttribute('x1', x);
      cursor.setAttribute('x2', x);
      cursor.classList.add('is-visible');
    }
    for (const mark of svg.querySelectorAll('[data-bucket]')) {
      mark.classList.toggle('is-active', Number(mark.dataset.bucket) === index);
    }
    fillTip(tip, model, index);
    tip.hidden = false;
    placeTip(tip, host, (x / model.width) * host.clientWidth);
  };
  const hide = () => {
    svg.querySelector('[data-chart-cursor]')?.classList.remove('is-visible');
    for (const mark of svg.querySelectorAll('.is-active')) mark.classList.remove('is-active');
    tip.hidden = true;
  };
  svg.addEventListener('pointermove', (event) => {
    const hit = event.target.closest('[data-bucket]');
    if (hit) show(Number(hit.dataset.bucket));
  });
  svg.addEventListener('pointerleave', hide);
  svg.tabIndex = 0;
  svg.addEventListener('keydown', (event) => {
    const g = geo();
    if (!g) return;
    const step = { ArrowLeft: -1, ArrowRight: 1, Home: -g.buckets, End: g.buckets }[event.key];
    if (step === undefined) return;
    event.preventDefault();
    const start = focused < 0 ? g.buckets - 1 : focused;
    show(Math.min(Math.max(start + step, 0), g.buckets - 1));
  });
  svg.addEventListener('blur', hide);
  return { show, hide };
};

// The legend doubles as the series toggle; identity never rests on colour
// alone because the name sits beside every swatch.
export const buildLegend = (model, onToggle) => {
  if (model.series.length < 2) return null;
  const list = node('ul', 'sp-chart__legend');
  for (const s of model.series) {
    const item = node('li', 'sp-chart__legend-item');
    item.setAttribute('role', 'button');
    item.tabIndex = 0;
    item.setAttribute('aria-pressed', 'true');
    const swatch = node('i', model.kind === 'line' ? 'sp-chart__key sp-chart__key--line' : 'sp-chart__key');
    swatch.style.setProperty('--sp-series', `var(${s.color})`);
    item.append(swatch, node('span', 'sp-chart__legend-name', s.label), node('b', 'sp-chart__legend-value', s.total));
    const toggle = () => {
      s.hidden = !s.hidden;
      item.classList.toggle('is-hidden', s.hidden);
      item.setAttribute('aria-pressed', s.hidden ? 'false' : 'true');
      onToggle();
    };
    item.addEventListener('click', toggle);
    item.addEventListener('keydown', (event) => {
      if (event.key !== 'Enter' && event.key !== ' ') return;
      event.preventDefault();
      toggle();
    });
    list.append(item);
  }
  return list;
};

// Every value a chart shows is reachable without hovering: the table view
// is the same data, one row per bucket, one column per series.
export const buildTable = (model) => {
  const table = node('table', 'sp-chart__table');
  const head = node('thead');
  const headRow = node('tr');
  headRow.append(node('th', undefined, 'Bucket'));
  for (const s of model.series) headRow.append(node('th', 'sp-chart__table-num', s.label));
  head.append(headRow);
  const body = node('tbody');
  model.labels.forEach((label, i) => {
    const row = node('tr');
    row.append(node('th', undefined, label));
    for (const s of model.series) row.append(node('td', 'sp-chart__table-num', formatValue(s.values[i] ?? 0, model.unit, true)));
    body.append(row);
  });
  table.append(head, body);
  return table;
};
