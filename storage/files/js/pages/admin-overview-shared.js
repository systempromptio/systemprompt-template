// Live Overview — shared primitives for the five real-time panes.
//
// Provides:
//   • openStream(url, handlers)       → wraps EventSource with auto-reconnect
//   • appendTicker(el, row, max)      → prepends a rich expandable row to a ticker
//   • initTickerExpand(el)            → attach delegated expand/collapse handler
//   • updateTickerRow(el, id, delta)  → flip tag on an existing row in place
//   • Sparkline(el, buckets)          → 60-second rolling bar chart
//   • bumpKpi(el, n)                  → increment a numeric KPI smoothly
//   • formatTime(iso)                 → HH:MM:SS

export function openStream(url, handlers) {
  let es = null;
  let backoff = 1000;
  let stopped = false;
  let retryTimer = null;

  const close = () => {
    stopped = true;
    if (retryTimer) {
      clearTimeout(retryTimer);
      retryTimer = null;
    }
    es?.close();
    es = null;
  };

  const connect = () => {
    if (stopped) return;
    es = new EventSource(url, { withCredentials: true });
    es.addEventListener('hello', () => {
      backoff = 1000;
      handlers.onOpen?.();
    });
    es.addEventListener('lagged', () => handlers.onLag?.());
    for (const [name, fn] of Object.entries(handlers.events ?? {})) {
      es.addEventListener(name, (ev) => {
        let payload;
        try { payload = JSON.parse(ev.data); } catch { return; }
        fn(payload);
      });
    }
    es.onerror = () => {
      es?.close();
      es = null;
      if (stopped) return;
      handlers.onError?.();
      retryTimer = setTimeout(connect, Math.min(backoff, 30000));
      backoff *= 2;
    };
  };

  // Close the stream as soon as the page is leaving — `pagehide` fires for
  // both navigation and bfcache. Without this, an in-flight SSE keeps a
  // socket parked under the HTTP/1.1 6-per-origin limit, which can stall the
  // next page's document request behind it.
  const onPageHide = () => close();
  addEventListener('pagehide', onPageHide, { once: true });
  addEventListener('beforeunload', onPageHide, { once: true });

  connect();
  return { close };
}

function makeKv(label, value, href) {
  const kv = document.createElement('div');
  kv.className = 'live-ticker__detail-kv';
  const lEl = document.createElement('span');
  lEl.className = 'live-ticker__detail-label';
  lEl.textContent = label;
  let vEl;
  if (href) {
    vEl = document.createElement('a');
    vEl.className = 'live-ticker__detail-link';
    vEl.href = href;
    vEl.textContent = `${value} ↗`;
  } else {
    vEl = document.createElement('span');
    vEl.className = 'live-ticker__detail-value';
    vEl.textContent = value;
  }
  kv.append(lEl, vEl);
  return kv;
}

function buildSummaryText(actor, target, latency, cost, contextId) {
  let s = target ? `${actor ?? '—'} → ${target}` : (actor ?? '—');
  if (latency) s += `  ${latency}`;
  if (cost && cost !== '$0.0000' && cost !== '$0.000000') s += `  ${cost}`;
  if (contextId) s += `  ${contextId.slice(0, 8)}…`;
  return s;
}

function buildTag(tag, tagClass, errorHref) {
  let g;
  if (errorHref) {
    g = document.createElement('a');
    g.href = errorHref;
    g.title = 'View request detail';
  } else {
    g = document.createElement('span');
  }
  g.className = `live-ticker__tag${tagClass ? ' ' + tagClass : ''}`;
  g.textContent = tag ?? '';
  return g;
}

