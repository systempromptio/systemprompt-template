(function() {
    'use strict';

    const feedEl = document.getElementById('live-feed');
    const indicator = document.getElementById('sse-indicator');
    const ribbon = document.getElementById('metric-ribbon');
    if (!feedEl && !ribbon) return;

    let source = null;
    let reconnectDelay = 1000;

    const connect = () => {
        source = new EventSource('/admin/api/sse/dashboard');

        source.addEventListener('open', () => {
            if (indicator) indicator.classList.remove('disconnected');
            reconnectDelay = 1000;
        });

        source.addEventListener('activity', (e) => {
            try {
                const events = JSON.parse(e.data);
                for (let i = events.length - 1; i >= 0; i--) {
                    prependFeedItem(events[i]);
                }
                trimFeed(50);
            } catch (_) {}
        });

        source.addEventListener('stats', (e) => {
            try {
                const stats = JSON.parse(e.data);
                updateRibbon(stats);
            } catch (_) {}
        });

        source.addEventListener('error', () => {
            if (indicator) indicator.classList.add('disconnected');
            source.close();
            setTimeout(connect, reconnectDelay);
            reconnectDelay = Math.min(reconnectDelay * 2, 30000);
        });
    };

    const prependFeedItem = (evt) => {
        if (!feedEl) return;
        const div = document.createElement('div');
        div.className = 'feed-item new-item';
        if (evt.created_at) div.setAttribute('data-ts', evt.created_at);

        let colorClass = 'feed-orange';
        if (evt.category === 'login') colorClass = 'feed-blue';
        else if (evt.category === 'marketplace_connect') colorClass = 'feed-purple';
        else if (evt.category === 'marketplace_edit') colorClass = 'feed-green';
        else if (evt.category === 'session') colorClass = 'feed-cyan';
        else if (evt.category === 'tool_usage') colorClass = 'feed-indigo';
        else if (evt.category === 'error') colorClass = 'feed-red';
        else if (evt.category === 'notification') colorClass = 'feed-amber';
        else if (evt.category === 'agent_response') colorClass = 'feed-teal';

        const name = evt.display_name || 'Anonymous';
        const desc = evt.description || '';

        const iconEl = document.createElement('div');
        iconEl.className = 'feed-icon ' + colorClass;

        const contentEl = document.createElement('div');
        contentEl.className = 'feed-content';

        const textSpan = document.createElement('span');
        textSpan.className = 'feed-text';
        const strong = document.createElement('strong');
        strong.textContent = name;
        textSpan.append(strong, ' ' + desc);

        const timeSpan = document.createElement('span');
        timeSpan.className = 'feed-time';
        timeSpan.textContent = 'just now';

        contentEl.append(textSpan, timeSpan);
        div.append(iconEl, contentEl);

        const footer = feedEl.querySelector('.section-footer');
        if (footer) {
            feedEl.insertBefore(div, footer);
        } else {
            feedEl.insertBefore(div, feedEl.firstChild);
        }
    };

    const trimFeed = (max) => {
        if (!feedEl) return;
        let items = feedEl.querySelectorAll('.feed-item');
        while (items.length > max) {
            items[items.length - 1].remove();
            items = feedEl.querySelectorAll('.feed-item');
        }
    };

    const updateRibbon = (stats) => {
        if (!ribbon) return;
        const keys = ['events_today', 'tool_uses', 'prompts', 'total_sessions', 'subagents_spawned', 'error_count', 'total_tokens', 'total_cost', 'failure_count'];
        for (let i = 0; i < keys.length; i++) {
            const key = keys[i];
            if (stats[key] === undefined) continue;
            const item = ribbon.querySelector('[data-key="' + key + '"]');
            if (!item) continue;
            const valEl = item.querySelector('.metric-ribbon-value');
            if (!valEl) continue;
            const oldVal = parseInt(valEl.textContent, 10);
            const newVal = stats[key];
            if (newVal !== oldVal) {
                valEl.textContent = newVal;
                valEl.classList.remove('metric-flash');
                void valEl.offsetWidth;
                valEl.classList.add('metric-flash');
            }
        }
    };

    connect();

    window.addEventListener('beforeunload', () => {
        if (source) source.close();
    });
})();
