// Live audit event stream — subscribes to /admin/api/sse/audit, renders
// severity-coded rows with click-to-open chain drawer, and maintains rolling
// 60s counters. Pause / autoscroll / severity filters / text search are all
// client-side. Notifications API is opt-in.
(function () {
  'use strict';

  var SSE_URL = '/admin/api/sse/audit';
  var MAX_ROWS = 500;
  var WINDOW_MS = 60_000;

  var state = {
    paused: false,
    autoScroll: true,
    notify: false,
    severityFilters: { info: true, warn: true, deny: true, breach: true, error: true },
    searchTerm: '',
    history: [],            // ringbuffer of {ts, severity}
    eventSource: null,
    retryTimeout: null,
    retryDelayMs: 1000,
  };

  function $(sel, root) { return (root || document).querySelector(sel); }
  function $$(sel, root) { return Array.prototype.slice.call((root || document).querySelectorAll(sel)); }

  function escapeText(s) {
    return String(s == null ? '' : s)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');
  }

  function setStatus(label, ok) {
    var stateEl = $('[data-stream-state]');
    var indicator = $('[data-stream-indicator]');
    if (stateEl) stateEl.textContent = label;
    if (indicator) {
      indicator.classList.remove('is-ok', 'is-warn', 'is-down');
      indicator.classList.add(ok === true ? 'is-ok' : (ok === false ? 'is-down' : 'is-warn'));
    }
  }

  function pruneHistory() {
    var cutoff = Date.now() - WINDOW_MS;
    while (state.history.length && state.history[0].ts < cutoff) {
      state.history.shift();
    }
  }

  function updateCounters() {
    pruneHistory();
    var totalRate = state.history.length / (WINDOW_MS / 1000);
    var denyCount = 0, breachCount = 0, errorCount = 0;
    state.history.forEach(function (h) {
      if (h.severity === 'deny' || h.severity === 'breach') denyCount++;
      if (h.severity === 'breach') breachCount++;
      if (h.severity === 'error') errorCount++;
    });
    var denyRate = denyCount / (WINDOW_MS / 1000);
    var rate = $('[data-rail-rate]');
    var denyRateEl = $('[data-rail-deny-rate]');
    var breachEl = $('[data-rail-breach]');
    var errorEl = $('[data-rail-error]');
    if (rate) rate.textContent = totalRate.toFixed(1);
    if (denyRateEl) denyRateEl.textContent = denyRate.toFixed(2);
    if (breachEl) breachEl.textContent = String(breachCount);
    if (errorEl) errorEl.textContent = String(errorCount);
  }

  function shouldShow(payload) {
    var sev = payload.severity || 'info';
    if (!state.severityFilters[sev]) return false;
    if (!state.searchTerm) return true;
    var hay = [
      payload.user_id, payload.tool_name, payload.policy,
      payload.decision, payload.model, payload.session_id, payload.trace_id,
    ].filter(Boolean).join(' ').toLowerCase();
    return hay.indexOf(state.searchTerm) !== -1;
  }

  function severityBadge(sev) {
    var label = String(sev || 'info').toUpperCase();
    return '<span class="event-row__severity event-row__severity--' +
      escapeText(sev) + '">' + escapeText(label) + '</span>';
  }

  function renderRow(payload) {
    var row = document.createElement('article');
    row.className = 'event-row event-row--' + escapeText(payload.severity || 'info');
    if (payload.id) row.setAttribute('data-chain-id', payload.id);
    row.tabIndex = 0;
    row.setAttribute('role', 'button');
    var when = payload.created_at
      ? new Date(payload.created_at).toLocaleTimeString()
      : new Date().toLocaleTimeString();
    var primary = payload.table === 'governance_decisions'
      ? (payload.policy || 'governance') + ' · ' + (payload.decision || '')
      : (payload.model || 'request') + ' · ' + (payload.status || '');
    var secondary = [
      payload.user_id ? 'user ' + payload.user_id : null,
      payload.tool_name ? 'tool ' + payload.tool_name : null,
      payload.tenant_id ? 'tenant ' + payload.tenant_id : null,
    ].filter(Boolean).join(' · ');

    row.innerHTML =
      '<span class="event-row__time">' + escapeText(when) + '</span>' +
      severityBadge(payload.severity) +
      '<span class="event-row__primary">' + escapeText(primary) + '</span>' +
      '<span class="event-row__secondary">' + escapeText(secondary) + '</span>';
    return row;
  }

  function maybeNotify(payload) {
    if (!state.notify) return;
    if (!('Notification' in window)) return;
    if (Notification.permission !== 'granted') return;
    if (payload.severity !== 'breach' && payload.severity !== 'error') return;
    try {
      new Notification('Audit ' + (payload.severity || ''), {
        body: (payload.policy || payload.model || 'event') + ' · ' +
          (payload.decision || payload.status || ''),
        tag: payload.id || String(Date.now()),
      });
    } catch (_e) {
      // Some browsers throw if called from non-secure context — ignore.
    }
  }

  function appendEvent(payload) {
    state.history.push({ ts: Date.now(), severity: payload.severity || 'info' });
    updateCounters();
    if (state.paused) return;
    if (!shouldShow(payload)) return;

    var list = $('[data-stream-list]');
    if (!list) return;
    var empty = $('[data-stream-empty]');
    if (empty) empty.remove();

    var row = renderRow(payload);
    list.append(row);

    while (list.children.length > MAX_ROWS) {
      list.removeChild(list.firstElementChild);
    }
    if (state.autoScroll) {
      list.scrollTop = list.scrollHeight;
    }
    maybeNotify(payload);
  }

  function connect() {
    if (state.eventSource) {
      try { state.eventSource.close(); } catch (_e) {}
    }
    setStatus('connecting…', null);
    var es = new EventSource(SSE_URL);
    state.eventSource = es;

    es.addEventListener('hello', function () {
      setStatus('live', true);
      state.retryDelayMs = 1000;
    });
    es.addEventListener('audit', function (ev) {
      try {
        var payload = JSON.parse(ev.data);
        appendEvent(payload);
      } catch (_e) {
        // ignore malformed payloads
      }
    });
    es.addEventListener('lagged', function (ev) {
      setStatus('lagging', false);
      try {
        var info = JSON.parse(ev.data);
        // surface dropped count as a synthetic warn row
        appendEvent({
          severity: 'warn',
          policy: 'stream',
          decision: 'lagged · ' + (info.skipped || '?') + ' dropped',
        });
      } catch (_e) {}
    });
    es.onerror = function () {
      setStatus('reconnecting…', false);
      es.close();
      state.eventSource = null;
      var delay = Math.min(state.retryDelayMs, 15_000);
      state.retryDelayMs = Math.min(state.retryDelayMs * 2, 15_000);
      clearTimeout(state.retryTimeout);
      state.retryTimeout = setTimeout(connect, delay);
    };
  }

  function bindControls() {
    var toggle = $('[data-stream-toggle]');
    if (toggle) {
      toggle.addEventListener('click', function () {
        state.paused = !state.paused;
        toggle.textContent = state.paused ? 'Resume' : 'Pause';
        toggle.classList.toggle('is-paused', state.paused);
      });
    }

    var auto = $('[data-stream-autoscroll]');
    if (auto) {
      auto.addEventListener('change', function () {
        state.autoScroll = auto.checked;
      });
    }

    var notify = $('[data-stream-notify]');
    if (notify) {
      notify.addEventListener('change', function () {
        if (!notify.checked) {
          state.notify = false;
          return;
        }
        if (!('Notification' in window)) {
          notify.checked = false;
          return;
        }
        if (Notification.permission === 'granted') {
          state.notify = true;
        } else if (Notification.permission !== 'denied') {
          Notification.requestPermission().then(function (p) {
            state.notify = p === 'granted';
            if (!state.notify) notify.checked = false;
          });
        } else {
          state.notify = false;
          notify.checked = false;
        }
      });
    }

    $$('[data-severity-filter]').forEach(function (cb) {
      cb.addEventListener('change', function () {
        state.severityFilters[cb.value] = cb.checked;
      });
    });

    var search = $('[data-stream-search]');
    if (search) {
      search.addEventListener('input', function () {
        state.searchTerm = search.value.trim().toLowerCase();
      });
    }
  }

  function bindChainOpen() {
    var list = $('[data-stream-list]');
    if (!list) return;
    list.addEventListener('keydown', function (ev) {
      if (ev.key !== 'Enter' && ev.key !== ' ') return;
      var target = ev.target;
      if (target && target.getAttribute && target.getAttribute('data-chain-id')) {
        target.click();
      }
    });
  }

  function boot() {
    if (!$('[data-stream-list]')) return;     // page is not events page
    bindControls();
    bindChainOpen();
    connect();
    setInterval(updateCounters, 1000);
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', boot);
  } else {
    boot();
  }
})();
