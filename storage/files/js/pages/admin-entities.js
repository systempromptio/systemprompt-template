const sizeBars = (selector, dataAttr, maxAttr) => {
  for (const container of document.querySelectorAll(selector)) {
    const max = parseFloat(container.getAttribute(maxAttr)) || 0;
    for (const bar of container.querySelectorAll(`[${dataAttr}]`)) {
      const value = parseFloat(bar.getAttribute(dataAttr)) || 0;
      bar.style.height = `${max > 0 ? Math.max(2, (value / max) * 100) : 0}%`;
    }
  }
};

const buildRow = (span, t0, total) => {
  const start = new Date(span.started_at).getTime();
  const end = new Date(span.ended_at).getTime();
  const left = ((start - t0) / total) * 100;
  const width = Math.max(0.5, ((end - start) / total) * 100);

  const row = document.createElement('div');
  row.className = 'sp-waterfall__row';

  const label = document.createElement('span');
  label.className = 'sp-waterfall__label';
  label.textContent = span.name || span.kind;

  const track = document.createElement('div');
  track.className = 'sp-waterfall__track';

  const bar = document.createElement('div');
  // The `sp-` prefix was missing, so every drawer bar rendered unstyled, and
  // the `--bad` modifier it appended has never had a rule to apply.
  bar.className = `sp-waterfall__bar sp-waterfall__swatch--${span.kind}`;
  bar.style.left = `${left}%`;
  bar.style.width = `${width}%`;
  bar.title = `${span.name} · ${span.duration_ms} ms · ${span.status}`;

  const duration = document.createElement('span');
  duration.className = 'sp-waterfall__dur';
  duration.textContent = `${span.duration_ms} ms`;

  bar.append(duration);
  track.append(bar);
  row.append(label, track);
  return row;
};

const renderWaterfall = () => {
  const root = document.querySelector('[data-waterfall]');
  const payload = document.getElementById('trace-spans');
  if (!root || !payload) return;

  let spans;
  try {
    spans = JSON.parse(payload.textContent || '[]');
  } catch {
    return;
  }

  if (!spans.length) {
    const empty = document.createElement('p');
    empty.className = 'sp-u-text-tertiary';
    empty.textContent = 'No spans to plot.';
    root.replaceChildren(empty);
    return;
  }

  const t0 = Math.min(...spans.map((s) => new Date(s.started_at).getTime()));
  const t1 = Math.max(...spans.map((s) => new Date(s.ended_at).getTime()));
  const total = Math.max(1, t1 - t0);

  root.replaceChildren(...spans.map((s) => buildRow(s, t0, total)));
};

const init = () => {
  sizeBars('.sp-latency-histogram', 'data-count', 'data-histogram-max');
  sizeBars('.sp-cost-spark', 'data-cost', 'data-cost-max');
  renderWaterfall();
};

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}
