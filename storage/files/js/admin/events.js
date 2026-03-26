(function(app) {
    'use strict';

    const handlers = {
        click: [],
        change: [],
        keydown: [],
        input: []
    };

    function on(eventType, selector, handler, options) {
        const entry = { selector, handler, exclusive: (options && options.exclusive) || false };
        if (handlers[eventType]) {
            handlers[eventType].push(entry);
        }
    }

    function dispatch(entries, e) {
        for (let i = 0; i < entries.length; i++) {
            const entry = entries[i];
            const match = e.target.closest(entry.selector);
            if (match) {
                entry.handler(e, match);
                if (entry.exclusive) return true;
            }
        }
        return false;
    }

    function init() {
        document.addEventListener('click', function(e) {
            const handled = dispatch(handlers.click, e);
            if (!handled && app.shared) {
                app.shared.closeAllMenus();
            }
        });

        document.addEventListener('change', function(e) {
            dispatch(handlers.change, e);
        });

        document.addEventListener('input', function(e) {
            dispatch(handlers.input, e);
        });

        document.addEventListener('keydown', function(e) {
            if (e.key === 'Escape' && app.shared) {
                app.shared.closeAllMenus();
            }
            dispatch(handlers.keydown, e);
        });
    }

    app.events = { on, init };
})(window.AdminApp);
