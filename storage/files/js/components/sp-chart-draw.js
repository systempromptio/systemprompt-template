// Drawing the live chart at pixel size: hairline gridlines, y and x tick
// labels in text tokens, and the marks — 2px lines with a 10% wash for a
// single series, or ≤24px columns (stacked for several series) with a 2px
// surface gap and rounded caps. Everything returns geometry the tooltip
// module reuses, so the crosshair and the marks agree on where a bucket is.

import { formatValue, thinLabels, yTicks } from './sp-chart-scale.js';

const NS = 'http://www.w3.org/2000/svg';
const MARGIN = { top: 10, right: 12, bottom: 24, left: 8 };
const MAX_BAR = 24;
const BAR_GAP = 2;

const el = (name, attrs = {}) => {
  const node = document.createElementNS(NS, name);
  for (const [key, value] of Object.entries(attrs)) node.setAttribute(key, String(value));
  return node;
};

const text = (x, y, content, cls, anchor = 'end') => {
  const node = el('text', { x, y, class: cls, 'text-anchor': anchor });
  node.textContent = content;
  return node;
};

const stacked = (kind, visible) => kind === 'stacked' || (kind === 'bars' && visible.length > 1);

// Columns stack, lines share one axis: the y top is the tallest stack or the
// tallest single value, never one axis per series.
const maxValue = (kind, visible, buckets) => {
  if (stacked(kind, visible)) {
    let max = 0;
    for (let i = 0; i < buckets; i += 1) {
      max = Math.max(max, visible.reduce((acc, s) => acc + (s.values[i] ?? 0), 0));
    }
    return max;
  }
  return Math.max(0, ...visible.flatMap((s) => s.values));
};

const measureLeft = (ticks, unit) => {
  const longest = Math.max(...ticks.map((t) => formatValue(t, unit).length), 1);
  return MARGIN.left + longest * 7 + 8;
};

export const layout = ({ kind, visible, labels, unit, width, height }) => {
  const buckets = Math.max(labels.length, ...visible.map((s) => s.values.length), 1);
  const { top, ticks } = yTicks(maxValue(kind, visible, buckets));
  const left = measureLeft(ticks, unit);
  const plotW = Math.max(width - left - MARGIN.right, 10);
  const plotH = Math.max(height - MARGIN.top - MARGIN.bottom, 10);
  const slot = plotW / buckets;
  return {
    buckets,
    top,
    ticks,
    left,
    plotW,
    plotH,
    slot,
    bucketX: (i) => left + (i + 0.5) * slot,
    yFor: (v) => MARGIN.top + plotH * (1 - v / top),
    baseline: MARGIN.top + plotH,
  };
};

const drawAxes = (svg, geo, labels, unit) => {
  for (const tick of geo.ticks) {
    const y = geo.yFor(tick);
    svg.append(el('line', {
      x1: geo.left, x2: geo.left + geo.plotW, y1: y, y2: y,
      class: tick === 0 ? 'sp-chart__baseline' : 'sp-chart__grid',
    }));
    svg.append(text(geo.left - 6, y + 3.5, formatValue(tick, unit), 'sp-chart__tick'));
  }
  const shown = thinLabels(labels, geo.plotW);
  shown.forEach((label, i) => {
    if (!label) return;
    const anchor = i === 0 ? 'start' : i === shown.length - 1 ? 'end' : 'middle';
    const x = i === 0 ? geo.left : i === shown.length - 1 ? geo.left + geo.plotW : geo.bucketX(i);
    svg.append(text(x, geo.baseline + 16, label, 'sp-chart__tick', anchor));
  });
};

const linePath = (points) => points.map(([x, y], i) => `${i === 0 ? 'M' : 'L'}${x.toFixed(1)},${y.toFixed(1)}`).join(' ');

const drawLines = (svg, geo, visible) => {
  const single = visible.length === 1;
  for (const s of visible) {
    const points = s.values.map((v, i) => [geo.bucketX(i), geo.yFor(v)]);
    const group = el('g', { class: 'sp-chart__series', 'data-series': s.index });
    if (single && points.length >= 3) {
      const first = points[0][0];
      const last = points.at(-1)[0];
      group.append(el('path', {
        d: `${linePath(points)} L${last.toFixed(1)},${geo.baseline} L${first.toFixed(1)},${geo.baseline} Z`,
        class: 'sp-chart__area', fill: `var(${s.color})`,
      }));
    }
    group.append(el('path', { d: linePath(points), class: 'sp-chart__line', stroke: `var(${s.color})` }));
    const showDots = points.length <= 16;
    points.forEach(([x, y], i) => {
      group.append(el('circle', {
        cx: x, cy: y, r: showDots ? 4 : 3.5, fill: `var(${s.color})`,
        class: `sp-chart__dot${showDots ? '' : ' sp-chart__dot--hover'}`, 'data-bucket': i,
      }));
    });
    svg.append(group);
  }
};

const drawColumns = (svg, geo, visible) => {
  const width = Math.min(MAX_BAR, Math.max(geo.slot - 6, 2));
  const stacks = new Array(geo.buckets).fill(0);
  for (const s of visible) {
    const group = el('g', { class: 'sp-chart__series', 'data-series': s.index });
    s.values.forEach((v, i) => {
      if (v <= 0) return;
      const bottom = geo.yFor(stacks[i]);
      const topY = geo.yFor(stacks[i] + v);
      const h = Math.max(bottom - topY - (stacks[i] > 0 ? BAR_GAP : 0), 1);
      group.append(el('rect', {
        x: geo.bucketX(i) - width / 2, y: bottom - h - (stacks[i] > 0 ? BAR_GAP : 0), width, height: h,
        rx: stacks[i] === 0 && visible.length === 1 ? 4 : 0, fill: `var(${s.color})`,
        class: 'sp-chart__bar', 'data-bucket': i,
      }));
      stacks[i] += v;
    });
    svg.append(group);
  }
};

// Redraws the whole SVG for the current size and the visible series and
// returns the geometry the tooltip needs. Hit bands are transparent rects
// per bucket, so the reader aims at a date rather than a 2px line.
export const render = (svg, model) => {
  svg.replaceChildren();
  const geo = layout(model);
  svg.setAttribute('viewBox', `0 0 ${model.width} ${model.height}`);
  drawAxes(svg, geo, model.labels, model.unit);
  if (model.kind === 'bars' || model.kind === 'stacked') drawColumns(svg, geo, model.visible);
  else drawLines(svg, geo, model.visible);
  svg.append(el('line', {
    x1: 0, x2: 0, y1: MARGIN.top, y2: geo.baseline, class: 'sp-chart__cursor', 'data-chart-cursor': '',
  }));
  const hits = el('g', { class: 'sp-chart__hits' });
  for (let i = 0; i < geo.buckets; i += 1) {
    hits.append(el('rect', {
      x: geo.left + i * geo.slot, y: MARGIN.top, width: Math.max(geo.slot, 24), height: geo.plotH,
      class: 'sp-chart__hit', 'data-bucket': i,
    }));
  }
  svg.append(hits);
  return geo;
};
