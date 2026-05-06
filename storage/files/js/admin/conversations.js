(function (app) {
    'use strict';

    function init() {
        var page = document.querySelector('[data-page="conversations"]');
        if (!page) return;

        var toggle = document.getElementById('conversations-redaction-toggle');
        if (toggle && !toggle.disabled) {
            toggle.addEventListener('click', onToggleClick);
        }

        // Mark active jump-nav link as the user scrolls.
        var turnsContainer = document.getElementById('conversations-turns');
        if (turnsContainer && 'IntersectionObserver' in window) {
            var navLinks = page.querySelectorAll('.conversations-page__jumpnav a[data-jump]');
            var byOrdinal = {};
            navLinks.forEach(function (a) { byOrdinal[a.dataset.jump] = a; });
            var io = new IntersectionObserver(function (entries) {
                entries.forEach(function (entry) {
                    if (entry.isIntersecting) {
                        var ord = entry.target.dataset.ordinal;
                        navLinks.forEach(function (a) { a.classList.remove('is-active'); });
                        if (byOrdinal[ord]) byOrdinal[ord].classList.add('is-active');
                    }
                });
            }, { root: turnsContainer, threshold: 0.4 });
            turnsContainer.querySelectorAll('.transcript-turn').forEach(function (el) {
                io.observe(el);
            });
        }
    }

    function onToggleClick(ev) {
        var btn = ev.currentTarget;
        var sessionId = btn.dataset.sessionId;
        var body = document.querySelector('[data-conversation-detail]');
        if (!sessionId || !body) return;

        var mode = body.dataset.redactionMode || 'redacted';
        if (mode === 'raw') {
            switchToMode(body, btn, 'redacted');
            return;
        }

        // Need to fetch raw bodies before we can switch.
        if (btn.dataset.rawLoaded === '1') {
            switchToMode(body, btn, 'raw');
            return;
        }

        btn.disabled = true;
        var originalLabel = btn.textContent;
        btn.textContent = 'Loading…';

        fetchRaw(sessionId)
            .then(function (envelope) {
                applyRawTurns(body, envelope);
                btn.dataset.rawLoaded = '1';
                switchToMode(body, btn, 'raw');
            })
            .catch(function (err) {
                if (app && app.Toast) {
                    app.Toast.show('Could not load raw transcript: ' + err.message, 'error');
                } else {
                    console.error('raw transcript fetch failed', err);
                }
            })
            .then(function () {
                btn.disabled = false;
                if (btn.textContent === 'Loading…') btn.textContent = originalLabel;
            });
    }

    function fetchRaw(sessionId) {
        return fetch('/admin/api/conversations/' + encodeURIComponent(sessionId) + '/raw', {
            credentials: 'same-origin',
            headers: { Accept: 'application/json' }
        }).then(function (r) {
            if (r.status === 403) throw new Error('Forbidden — auditor role required');
            if (!r.ok) throw new Error('HTTP ' + r.status);
            return r.json();
        });
    }

    function applyRawTurns(body, envelope) {
        if (!envelope || !Array.isArray(envelope.turns)) return;
        var turns = body.querySelectorAll('.transcript-turn');
        turns.forEach(function (turnEl) {
            var ordinal = parseInt(turnEl.dataset.ordinal, 10);
            var raw = envelope.turns.find(function (t) { return t.ordinal === ordinal; });
            if (!raw) return;
            // If the partial didn't render the raw <div>, inject one. Otherwise overwrite.
            var bubble = turnEl.querySelector('.transcript-turn__bubble');
            if (!bubble) return;
            var rawEl = bubble.querySelector('.transcript-turn__content--raw');
            if (!rawEl) {
                rawEl = document.createElement('div');
                rawEl.className = 'transcript-turn__content transcript-turn__content--raw';
                rawEl.dataset.mode = 'raw';
                rawEl.hidden = true;
                bubble.appendChild(rawEl);
            }
            rawEl.textContent = raw.content || '';
        });
    }

    function switchToMode(body, btn, mode) {
        body.dataset.redactionMode = mode;
        var redacted = body.querySelectorAll('.transcript-turn__content--redacted');
        var raw = body.querySelectorAll('.transcript-turn__content--raw');
        redacted.forEach(function (el) { el.hidden = (mode === 'raw'); });
        raw.forEach(function (el) { el.hidden = (mode !== 'raw'); });
        btn.textContent = (mode === 'raw') ? 'Show redacted content' : 'Show raw content';
        btn.classList.toggle('is-active', mode === 'raw');
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})(window.AdminApp || (window.AdminApp = {}));
