(function(app) {
    'use strict';

    const escapeHtml = app.escapeHtml;

    function truncate(str, max) {
        if (!str) return '';
        if (str.length <= (max || 60)) return str;
        return str.substring(0, max || 60) + '...';
    }

    const DropdownManager = {
        portal: null,
        activeMenu: null,
        activeDropdown: null,
        activeTrigger: null,

        init: () => {
            let portal = document.getElementById('dropdown-portal');
            if (!portal) {
                portal = document.createElement('div');
                portal.id = 'dropdown-portal';
                portal.style.cssText = 'position:fixed;top:0;left:0;width:0;height:0;z-index:1000;pointer-events:none;';
                document.body.append(portal);
            }
            DropdownManager.portal = portal;

            document.addEventListener('click', (e) => {
                if (!DropdownManager.activeDropdown) return;
                if (DropdownManager.activeDropdown.contains(e.target)) return;
                if (e.target.closest('[data-action="menu"]')) return;
                DropdownManager.close();
            }, true);

            document.addEventListener('keydown', (e) => {
                if (e.key === 'Escape') DropdownManager.close();
            });
        },

        open: (triggerBtn) => {
            DropdownManager.close();

            const menu = triggerBtn.closest('.actions-menu');
            if (!menu) return;
            const dropdown = menu.querySelector('.actions-dropdown');
            if (!dropdown) return;

            const rect = triggerBtn.getBoundingClientRect();

            const clone = dropdown.cloneNode(true);
            clone.style.cssText = 'position:fixed;' +
                'top:' + (rect.bottom + 4) + 'px;' +
                'right:' + (window.innerWidth - rect.right) + 'px;' +
                'left:auto;' +
                'opacity:1;visibility:visible;transform:translateY(0);pointer-events:auto;' +
                'background:var(--sp-bg-surface-overlay);border:1px solid var(--sp-border-default);' +
                'border-radius:var(--sp-radius-md);box-shadow:var(--sp-shadow-lg);' +
                'min-width:140px;padding:var(--sp-space-1) 0;z-index:1000;';
            clone.setAttribute('data-portal-dropdown', 'true');

            DropdownManager.portal.append(clone);
            DropdownManager.activeMenu = menu;
            DropdownManager.activeDropdown = clone;
            DropdownManager.activeTrigger = triggerBtn;
            menu.classList.add('open');
        },

        close: () => {
            if (DropdownManager.activeDropdown) {
                DropdownManager.activeDropdown.remove();
                DropdownManager.activeDropdown = null;
            }
            if (DropdownManager.activeMenu) {
                DropdownManager.activeMenu.classList.remove('open');
                DropdownManager.activeMenu = null;
            }
            DropdownManager.activeTrigger = null;
        }
    };

    function closeAllMenus() {
        DropdownManager.close();
        document.querySelectorAll('body > .actions-dropdown').forEach(dd => {
            if (dd._originalParent) {
                dd.style.cssText = '';
                dd._originalParent.append(dd);
                dd._originalParent = null;
            }
        });
        document.querySelectorAll('.actions-menu.open').forEach(m => {
            m.classList.remove('open');
        });
        const installMenu = document.getElementById('install-menu');
        if (installMenu && installMenu.classList.contains('open')) {
            installMenu.classList.remove('open');
            const trig = installMenu.querySelector('.install-trigger');
            if (trig) trig.setAttribute('aria-expanded', 'false');
        }
    }

    function closeDeleteConfirm(overlayId) {
        const overlay = document.getElementById(overlayId || 'delete-confirm');
        if (overlay) overlay.remove();
    }

    function showConfirmDialog(title, message, confirmLabel, onConfirm, opts) {
        const btnClass = (opts && opts.btnClass) || 'btn-danger';
        const overlay = document.createElement('div');
        overlay.className = 'confirm-overlay';
        if (opts && opts.id) overlay.id = opts.id;
        overlay.innerHTML = '<div class="confirm-dialog">' +
            '<h3>' + escapeHtml(title) + '</h3>' +
            '<p>' + escapeHtml(message) + '</p>' +
            '<div style="display:flex;gap:var(--sp-space-3);justify-content:flex-end;margin-top:var(--sp-space-5)">' +
                '<button class="btn btn-secondary" data-action="cancel">Cancel</button>' +
                '<button class="btn ' + btnClass + '" data-action="confirm">' + escapeHtml(confirmLabel) + '</button>' +
            '</div>' +
        '</div>';
        overlay.querySelector('[data-action="cancel"]').addEventListener('click', () => overlay.remove());
        overlay.querySelector('[data-action="confirm"]').addEventListener('click', () => {
            overlay.remove();
            onConfirm();
        });
        overlay.addEventListener('click', e => {
            if (e.target === overlay) overlay.remove();
        });
        document.body.append(overlay);
        return overlay;
    }

    function showDeleteConfirmDialog(title, itemId) {
        const overlay = document.createElement('div');
        overlay.className = 'confirm-overlay';
        overlay.id = 'delete-confirm';
        overlay.innerHTML = '<div class="confirm-dialog">' +
            '<h3>' + escapeHtml(title) + '</h3>' +
            '<p>This action cannot be undone.</p>' +
            '<div style="display:flex;gap:var(--sp-space-3);justify-content:flex-end;margin-top:var(--sp-space-5)">' +
                '<button class="btn btn-secondary" data-confirm-cancel>Cancel</button>' +
                '<button class="btn btn-danger" data-confirm-delete="' + escapeHtml(itemId) + '">Delete</button>' +
            '</div>' +
        '</div>';
        document.body.append(overlay);
        return overlay;
    }

    function createDebouncedSearch(root, inputId, onSearch, delay) {
        let searchTimer = null;
        app.events.on('input', '#' + inputId, (e, el) => {
            clearTimeout(searchTimer);
            searchTimer = setTimeout(() => {
                onSearch(el.value);
                const input = document.getElementById(inputId);
                if (input) {
                    input.focus();
                    input.selectionStart = input.selectionEnd = input.value.length;
                }
            }, delay || 200);
        });
    }

    function handleMenuToggle(e, menuBtn) {
        if (!menuBtn) menuBtn = e.target.closest('[data-action="menu"]');
        if (!menuBtn) return false;
        const menu = menuBtn.closest('.actions-menu');
        const wasOpen = menu && menu.classList.contains('open');
        DropdownManager.close();
        if (!wasOpen) {
            DropdownManager.open(menuBtn);
        }
        return true;
    }

    function showLoading(el, msg) {
        el.innerHTML = '<div class="loading-spinner"><div class="spinner"></div><p>' +
            escapeHtml(msg || 'Loading...') + '</p></div>';
    }

    function showEmpty(el, msg) {
        el.innerHTML = '<div class="empty-state"><p>' + escapeHtml(msg) + '</p></div>';
    }

    function loadJSZip() {
        return new Promise((resolve, reject) => {
            if (window.JSZip) return resolve(window.JSZip);
            const script = document.createElement('script');
            script.src = 'https://cdnjs.cloudflare.com/ajax/libs/jszip/3.10.1/jszip.min.js';
            script.integrity = 'sha384-+mbV2IY1Zk/X1p/nWllGySJSUN8uMs+gUAN10Or95UBH0fpj6GfKgPmgC5EXieXG';
            script.crossOrigin = 'anonymous';
            script.onload = () => resolve(window.JSZip);
            script.onerror = () => reject(new Error('Failed to load JSZip'));
            document.head.append(script);
        });
    }

    app.events.on('click', '[data-action="menu"]', (e, menuBtn) => {
        handleMenuToggle(e, menuBtn);
    }, { exclusive: true });

    app.events.on('click', '[data-confirm-cancel]', () => closeDeleteConfirm(), { exclusive: true });
    app.events.on('click', '#delete-confirm', (e, el) => {
        if (e.target === el) closeDeleteConfirm();
    }, { exclusive: true });

    const ROLES = ['admin', 'ceo', 'finance', 'sales', 'marketing', 'operations', 'hr', 'it', 'support'];
    const HOOK_EVENTS = ['PostToolUse', 'SessionStart', 'PreToolUse', 'Notification'];
    app.constants = { ROLES, HOOK_EVENTS };

    DropdownManager.init();

    app.shared = {
        truncate,
        closeAllMenus,
        closeDeleteConfirm,
        showConfirmDialog,
        showDeleteConfirmDialog,
        createDebouncedSearch,
        handleMenuToggle,
        showLoading,
        showEmpty,
        loadJSZip,
        DropdownManager
    };
})(window.AdminApp);
