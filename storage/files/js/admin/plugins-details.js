(function(app) {
    'use strict';

    window.addEventListener('env-saved', (e) => {
        const pid = e.detail && e.detail.pluginId;
        if (!pid) return;
        const containerId = 'env-status-' + pid;
        const container = document.getElementById(containerId);
        if (container) {
            container.removeAttribute('data-loaded');
            container.replaceChildren();
            const refreshDiv = document.createElement('div');
            refreshDiv.style.cssText = 'padding:var(--sp-space-4);color:var(--sp-text-tertiary);font-size:var(--sp-text-sm)';
            refreshDiv.textContent = 'Refreshing...';
            container.append(refreshDiv);
        }
    });
    app.pluginDetails = { render: () => '' };
})(window.AdminApp);
