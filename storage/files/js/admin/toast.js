(function(app) {
    'use strict';

    let container = null;
    const icons = {
        success: '\u2713',
        error: '\u2717',
        info: '\u24D8'
    };
    app.Toast = {
        init() {
            if (container) return;
            container = document.createElement('div');
            container.className = 'toast-container';
            document.body.append(container);
        },
        show(message, type) {
            if (!container) this.init();
            type = type || 'info';
            const icon = icons[type] || icons.info;
            const el = document.createElement('div');
            el.className = 'toast toast-' + type;
            el.innerHTML = '<span class="toast-icon">' + icon + '</span>' +
                '<span class="toast-message">' + app.escapeHtml(message) + '</span>';
            container.append(el);
            setTimeout(() => {
                el.style.opacity = '0';
                setTimeout(() => { el.remove(); }, 300);
            }, 4000);
        }
    };
})(window.AdminApp);
