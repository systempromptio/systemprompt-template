// Sparkline auto-mounter — finds every `[data-sparkline]` element with a JSON
// array of numbers and draws a line + faint area underneath inside an injected
// canvas. Color comes from CSS `currentColor`, which the sparkline-card maps
// from `data-sparkline-color`. Hidpi-aware.
(function () {
  'use strict';

  function readPoints(el) {
    var raw = el.getAttribute('data-sparkline');
    if (!raw) return null;
    try {
      var parsed = JSON.parse(raw);
      if (!Array.isArray(parsed) || parsed.length < 2) return null;
      return parsed.map(function (n) {
        var v = Number(n);
        return Number.isFinite(v) ? v : 0;
      });
    } catch (_e) {
      return null;
    }
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

  function draw(el, points) {
    var rect = el.getBoundingClientRect();
    var w = rect.width || 120;
    var h = rect.height || 32;
    if (w < 4 || h < 4) return;

    var canvas = el.querySelector('canvas');
    if (!canvas) {
      canvas = document.createElement('canvas');
      el.textContent = '';
      el.append(canvas);
    }
    var ctx = fillCanvas(canvas, w, h);
    if (!ctx) return;

    var color = window.getComputedStyle(el).color || '#6366f1';
    var max = Math.max.apply(null, points);
    var min = Math.min.apply(null, points);
    var range = max - min || 1;
    var pad = 1.5;
    var step = (w - pad * 2) / (points.length - 1);

    var coords = points.map(function (v, i) {
      return {
        x: pad + i * step,
        y: pad + (h - pad * 2) - ((v - min) / range) * (h - pad * 2),
      };
    });

    // Faint area fill
    ctx.beginPath();
    ctx.moveTo(coords[0].x, h);
    for (var i = 0; i < coords.length; i++) ctx.lineTo(coords[i].x, coords[i].y);
    ctx.lineTo(coords[coords.length - 1].x, h);
    ctx.closePath();
    ctx.fillStyle = color;
    ctx.globalAlpha = 0.12;
    ctx.fill();
    ctx.globalAlpha = 1;

    // Line
    ctx.beginPath();
    ctx.moveTo(coords[0].x, coords[0].y);
    for (var j = 1; j < coords.length; j++) ctx.lineTo(coords[j].x, coords[j].y);
    ctx.lineWidth = 1.5;
    ctx.strokeStyle = color;
    ctx.stroke();
  }

  function mount() {
    document.querySelectorAll('[data-sparkline]').forEach(function (el) {
      var pts = readPoints(el);
      if (pts) draw(el, pts);
    });
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', mount);
  } else {
    mount();
  }

  // Redraw on resize (debounced)
  var t;
  window.addEventListener('resize', function () {
    clearTimeout(t);
    t = setTimeout(mount, 120);
  });
})();
