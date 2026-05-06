// Waterfall renderer for the per-trace detail page.
// Reads spans from <script type="application/json" id="trace-spans"> and
// draws an inline SVG with one row per span. Bars color-coded by `kind` via
// CSS class (`.waterfall__bar--{kind}`); deny/error get an extra outline.
// Each row is clickable: it dispatches `data-chain-id` which the existing
// chain-drawer.js picks up.
(function () {
  'use strict';

  var ROW_H = 26;
  var BAR_H = 14;
  var LABEL_W = 220;
  var RIGHT_W = 80;
  var AXIS_H = 22;
  var PAD_TOP = 8;
  var PAD_BOTTOM = 4;
  var MIN_BAR_PX = 3;

  function readSpans() {
    var el = document.getElementById('trace-spans');
    if (!el) return null;
    try {
      var arr = JSON.parse(el.textContent || '[]');
      return Array.isArray(arr) ? arr : null;
    } catch (_e) {
      return null;
    }
  }

  function svgEl(name, attrs) {
    var ns = 'http://www.w3.org/2000/svg';
    var n = document.createElementNS(ns, name);
    if (attrs) {
      for (var k in attrs) {
        if (Object.prototype.hasOwnProperty.call(attrs, k)) {
          n.setAttribute(k, attrs[k]);
        }
      }
    }
    return n;
  }

  function escapeText(s) {
    return String(s == null ? '' : s);
  }

  function formatDuration(ms) {
    if (ms < 1) return '<1 ms';
    if (ms < 1000) return ms + ' ms';
    if (ms < 60000) return (ms / 1000).toFixed(2) + ' s';
    return (ms / 60000).toFixed(1) + ' min';
  }

  function render(root, spans) {
    if (!spans || !spans.length) {
      root.innerHTML = '<div class="waterfall__empty">No spans in this trace.</div>';
      return;
    }

    var startsMs = spans.map(function (s) { return new Date(s.started_at).getTime(); });
    var endsMs = spans.map(function (s) {
      var e = new Date(s.ended_at).getTime();
      var st = new Date(s.started_at).getTime();
      // Point-in-time spans (duration_ms === 0) get a 1px nudge so they remain visible.
      return Math.max(e, st);
    });
    var totalStart = Math.min.apply(null, startsMs);
    var totalEnd = Math.max.apply(null, endsMs);
    var totalSpan = Math.max(1, totalEnd - totalStart);

    var rect = root.getBoundingClientRect();
    var width = Math.max(rect.width || 800, 600);
    var plotL = LABEL_W;
    var plotR = width - RIGHT_W;
    var plotW = Math.max(50, plotR - plotL);
    var height = PAD_TOP + AXIS_H + spans.length * ROW_H + PAD_BOTTOM;

    var svg = svgEl('svg', {
      'class': 'waterfall__svg',
      'viewBox': '0 0 ' + width + ' ' + height,
      'role': 'img',
      'aria-label': 'Trace waterfall with ' + spans.length + ' spans',
    });

    // Axis grid (4 ticks)
    var axisY = PAD_TOP + AXIS_H - 4;
    for (var t = 0; t <= 4; t++) {
      var x = plotL + (plotW * t) / 4;
      var ms = (totalSpan * t) / 4;
      var line = svgEl('line', {
        'class': 'waterfall__grid',
        'x1': x.toFixed(1), 'y1': PAD_TOP + AXIS_H,
        'x2': x.toFixed(1), 'y2': height - PAD_BOTTOM,
      });
      svg.append(line);
      var lbl = svgEl('text', {
        'class': 'waterfall__axis',
        'x': x.toFixed(1), 'y': axisY,
        'text-anchor': t === 0 ? 'start' : (t === 4 ? 'end' : 'middle'),
      });
      lbl.textContent = formatDuration(ms);
      svg.append(lbl);
    }

    // Per-span row
    spans.forEach(function (s, i) {
      var rowY = PAD_TOP + AXIS_H + i * ROW_H;
      var startMs = new Date(s.started_at).getTime() - totalStart;
      var dur = Math.max(0, s.duration_ms || 0);
      var x0 = plotL + (startMs / totalSpan) * plotW;
      var w = (dur / totalSpan) * plotW;
      if (w < MIN_BAR_PX) w = MIN_BAR_PX;

      // Row background — clickable area for chain-drawer
      var bg = svgEl('rect', {
        'class': 'waterfall__row-bg',
        'x': 0, 'y': rowY,
        'width': width, 'height': ROW_H,
        'fill': 'transparent',
      });
      svg.append(bg);

      // Group whose click triggers chain drawer via data-chain-id
      var g = svgEl('g', {
        'class': 'waterfall__row',
        'data-chain-id': s.id,
        'tabindex': '0',
        'role': 'button',
        'aria-label': 'Open chain envelope for ' + (s.name || s.kind),
      });

      // Left label (name)
      var label = svgEl('text', {
        'class': 'waterfall__label',
        'x': 8, 'y': rowY + ROW_H / 2 + 4,
      });
      var labelText = escapeText(s.name || s.kind);
      if (labelText.length > 30) labelText = labelText.slice(0, 30) + '…';
      label.textContent = labelText;
      g.append(label);

      // Bar
      var classes = ['waterfall__bar', 'waterfall__bar--' + (s.kind || 'tool')];
      if (s.status === 'deny') classes.push('waterfall__bar--deny');
      if (s.status === 'error') classes.push('waterfall__bar--error');
      var bar = svgEl('rect', {
        'class': classes.join(' '),
        'x': x0.toFixed(1),
        'y': rowY + (ROW_H - BAR_H) / 2,
        'width': w.toFixed(1),
        'height': BAR_H,
        'rx': 2, 'ry': 2,
      });
      var titleEl = svgEl('title');
      titleEl.textContent = (s.kind || '') + ' · ' + (s.name || '') +
        ' · ' + formatDuration(dur) + ' · ' + (s.status || '');
      bar.append(titleEl);
      g.append(bar);

      // Right-side duration label
      var right = svgEl('text', {
        'class': 'waterfall__label waterfall__label--right',
        'x': width - 8, 'y': rowY + ROW_H / 2 + 4,
        'text-anchor': 'end',
      });
      right.textContent = formatDuration(dur);
      g.append(right);

      svg.append(g);
    });

    root.textContent = '';
    root.append(svg);
  }

  function dispatchChainOpen(target) {
    // Walk up to find data-chain-id on the SVG group.
    var n = target;
    while (n && n !== document) {
      if (n.getAttribute && n.getAttribute('data-chain-id')) {
        // The chain-drawer.js picks up clicks via document delegation, but
        // SVG events sometimes don't bubble cleanly — synthesize a click on
        // a hidden anchor with the same attribute.
        var ghost = document.createElement('button');
        ghost.setAttribute('data-chain-id', n.getAttribute('data-chain-id'));
        ghost.style.display = 'none';
        document.body.append(ghost);
        ghost.click();
        ghost.remove();
        return;
      }
      n = n.parentNode;
    }
  }

  function bindClicks(root) {
    root.addEventListener('click', function (ev) {
      dispatchChainOpen(ev.target);
    });
    root.addEventListener('keydown', function (ev) {
      if (ev.key !== 'Enter' && ev.key !== ' ') return;
      dispatchChainOpen(ev.target);
    });
  }

  function boot() {
    var root = document.querySelector('[data-waterfall]');
    if (!root) return;
    var spans = readSpans();
    render(root, spans || []);
    bindClicks(root);
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', boot);
  } else {
    boot();
  }

  var t;
  window.addEventListener('resize', function () {
    clearTimeout(t);
    t = setTimeout(function () {
      var root = document.querySelector('[data-waterfall]');
      if (!root) return;
      var spans = readSpans();
      render(root, spans || []);
    }, 150);
  });
})();
