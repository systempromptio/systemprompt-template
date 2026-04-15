(function(app) {
    'use strict';

    const handlers = {
        click: [],
        change: [],
        keydown: [],
        input: []
    };

    const on = (eventType, selector, handler, options) => {
        const entry = { selector, handler, exclusive: (options && options.exclusive) || false };
        if (handlers[eventType]) {
            handlers[eventType].push(entry);
        }
    };

    const dispatch = (entries, e) => {
        for (let i = 0; i < entries.length; i++) {
            const entry = entries[i];
            const match = e.target.closest(entry.selector);
            if (match) {
                entry.handler(e, match);
                if (entry.exclusive) return true;
            }
        }
        return false;
    };

    const init = () => {
        document.addEventListener('click', (e) => {
            const handled = dispatch(handlers.click, e);
            if (!handled && app.shared) {
                app.shared.closeAllMenus();
            }
        });

        document.addEventListener('change', (e) => {
            dispatch(handlers.change, e);
        });

        document.addEventListener('input', (e) => {
            dispatch(handlers.input, e);
        });

        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && app.shared) {
                app.shared.closeAllMenus();
            }
            dispatch(handlers.keydown, e);
        });
    };

    app.events = { on, init };
})(window.AdminApp);
