(function (global) {
    'use strict';

    var DRAWER_ID = 'chain-drawer';
    var QS_KEY = 'chain';
    var FOUR_STAGES = ['scope', 'secret_scan', 'blocklist', 'rate_limit'];

    var lastFocused = null;

    function getDrawer() {
        return document.getElementById(DRAWER_ID);
    }

    function escapeText(s) {
        return String(s == null ? '' : s)
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;');
    }

    function formatCost(microdollars) {
        if (microdollars == null || isNaN(microdollars)) return '—';
        var dollars = Number(microdollars) / 1000000;
        if (dollars === 0) return '$0';
        if (dollars < 0.01) return '$' + dollars.toFixed(6);
        return '$' + dollars.toFixed(4);
    }

    function formatTokens(input, output) {
        if (input == null && output == null) return '—';
        return (input || 0).toLocaleString() + ' / ' + (output || 0).toLocaleString();
    }

    function formatTime(isoString) {
        if (!isoString) return '—';
        try {
            var d = new Date(isoString);
            if (isNaN(d.getTime())) return isoString;
            return d.toISOString().replace('T', ' ').replace(/\..+/, '');
        } catch (e) {
            return isoString;
        }
    }

    function setText(el, value, fallback) {
        if (!el) return;
        var t = (value == null || value === '') ? (fallback || '—') : value;
        el.textContent = String(t);
    }

    function clearChildren(el) {
        if (!el) return;
        while (el.firstChild) el.removeChild(el.firstChild);
    }

    function copyToClipboard(text) {
        if (!text) return;
        if (navigator.clipboard && navigator.clipboard.writeText) {
            navigator.clipboard.writeText(text).catch(function () {});
            return;
        }
        var ta = document.createElement('textarea');
        ta.value = text;
        ta.setAttribute('readonly', '');
        ta.style.position = 'absolute';
        ta.style.left = '-9999px';
        document.body.appendChild(ta);
        ta.select();
        try { document.execCommand('copy'); } catch (e) {}
        document.body.removeChild(ta);
    }

    function renderHeader(drawer, env) {
        var traceEl = drawer.querySelector('[data-chain-trace-id]');
        if (traceEl) {
            traceEl.textContent = env.trace_id || env.session_id || '—';
            traceEl.dataset.value = env.trace_id || env.session_id || '';
        }

        var statusEl = drawer.querySelector('[data-chain-status]');
        if (statusEl) {
            statusEl.classList.remove('chain-drawer__pill--allow', 'chain-drawer__pill--deny');
            var hasDeny = (env.totals && env.totals.deny_count > 0);
            statusEl.textContent = hasDeny ? 'denied' : 'allowed';
            statusEl.classList.add(hasDeny ? 'chain-drawer__pill--deny' : 'chain-drawer__pill--allow');
        }

        var idEl = drawer.querySelector('[data-chain-identity]');
        if (idEl && env.identity) {
            var parts = [];
            if (env.identity.user_id) parts.push('user=' + env.identity.user_id);
            if (env.identity.tenant_id) parts.push('tenant=' + env.identity.tenant_id);
            if (env.identity.agent_id) parts.push('agent=' + env.identity.agent_id);
            idEl.textContent = parts.join(' · ');
        }

        var totals = env.totals || {};
        setText(drawer.querySelector('[data-chain-total="decisions"]'),
                totals.decision_count != null ? totals.decision_count : '—');
        setText(drawer.querySelector('[data-chain-total="denies"]'),
                totals.deny_count != null ? totals.deny_count : '—');
        setText(drawer.querySelector('[data-chain-total="cost"]'),
                formatCost(totals.total_cost_microdollars));
        setText(drawer.querySelector('[data-chain-total="tokens"]'),
                formatTokens(totals.total_input_tokens, totals.total_output_tokens));
    }

    function renderStepper(drawer, env) {
        var byPolicy = {};
        var decisions = (env.decisions || []);
        for (var i = 0; i < decisions.length; i++) {
            var d = decisions[i];
            // First match wins; we want the earliest decision per stage.
            if (!byPolicy[d.policy]) byPolicy[d.policy] = d;
        }

        for (var s = 0; s < FOUR_STAGES.length; s++) {
            var stage = FOUR_STAGES[s];
            var li = drawer.querySelector('.chain-drawer__stage[data-stage="' + stage + '"]');
            if (!li) continue;
            li.classList.remove('chain-drawer__stage--pass', 'chain-drawer__stage--fail',
                                'chain-drawer__stage--skipped');
            // remove any previous detail row
            var oldDetail = li.querySelector('.chain-drawer__stage-detail');
            if (oldDetail) oldDetail.remove();

            var stateEl = li.querySelector('.chain-drawer__stage-state');
            var hit = byPolicy[stage];
            if (!hit) {
                li.classList.add('chain-drawer__stage--skipped');
                if (stateEl) stateEl.textContent = 'skipped';
                continue;
            }

            var failed = (hit.decision === 'deny');
            li.classList.add(failed ? 'chain-drawer__stage--fail' : 'chain-drawer__stage--pass');
            if (stateEl) stateEl.textContent = failed ? 'fail' : 'pass';

            if (hit.reason) {
                var detail = document.createElement('span');
                detail.className = 'chain-drawer__stage-detail';
                detail.textContent = hit.reason;
                li.appendChild(detail);
            }
        }
    }

    function renderEvents(drawer, env) {
        var ul = drawer.querySelector('[data-chain-events]');
        if (!ul) return;
        clearChildren(ul);
        var events = env.events || [];
        if (!events.length) {
            var empty = document.createElement('li');
            empty.className = 'chain-drawer__empty';
            empty.textContent = 'No tool calls.';
            ul.appendChild(empty);
            return;
        }
        for (var i = 0; i < events.length; i++) {
            var ev = events[i];
            var li = document.createElement('li');
            li.className = 'chain-drawer__event';
            var type = document.createElement('span');
            type.className = 'chain-drawer__event-type';
            type.textContent = ev.event_type || '—';
            var tool = document.createElement('span');
            tool.className = 'chain-drawer__event-tool';
            tool.textContent = (ev.tool_name || ev.description || '').slice(0, 200);
            var time = document.createElement('span');
            time.className = 'chain-drawer__event-time';
            time.textContent = formatTime(ev.created_at);
            li.appendChild(type);
            li.appendChild(tool);
            li.appendChild(time);
            ul.appendChild(li);
        }
    }

    function renderRequests(drawer, env) {
        var table = drawer.querySelector('[data-chain-requests]');
        if (!table) return;
        var tbody = table.querySelector('tbody');
        if (!tbody) return;
        clearChildren(tbody);

        var rows = env.requests || [];
        if (!rows.length) {
            var tr = document.createElement('tr');
            var td = document.createElement('td');
            td.colSpan = 6;
            td.className = 'chain-drawer__empty';
            td.textContent = 'No AI requests.';
            tr.appendChild(td);
            tbody.appendChild(tr);
            return;
        }

        for (var i = 0; i < rows.length; i++) {
            var r = rows[i];
            var tr2 = document.createElement('tr');
            tr2.appendChild(td(formatTime(r.created_at)));
            tr2.appendChild(td(r.model || '—'));
            tr2.appendChild(td(r.status || '—'));
            tr2.appendChild(td(formatTokens(r.input_tokens, r.output_tokens)));
            tr2.appendChild(td(r.latency_ms != null ? r.latency_ms + ' ms' : '—'));
            tr2.appendChild(td(formatCost(r.cost_microdollars)));
            tbody.appendChild(tr2);
        }

        function td(text) {
            var c = document.createElement('td');
            c.textContent = text;
            return c;
        }
    }

    function renderTranscript(drawer, env) {
        var holder = drawer.querySelector('[data-chain-transcript]');
        if (!holder) return;
        clearChildren(holder);

        var summary = env.summary;
        if (summary && (summary.ai_title || summary.ai_summary)) {
            if (summary.ai_title) {
                var h = document.createElement('p');
                h.style.fontWeight = '600';
                h.textContent = summary.ai_title;
                holder.appendChild(h);
            }
            if (summary.ai_summary) {
                var p = document.createElement('p');
                p.textContent = summary.ai_summary;
                holder.appendChild(p);
            }
        }

        if (!env.transcript) {
            if (!holder.firstChild) {
                var empty = document.createElement('p');
                empty.className = 'chain-drawer__empty';
                empty.textContent = 'No transcript captured.';
                holder.appendChild(empty);
            }
            return;
        }

        var tEl = document.createElement('div');
        tEl.className = 'json-tree';
        holder.appendChild(tEl);
        mountJson(tEl, env.transcript.transcript, { rootLabel: 'transcript', startCollapsed: true });
    }

    function renderRaw(drawer, env) {
        var holder = drawer.querySelector('[data-chain-raw]');
        if (!holder) return;
        clearChildren(holder);
        var el = document.createElement('div');
        el.className = 'json-tree';
        holder.appendChild(el);
        mountJson(el, env, { rootLabel: 'envelope', startCollapsed: true });
    }

    function mountJson(rootEl, value, options) {
        var ns = global.SystempromptAdmin && global.SystempromptAdmin.jsonTree;
        if (ns && typeof ns.mountJsonTree === 'function') {
            ns.mountJsonTree(rootEl, value, options);
            return;
        }
        // fallback: stringified pre
        var pre = document.createElement('pre');
        try {
            pre.textContent = JSON.stringify(value, null, 2);
        } catch (e) {
            pre.textContent = String(value);
        }
        rootEl.appendChild(pre);
    }

    function renderError(drawer, message) {
        var panel = drawer.querySelector('.chain-drawer__panel');
        if (!panel) return;
        var existing = panel.querySelector('.chain-drawer__error');
        if (existing) existing.remove();
        var div = document.createElement('div');
        div.className = 'chain-drawer__error';
        div.textContent = message;
        panel.insertBefore(div, panel.firstChild.nextSibling);
    }

    function clearError(drawer) {
        var existing = drawer.querySelector('.chain-drawer__error');
        if (existing) existing.remove();
    }

    function showDrawer(drawer) {
        if (!drawer || drawer.hidden === false) return;
        lastFocused = document.activeElement;
        drawer.hidden = false;
        document.body.style.overflow = 'hidden';
        // Defer focus to next frame so animation can begin first.
        requestAnimationFrame(function () {
            var closeBtn = drawer.querySelector('.chain-drawer__close');
            if (closeBtn) closeBtn.focus();
        });
    }

    function hideDrawer() {
        var drawer = getDrawer();
        if (!drawer || drawer.hidden) return;
        drawer.hidden = true;
        document.body.style.overflow = '';
        clearError(drawer);
        if (lastFocused && typeof lastFocused.focus === 'function') {
            lastFocused.focus();
        }
        // Strip ?chain= from URL without reload.
        try {
            var url = new URL(window.location.href);
            if (url.searchParams.has(QS_KEY)) {
                url.searchParams.delete(QS_KEY);
                window.history.replaceState({}, '', url.toString());
            }
        } catch (e) {}
    }

    function trapFocus(drawer, event) {
        if (event.key !== 'Tab') return;
        var focusable = drawer.querySelectorAll(
            'button:not([disabled]), [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
        );
        if (!focusable.length) return;
        var first = focusable[0];
        var last = focusable[focusable.length - 1];
        if (event.shiftKey) {
            if (document.activeElement === first || !drawer.contains(document.activeElement)) {
                event.preventDefault();
                last.focus();
            }
        } else if (document.activeElement === last) {
            event.preventDefault();
            first.focus();
        }
    }

    function openChainDrawer(id) {
        var drawer = getDrawer();
        if (!drawer || !id) return;
        clearError(drawer);
        showDrawer(drawer);

        // Update URL deep-link
        try {
            var url = new URL(window.location.href);
            url.searchParams.set(QS_KEY, id);
            window.history.replaceState({}, '', url.toString());
        } catch (e) {}

        fetch('/admin/api/chain/' + encodeURIComponent(id), {
            credentials: 'same-origin',
            headers: { 'Accept': 'application/json' }
        })
            .then(function (resp) {
                if (resp.status === 404) throw new Error('No chain found for ' + id);
                if (!resp.ok) throw new Error('Failed (' + resp.status + ')');
                return resp.json();
            })
            .then(function (env) {
                renderHeader(drawer, env);
                renderStepper(drawer, env);
                renderEvents(drawer, env);
                renderRequests(drawer, env);
                renderTranscript(drawer, env);
                renderRaw(drawer, env);
            })
            .catch(function (err) {
                renderError(drawer, err.message || String(err));
            });
    }

    function bindGlobalHandlers() {
        document.addEventListener('click', function (e) {
            // Close handlers
            var closeEl = e.target.closest && e.target.closest('[data-chain-close]');
            if (closeEl) {
                e.preventDefault();
                hideDrawer();
                return;
            }

            // Copy handlers
            var copyEl = e.target.closest && e.target.closest('[data-chain-copy]');
            if (copyEl) {
                e.preventDefault();
                var key = copyEl.getAttribute('data-chain-copy');
                var drawer = getDrawer();
                if (drawer) {
                    var src = drawer.querySelector('[data-chain-' + key + ']');
                    if (src) copyToClipboard(src.dataset.value || src.textContent || '');
                }
                return;
            }

            // Open trigger
            var trigger = e.target.closest && e.target.closest('[data-chain-id]');
            if (trigger) {
                var id = trigger.getAttribute('data-chain-id');
                if (!id) return;
                e.preventDefault();
                openChainDrawer(id);
            }
        });

        document.addEventListener('keydown', function (e) {
            var drawer = getDrawer();
            if (!drawer || drawer.hidden) return;
            if (e.key === 'Escape') {
                e.preventDefault();
                hideDrawer();
                return;
            }
            trapFocus(drawer, e);
        });
    }

    function checkDeepLink() {
        try {
            var url = new URL(window.location.href);
            var id = url.searchParams.get(QS_KEY);
            if (id) openChainDrawer(id);
        } catch (e) {}
    }

    function init() {
        bindGlobalHandlers();
        checkDeepLink();
    }

    var ns = global.SystempromptAdmin = global.SystempromptAdmin || {};
    ns.chainDrawer = {
        open: openChainDrawer,
        close: hideDrawer
    };

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})(window);
