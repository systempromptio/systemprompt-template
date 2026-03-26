(function() {
    'use strict';

    var feedEl = document.getElementById('live-feed');
    var indicator = document.getElementById('sse-indicator');
    var ribbon = document.getElementById('metric-ribbon');
    if (!feedEl && !ribbon) return;

    var source = null;
    var reconnectDelay = 1000;

    function connect() {
        source = new EventSource('/admin/api/sse/dashboard');

        source.addEventListener('open', function() {
            if (indicator) indicator.classList.remove('disconnected');
            reconnectDelay = 1000;
        });

        source.addEventListener('activity', function(e) {
            try {
                var events = JSON.parse(e.data);
                for (var i = events.length - 1; i >= 0; i--) {
                    prependFeedItem(events[i]);
                }
                trimFeed(50);
            } catch (_) {}
        });

        source.addEventListener('stats', function(e) {
            try {
                var stats = JSON.parse(e.data);
                updateRibbon(stats);
            } catch (_) {}
        });

        source.addEventListener('error', function() {
            if (indicator) indicator.classList.add('disconnected');
            source.close();
            setTimeout(connect, reconnectDelay);
            reconnectDelay = Math.min(reconnectDelay * 2, 30000);
        });
    }

    function prependFeedItem(evt) {
        if (!feedEl) return;
        var div = document.createElement('div');
        div.className = 'feed-item new-item';
        if (evt.created_at) div.setAttribute('data-ts', evt.created_at);

        var colorClass = 'feed-orange';
        if (evt.category === 'login') colorClass = 'feed-blue';
        else if (evt.category === 'marketplace_connect') colorClass = 'feed-purple';
        else if (evt.category === 'marketplace_edit') colorClass = 'feed-green';
        else if (evt.category === 'session') colorClass = 'feed-cyan';
        else if (evt.category === 'tool_usage') colorClass = 'feed-indigo';
        else if (evt.category === 'error') colorClass = 'feed-red';
        else if (evt.category === 'notification') colorClass = 'feed-amber';
        else if (evt.category === 'agent_response') colorClass = 'feed-teal';

        var name = escapeHtml(evt.display_name || 'Anonymous');
        var desc = escapeHtml(evt.description || '');

        div.innerHTML = '<div class="feed-icon ' + colorClass + '"></div>'
            + '<div class="feed-content">'
            + '<span class="feed-text"><strong>' + name + '</strong> ' + desc + '</span>'
            + '<span class="feed-time">just now</span>'
            + '</div>';

        var footer = feedEl.querySelector('.section-footer');
        if (footer) {
            feedEl.insertBefore(div, footer);
        } else {
            feedEl.insertBefore(div, feedEl.firstChild);
        }
    }

    function trimFeed(max) {
        if (!feedEl) return;
        var items = feedEl.querySelectorAll('.feed-item');
        while (items.length > max) {
            items[items.length - 1].remove();
            items = feedEl.querySelectorAll('.feed-item');
        }
    }

    function updateRibbon(stats) {
        if (!ribbon) return;
        var keys = ['events_today', 'tool_uses', 'prompts', 'total_sessions', 'subagents_spawned', 'error_count', 'total_tokens', 'total_cost', 'failure_count'];
        for (var i = 0; i < keys.length; i++) {
            var key = keys[i];
            if (stats[key] === undefined) continue;
            var item = ribbon.querySelector('[data-key="' + key + '"]');
            if (!item) continue;
            var valEl = item.querySelector('.metric-ribbon-value');
            if (!valEl) continue;
            var oldVal = parseInt(valEl.textContent, 10);
            var newVal = stats[key];
            if (newVal !== oldVal) {
                valEl.textContent = newVal;
                valEl.classList.remove('metric-flash');
                void valEl.offsetWidth;
                valEl.classList.add('metric-flash');
            }
        }
    }

    function escapeHtml(str) {
        var div = document.createElement('div');
        div.textContent = str;
        return div.innerHTML;
    }

    connect();

    window.addEventListener('beforeunload', function() {
        if (source) source.close();
    });
})();
