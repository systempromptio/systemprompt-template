// Scales and labels for the live charts: a "nice" tick ladder for the y
// axis, unit-aware number formatting, and x-label thinning so dates never
// collide. Pure functions — no DOM — so the draw module stays about geometry.

const LADDER = [1, 2, 2.5, 5, 10];

// The smallest step from the 1/2/2.5/5 ladder that fits the value into the
// requested number of ticks, so gridlines land on round numbers.
export const niceStep = (max, ticks) => {
  if (max <= 0) return 1;
  const rough = max / ticks;
  const power = 10 ** Math.floor(Math.log10(rough));
  const unit = LADDER.find((step) => step * power >= rough) ?? 10;
  return unit * power;
};

export const yTicks = (max, count = 4, integer = true) => {
  if (max <= 0) return { top: 1, ticks: [0, 1] };
  const step = Math.max(niceStep(max, count), integer ? 1 : 0);
  const top = Math.ceil(max / step) * step;
  const ticks = [];
  for (let v = 0; v <= top + step / 2; v += step) ticks.push(Number(v.toFixed(6)));
  return { top, ticks };
};

const compact = (value) => {
  const abs = Math.abs(value);
  if (abs >= 1_000_000) return `${(value / 1_000_000).toFixed(abs >= 10_000_000 ? 0 : 1)}M`;
  if (abs >= 1_000) return `${(value / 1_000).toFixed(abs >= 10_000 ? 0 : 1)}k`;
  return Number.isInteger(value) ? value.toLocaleString() : value.toFixed(1);
};

// Axis ticks are compact ("1.2k"); tooltip values are exact ("1,234").
export const formatValue = (value, unit, exact = false) => {
  if (unit === 'µ$') {
    const dollars = value / 1_000_000;
    return `$${dollars.toFixed(dollars >= 1 ? 2 : 4)}`;
  }
  if (unit === 'ms') return value >= 1000 ? `${(value / 1000).toFixed(1)}s` : `${Math.round(value)}ms`;
  const text = exact ? value.toLocaleString() : compact(value);
  return unit ? `${text} ${unit}` : text;
};

// Keeps every n-th label so neighbours sit at least `minGap` pixels apart,
// always keeping the first and the last.
export const thinLabels = (labels, width, minGap = 56) => {
  const n = labels.length;
  if (n === 0) return [];
  const slot = width / n;
  const every = Math.max(1, Math.ceil(minGap / slot));
  return labels.map((label, i) => {
    if (i === n - 1) return label;
    const keep = i % every === 0 && (n - 1 - i) * slot >= minGap;
    return keep ? label : '';
  });
};

export const sum = (values) => values.reduce((acc, v) => acc + v, 0);
