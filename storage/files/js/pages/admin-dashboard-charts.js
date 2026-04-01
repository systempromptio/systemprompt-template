export const initTrafficChart = () => {
  const canvas = document.getElementById('traffic-chart');
  if (!canvas) return;

  const script = document.querySelector('script[data-chart="traffic"]');
  if (!script) return;

  let data;
  try { data = JSON.parse(script.textContent); } catch (e) { return; }
  if (!data || !data.labels || !data.values) return;

  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const { labels, values } = data;
  const max = Math.max(...values, 1);
  const w = canvas.width;
  const h = canvas.height;
  const pad = 40;
  const barW = Math.max(2, (w - pad * 2) / labels.length - 2);

  ctx.clearRect(0, 0, w, h);
  ctx.fillStyle = 'var(--sp-color-primary, #6366f1)';

  for (let i = 0; i < values.length; i++) {
    const x = pad + i * ((w - pad * 2) / labels.length);
    const barH = (values[i] / max) * (h - pad * 2);
    ctx.fillRect(x, h - pad - barH, barW, barH);
  }

  ctx.fillStyle = 'var(--sp-text-tertiary, #888)';
  ctx.font = '10px sans-serif';
  ctx.textAlign = 'center';
  const step = Math.max(1, Math.floor(labels.length / 8));
  for (let i = 0; i < labels.length; i += step) {
    const x = pad + i * ((w - pad * 2) / labels.length) + barW / 2;
    ctx.fillText(labels[i], x, h - pad + 14);
  }
};

export const initCountryChart = () => {
  const canvas = document.getElementById('country-chart');
  if (!canvas) return;

  const script = document.querySelector('script[data-chart="country"]');
  if (!script) return;

  let data;
  try { data = JSON.parse(script.textContent); } catch (e) { return; }
  if (!Array.isArray(data) || !data.length) return;

  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const w = canvas.width;
  const h = canvas.height;
  const pad = 10;
  const rowH = Math.min(28, (h - pad * 2) / data.length);
  const max = Math.max(...data.map((d) => d.count || d.value || 0), 1);

  ctx.clearRect(0, 0, w, h);

  for (let i = 0; i < data.length; i++) {
    const entry = data[i];
    const val = entry.count || entry.value || 0;
    const barW = (val / max) * (w - pad * 2 - 80);
    const y = pad + i * rowH;

    ctx.fillStyle = 'var(--sp-color-primary, #6366f1)';
    ctx.fillRect(80, y, barW, rowH - 4);

    ctx.fillStyle = 'var(--sp-text-secondary, #666)';
    ctx.font = '11px sans-serif';
    ctx.textAlign = 'right';
    ctx.fillText(entry.country || entry.label || '', 74, y + rowH - 6);
    ctx.textAlign = 'left';
    ctx.fillText(String(val), 80 + barW + 4, y + rowH - 6);
  }
};

export const initSparklines = () => {
  for (const el of document.querySelectorAll('[data-sparkline]')) {
    let values;
    try { values = JSON.parse(el.getAttribute('data-sparkline')); } catch (e) { continue; }
    if (!Array.isArray(values) || !values.length) continue;

    const canvas = document.createElement('canvas');
    canvas.width = el.clientWidth || 120;
    canvas.height = el.clientHeight || 32;
    canvas.style.display = 'block';
    canvas.style.width = '100%';
    canvas.style.height = '100%';
    el.textContent = '';
    el.append(canvas);

    const ctx = canvas.getContext('2d');
    if (!ctx) continue;

    const w = canvas.width;
    const h = canvas.height;
    const max = Math.max(...values, 1);
    const min = Math.min(...values, 0);
    const range = max - min || 1;
    const step = w / (values.length - 1 || 1);

    ctx.beginPath();
    ctx.strokeStyle = getComputedStyle(document.documentElement).getPropertyValue('--sp-color-primary').trim() || '#6366f1';
    ctx.lineWidth = 1.5;

    for (let i = 0; i < values.length; i++) {
      const x = i * step;
      const y = h - ((values[i] - min) / range) * (h - 4) - 2;
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
    ctx.stroke();
  }
};
