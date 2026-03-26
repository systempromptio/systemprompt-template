(function(app) {
    'use strict';

    app.events.on('click', '.install-trigger', function(e, trigger) {
        const menu = document.getElementById('install-menu');
        if (!menu) return;
        const isOpen = menu.classList.contains('open');
        menu.classList.toggle('open', !isOpen);
        trigger.setAttribute('aria-expanded', !isOpen);
    }, { exclusive: true });

    app.events.on('click', '[data-install-tab]', function(e, tabBtn) {
        const widget = tabBtn.closest('.install-menu');
        if (!widget) return;
        const tabId = tabBtn.getAttribute('data-install-tab');
        widget.querySelectorAll('[data-install-tab]').forEach(function(b) {
            b.classList.toggle('active', b === tabBtn);
        });
        widget.querySelectorAll('[data-install-panel]').forEach(function(p) {
            p.style.display = p.getAttribute('data-install-panel') === tabId ? '' : 'none';
        });
    });

    app.events.on('click', '[data-copy]', function(e, copyBtn) {
        const text = copyBtn.getAttribute('data-copy');
        navigator.clipboard.writeText(text).then(function() {
            const orig = copyBtn.innerHTML;
            copyBtn.innerHTML = '<span style="color:var(--success);font-size:16px">&#10003;</span>';
            setTimeout(function() { copyBtn.innerHTML = orig; }, 2000);
        });
    });

    app.events.on('click', '[data-action="toggle-token"]', function(e, btn) {
        const box = btn.closest('.install-token-box');
        if (!box) return;
        const code = box.querySelector('.install-token-value');
        if (!code) return;
        const masked = code.getAttribute('data-masked') === 'true';
        if (masked) {
            code.style.filter = 'none';
            code.setAttribute('data-masked', 'false');
            btn.title = 'Hide token';
        } else {
            code.style.filter = 'blur(4px)';
            code.setAttribute('data-masked', 'true');
            btn.title = 'Show token';
        }
    });

    // Apply initial mask on load
    document.addEventListener('DOMContentLoaded', function() {
        var tokenEl = document.querySelector('.install-token-value[data-masked="true"]');
        if (tokenEl) tokenEl.style.filter = 'blur(4px)';
    });
})(window.AdminApp);
