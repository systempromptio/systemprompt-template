window.AdminApp = window.AdminApp || {};
(function(app) {
    'use strict';

    app.BASE = window.ADMIN_BASE || '/admin';
    app.API_BASE = window.ADMIN_API_BASE || '/api/public/admin';
    app.api = async (path, options) => {
        const url = app.API_BASE + path;
        const resp = await fetch(url, {
            headers: { 'Content-Type': 'application/json' },
            ...options
        });
        if (!resp.ok) {
            const text = await resp.text();
            const err = new Error(text || resp.statusText);
            if (app.Toast) {
                app.Toast.show(err.message, 'error');
            }
            throw err;
        }
        return resp.json();
    };
    app.getUserInitials = (name) => {
        if (!name) return '?';
        return name.split(/[\s@._-]/).filter(Boolean).slice(0, 2).map((s) => s[0].toUpperCase()).join('');
    };
    app.formatDate = (iso) => {
        if (!iso) return '-';
        const d = new Date(iso);
        return d.toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
    };
    app.formatRelativeTime = (iso) => {
        if (!iso) return '-';
        const now = Date.now();
        const then = new Date(iso).getTime();
        const diff = now - then;
        const mins = Math.floor(diff / 60000);
        if (mins < 1) return 'just now';
        if (mins < 60) return mins + 'm ago';
        const hours = Math.floor(mins / 60);
        if (hours < 24) return hours + 'h ago';
        const days = Math.floor(hours / 24);
        if (days < 30) return days + 'd ago';
        return app.formatDate(iso);
    };
    app.escapeHtml = (str) => {
        if (!str) return '';
        return String(str).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
    };
    (() => {
        try {
            const cookie = document.cookie.split(';').find((c) => c.trim().startsWith('access_token='));
            if (cookie) {
                const token = cookie.trim().split('=')[1];
                const payload = JSON.parse(atob(token.split('.')[1]));
                app.user = { id: payload.sub, username: payload.username, email: payload.email };
            }
        } catch(e) {}
    })();
    (async () => {
        try {
            const resp = await fetch('/admin/auth/me');
            if (resp.ok) {
                const me = await resp.json();
                app.userContext = me;
                const meta = document.getElementById('user-meta');
                if (meta) {
                    const parts = [];
                    if (me.department) {
                        parts.push(app.escapeHtml(me.department));
                    }
                    (me.roles || []).forEach((role) => {
                        if (role !== 'user') {
                            parts.push(app.escapeHtml(role.charAt(0).toUpperCase() + role.slice(1)));
                        }
                    });
                    meta.textContent = parts.join(' \u00b7 ');
                }
            }
        } catch(e) {}
    })();
    const logoutBtn = document.getElementById('btn-logout');
    if (logoutBtn) {
        logoutBtn.addEventListener('click', () => {
            fetch(app.API_BASE.replace('/admin', '') + '/auth/session', { method: 'DELETE' })
                .finally(() => {
                    sessionStorage.clear();
                    window.location.href = app.BASE;
                });
        });
    }
})(window.AdminApp);
