(function(app) {
    'use strict';

    window.addEventListener('env-saved', (e) => {
        const pid = e.detail && e.detail.pluginId;
        if (!pid) return;
        const containerId = 'env-status-' + pid;
        const container = document.getElementById(containerId);
        if (container) {
            container.removeAttribute('data-loaded');
            container.innerHTML = '<div style="padding:var(--space-4);color:var(--text-tertiary);font-size:var(--text-sm)">Refreshing...</div>';
        }
    });
    app.pluginDetails = { render: () => '' };
})(window.AdminApp);
