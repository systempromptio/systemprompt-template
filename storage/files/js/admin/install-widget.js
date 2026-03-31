(function(app) {
    'use strict';

    app.events.on('click', '.install-trigger', (e, trigger) => {
        const menu = document.getElementById('install-menu');
        if (!menu) return;
        const isOpen = menu.classList.contains('open');
        menu.classList.toggle('open', !isOpen);
        trigger.setAttribute('aria-expanded', !isOpen);
    }, { exclusive: true });

    app.events.on('click', '[data-install-tab]', (e, tabBtn) => {
        const widget = tabBtn.closest('.install-menu');
        if (!widget) return;
        const tabId = tabBtn.getAttribute('data-install-tab');
        widget.querySelectorAll('[data-install-tab]').forEach((b) => {
            b.classList.toggle('active', b === tabBtn);
        });
        widget.querySelectorAll('[data-install-panel]').forEach((p) => {
            p.style.display = p.getAttribute('data-install-panel') === tabId ? '' : 'none';
        });
    });

    app.events.on('click', '[data-copy]', (e, copyBtn) => {
        const text = copyBtn.getAttribute('data-copy');
        navigator.clipboard.writeText(text).then(() => {
            var savedNodes = Array.from(copyBtn.childNodes).map((n) => n.cloneNode(true));
            copyBtn.replaceChildren();
            var checkSpan = document.createElement('span');
            checkSpan.style.cssText = 'color:var(--sp-success);font-size:16px';
            checkSpan.textContent = '\u2713';
            copyBtn.append(checkSpan);
            setTimeout(() => {
                copyBtn.replaceChildren();
                savedNodes.forEach((n) => { copyBtn.append(n); });
            }, 2000);
        });
    });

    app.events.on('click', '[data-action="toggle-token"]', (e, btn) => {
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

    document.addEventListener('DOMContentLoaded', () => {
        const tokenEl = document.querySelector('.install-token-value[data-masked="true"]');
        if (tokenEl) tokenEl.style.filter = 'blur(4px)';
    });
})(window.AdminApp);