export function appendTicker(el, {
  time, actor, target, tag, tagClass, traceId,
  userId, sessionId, contextId, requestId, displayName, department,
  policy, reason, latency, cost, errorHref, errorMessage,
}, max = 100) {
  if (!el) return;

  const msgText = buildSummaryText(actor, target, latency, cost, contextId);

  // Dedup by contextId — update summary and promote the existing row to the top
  if (contextId) {
    const existing = el.querySelector(`[data-context-id="${CSS.escape(contextId)}"]`);
    if (existing) {
      const m = existing.querySelector('.live-ticker__msg');
      if (m) m.textContent = msgText;
      const oldTag = existing.querySelector('.live-ticker__tag');
      const newTag = buildTag(tag, tagClass, errorHref);
      if (oldTag) oldTag.replaceWith(newTag);
      const t = existing.querySelector('.live-ticker__time');
      if (t) t.textContent = time;
      if (errorMessage) {
        const detail = existing.querySelector('.live-ticker__detail');
        if (detail) {
          let errKv = detail.querySelector('[data-kv="Error"]');
          if (!errKv) {
            errKv = makeKv('Error', errorMessage, null);
            errKv.dataset.kv = 'Error';
            detail.append(errKv);
          } else {
            const span = errKv.querySelector('.live-ticker__detail-value');
            if (span) span.textContent = errorMessage;
          }
        }
      }
      el.prepend(existing);
      return;
    }
  }

  const li = document.createElement('li');
  li.className = 'live-ticker__row';
  if (traceId)   li.dataset.traceId   = traceId;
  if (contextId) li.dataset.contextId = contextId;

  // Summary line
  const summary = document.createElement('div');
  summary.className = 'live-ticker__summary';

  const chevron = document.createElement('span');
  chevron.className = 'expand-indicator';
  chevron.setAttribute('aria-hidden', 'true');
  chevron.textContent = '▶';

  const t = document.createElement('span');
  t.className = 'live-ticker__time';
  t.textContent = time;

  const m = document.createElement('span');
  m.className = 'live-ticker__msg';
  m.textContent = msgText;

  summary.append(chevron, t, m, buildTag(tag, tagClass, errorHref));

  // Detail section
  const detail = document.createElement('div');
  detail.className = 'live-ticker__detail';

  if (displayName || userId) {
    const label = displayName || userId.slice(0, 8);
    const href = userId ? `/admin/user?id=${encodeURIComponent(userId)}` : null;
    detail.append(makeKv('User', label, href));
  }
  if (department)  detail.append(makeKv('Dept',    department, null));
  if (sessionId)   detail.append(makeKv('Session', sessionId.slice(0, 14) + '…', '/admin/users/sessions'));
  if (contextId)   detail.append(makeKv('Context', contextId, null));
  if (traceId)      detail.append(makeKv('Trace',   traceId.slice(0, 8) + '…', `/admin/analytics/requests?q=${encodeURIComponent(traceId)}&preset=7d`));
  if (requestId)    detail.append(makeKv('Request', requestId.slice(0, 8) + '…', null));
  if (policy)       detail.append(makeKv('Policy',  reason ? `${policy} — ${reason}` : policy, null));
  if (errorMessage) detail.append(makeKv('Error',   errorMessage, null));

  li.append(summary, detail);
  el.prepend(li);
  while (el.children.length > max) el.lastElementChild.remove();
}

// Attach a delegated click handler to a ticker <ul> for expand/collapse.
// Call once after the element is available. Idempotent.
export function initTickerExpand(el) {
  if (!el || el.dataset.expandInit) return;
  el.dataset.expandInit = '1';
  el.addEventListener('click', (e) => {
    // Let link clicks (detail links AND clickable error tag) navigate normally
    if (e.target.closest('a.live-ticker__detail-link')) return;
    // Clicking the error tag badge should navigate, not expand
    if (e.target.closest('a.live-ticker__tag')) return;
    const row = e.target.closest('.live-ticker__row');
    if (row) row.classList.toggle('is-expanded');
  });
}

// Finds a row by data-trace-id and updates its tag in place.
// Returns true if found, false otherwise.
export function updateTickerRow(el, traceId, { tag, tagClass }) {
  if (!el || !traceId) return false;
  const li = el.querySelector(`[data-trace-id="${CSS.escape(traceId)}"]`);
  if (!li) return false;
  const g = li.querySelector('.live-ticker__tag');
  if (g) {
    g.textContent = tag ?? '';
    g.className = `live-ticker__tag${tagClass ? ' ' + tagClass : ''}`;
  }
  return true;
}

export function formatTime(iso) {
  const d = iso ? new Date(iso) : new Date();
  const hh = String(d.getHours()).padStart(2, '0');
  const mm = String(d.getMinutes()).padStart(2, '0');
  const ss = String(d.getSeconds()).padStart(2, '0');
  return `${hh}:${mm}:${ss}`;
}

export function bumpKpi(el, delta = 1) {
  if (!el) return;
  const cur = Number.parseInt(el.textContent.replace(/,/g, ''), 10) || 0;
  el.textContent = (cur + delta).toLocaleString();
  el.animate(
    [{ color: 'oklch(0.62 0.20 145)' }, { color: '' }],
    { duration: 700, easing: 'ease-out' },
  );
}

export function setText(el, text) {
  if (!el) return;
  if (el.textContent !== String(text)) {
    el.textContent = text;
  }
}

export class Sparkline {
  constructor(el, buckets = 60) {
    this.el = el;
    this.buckets = buckets;
    this.bars = [];
    if (!el) return;
    el.innerHTML = '';
    for (let i = 0; i < buckets; i++) {
      const b = document.createElement('div');
      b.className = 'live-spark__bar';
      el.append(b);
      this.bars.push(b);
    }
    this.counts = new Array(buckets).fill(0);
    this.errors = new Array(buckets).fill(0);
    this.head = 0;
    this.max = 1;
    this.tick = setInterval(() => this.advance(), 1000);
  }

  add(isError = false) {
    if (!this.el) return;
    this.counts[this.head] += 1;
    if (isError) this.errors[this.head] += 1;
    this.render();
  }

  advance() {
    if (!this.el) return;
    this.head = (this.head + 1) % this.buckets;
    this.counts[this.head] = 0;
    this.errors[this.head] = 0;
    this.render();
  }

  render() {
    this.max = Math.max(1, ...this.counts);
    for (let i = 0; i < this.buckets; i++) {
      const idx = (this.head + 1 + i) % this.buckets;
      const v = this.counts[idx];
      const pct = (v / this.max) * 100;
      this.bars[i].style.height = `${pct}%`;
      this.bars[i].classList.toggle('is-error', this.errors[idx] > 0);
    }
  }

  destroy() {
    clearInterval(this.tick);
  }
}

export function statusEl(id) {
  return document.getElementById(id);
}

export function setStatus(el, text) {
  if (el) el.textContent = text;
}
