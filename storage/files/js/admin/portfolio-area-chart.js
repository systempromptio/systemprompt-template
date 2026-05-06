// Portfolio stacked-area chart — reads JSON from
// <script type="application/json" id="portfolio-area-data"> and renders an
// allow/deny stacked area on <canvas id="portfolio-area-canvas">.
//
// Data shape:
//   { buckets: [{ label, allow, deny }, ...] }
//
// No external chart lib — single canvas pass with the same getContext/moveTo
// patterns used by admin-dashboard-charts.js. Tokens read from CSS via
// computed style; falls back to hard-coded sane colors if not set.
(function () {
  'use strict';

  function readData() {
    var el = document.getElementById('portfolio-area-data');
    if (!el) return null;
    try {
      var parsed = JSON.parse(el.textContent || '{}');
      if (!parsed || !Array.isArray(parsed.buckets) || parsed.buckets.length < 2) return null;
      return parsed;
    } catch (_e) {
      return null;
    }
  }

  function token(name, fallback) {
    var v = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    return v || fallback;
  }

  function fillCanvas(canvas, w, h) {
    var dpr = window.devicePixelRatio || 1;
    canvas.width = Math.max(1, Math.floor(w * dpr));
    canvas.height = Math.max(1, Math.floor(h * dpr));
    canvas.style.width = w + 'px';
    canvas.style.height = h + 'px';
    var ctx = canvas.getContext('2d');
    if (ctx) ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    return ctx;
  }

  function draw() {
    var data = readData();
    if (!data) return;
    var canvas = document.getElementById('portfolio-area-canvas');
    if (!canvas) return;

    var rect = canvas.getBoundingClientRect();
    var w = rect.width || 800;
    var h = rect.height || 220;
    var ctx = fillCanvas(canvas, w, h);
    if (!ctx) return;

    var padL = 36;
    var padR = 8;
    var padT = 8;
    var padB = 24;
    var plotW = w - padL - padR;
    var plotH = h - padT - padB;

    var buckets = data.buckets;
    var n = buckets.length;
    var totals = buckets.map(function (b) { return (b.allow || 0) + (b.deny || 0); });
    var max = Math.max.apply(null, totals);
    if (max === 0) max = 1;

    var step = plotW / (n - 1);

    var allowColor = token('--sp-success', '#16a34a');
    var denyColor = token('--sp-danger', '#dc2626');
    var gridColor = token('--sp-border-default', '#e5e7eb');
    var textColor = token('--sp-text-tertiary', '#888');

    // Y grid (4 lines including 0 and max)
    ctx.strokeStyle = gridColor;
    ctx.lineWidth = 1;
    ctx.fillStyle = textColor;
    ctx.font = '10px sans-serif';
    ctx.textAlign = 'right';
    ctx.textBaseline = 'middle';
    for (var g = 0; g <= 4; g++) {
      var yv = max * (1 - g / 4);
      var y = padT + (plotH * g) / 4;
      ctx.beginPath();
      ctx.moveTo(padL, y);
      ctx.lineTo(padL + plotW, y);
      ctx.stroke();
      ctx.fillText(Math.round(yv), padL - 4, y);
    }

    function pointAt(i, value) {
      return {
        x: padL + i * step,
        y: padT + plotH - (value / max) * plotH,
      };
    }

    // Allow band (bottom): polygon from baseline up to allow values
    ctx.beginPath();
    ctx.moveTo(padL, padT + plotH);
    for (var i = 0; i < n; i++) {
      var p = pointAt(i, buckets[i].allow || 0);
      ctx.lineTo(p.x, p.y);
    }
    ctx.lineTo(padL + plotW, padT + plotH);
    ctx.closePath();
    ctx.fillStyle = allowColor;
    ctx.globalAlpha = 0.55;
    ctx.fill();
    ctx.globalAlpha = 1;

    // Deny band (stacked on top): polygon from allow line up to allow+deny
    ctx.beginPath();
    var first = pointAt(0, buckets[0].allow || 0);
    ctx.moveTo(first.x, first.y);
    for (var k = 0; k < n; k++) {
      var bk = buckets[k];
      var top = pointAt(k, (bk.allow || 0) + (bk.deny || 0));
      ctx.lineTo(top.x, top.y);
    }
    for (var m = n - 1; m >= 0; m--) {
      var bottom = pointAt(m, buckets[m].allow || 0);
      ctx.lineTo(bottom.x, bottom.y);
    }
    ctx.closePath();
    ctx.fillStyle = denyColor;
    ctx.globalAlpha = 0.6;
    ctx.fill();
    ctx.globalAlpha = 1;

    // X labels — show ~6 evenly-spaced ticks
    ctx.fillStyle = textColor;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'top';
    var labelStep = Math.max(1, Math.floor(n / 6));
    for (var li = 0; li < n; li += labelStep) {
      var lx = padL + li * step;
      ctx.fillText(buckets[li].label || '', lx, padT + plotH + 4);
    }
  }

  function boot() {
    draw();
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', boot);
  } else {
    boot();
  }

  var t;
  window.addEventListener('resize', function () {
    clearTimeout(t);
    t = setTimeout(draw, 120);
  });
})();
