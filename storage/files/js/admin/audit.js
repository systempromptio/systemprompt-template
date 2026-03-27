(function(app) {
    'use strict';

    const buildQueryString = () => {
        const params = new URLSearchParams();
        const from = document.getElementById('audit-from').value;
        const to = document.getElementById('audit-to').value;
        const dept = document.getElementById('audit-dept').value.trim();
        const user = document.getElementById('audit-user').value.trim();
        const skill = document.getElementById('audit-skill').value.trim();
        const format = document.getElementById('audit-format').value;
        if (from) params.set('from', from);
        if (to) params.set('to', to);
        if (dept) params.set('department', dept);
        if (user) params.set('user_id', user);
        if (skill) params.set('skill', skill);
        if (format) params.set('format', format);
        return params.toString();
    };

    app.auditInteractions = () => {
        app.events.on('click', '#btn-preview', () => {
            const qs = buildQueryString();
            window.location.href = app.BASE + '/audit/?' + qs;
        });

        app.events.on('click', '#btn-download', () => {
            const format = document.getElementById('audit-format').value;
            const downloadQs = buildQueryString();
            if (format === 'csv') {
                window.open(app.API_BASE + '/export/usage?' + downloadQs.replace(/from=([^&]+)/, (m, v) => 'from=' + v + 'T00:00:00Z').replace(/to=([^&]+)/, (m, v) => 'to=' + v + 'T23:59:59Z'), '_blank');
            } else {
                app.api('/export/usage?' + downloadQs.replace(/from=([^&]+)/, (m, v) => 'from=' + v + 'T00:00:00Z').replace(/to=([^&]+)/, (m, v) => 'to=' + v + 'T23:59:59Z')).then((rows) => {
                    const blob = new Blob([JSON.stringify(rows, null, 2)], { type: 'application/json' });
                    const url = URL.createObjectURL(blob);
                    const a = document.createElement('a');
                    a.href = url;
                    a.download = 'usage-export.json';
                    a.click();
                    URL.revokeObjectURL(url);
                }).catch((err) => {
                    app.Toast.show(err.message || 'Download failed', 'error');
                });
            }
        });
    };
})(window.AdminApp);
