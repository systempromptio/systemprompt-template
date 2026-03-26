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
                    (me.roles || []).forEach(function(role) {
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
            document.body.appendChild(container);
        },
        show(message, type) {
            if (!container) this.init();
            type = type || 'info';
            const icon = icons[type] || icons.info;
            const el = document.createElement('div');
            el.className = 'toast toast-' + type;
            el.innerHTML = '<span class="toast-icon">' + icon + '</span>' +
                '<span class="toast-message">' + app.escapeHtml(message) + '</span>';
            container.appendChild(el);
            setTimeout(() => {
                el.style.opacity = '0';
                setTimeout(() => { el.remove(); }, 300);
            }, 4000);
        }
    };
})(window.AdminApp);

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

(function(app) {
    'use strict';

    const escapeHtml = app.escapeHtml;

    function truncate(str, max) {
        if (!str) return '';
        if (str.length <= (max || 60)) return str;
        return str.substring(0, max || 60) + '...';
    }

    // Centralized dropdown manager — clone-based portal, no DOM reparenting
    const DropdownManager = {
        portal: null,
        activeMenu: null,
        activeDropdown: null,
        activeTrigger: null,

        init: function() {
            var portal = document.getElementById('dropdown-portal');
            if (!portal) {
                portal = document.createElement('div');
                portal.id = 'dropdown-portal';
                portal.style.cssText = 'position:fixed;top:0;left:0;width:0;height:0;z-index:1000;pointer-events:none;';
                document.body.appendChild(portal);
            }
            DropdownManager.portal = portal;

            // Capture-phase click handler to close dropdown when clicking outside
            document.addEventListener('click', function(e) {
                if (!DropdownManager.activeDropdown) return;
                if (DropdownManager.activeDropdown.contains(e.target)) return;
                if (e.target.closest('[data-action="menu"]')) return;
                DropdownManager.close();
            }, true);

            document.addEventListener('keydown', function(e) {
                if (e.key === 'Escape') DropdownManager.close();
            });
        },

        open: function(triggerBtn) {
            DropdownManager.close();

            var menu = triggerBtn.closest('.actions-menu');
            if (!menu) return;
            var dropdown = menu.querySelector('.actions-dropdown');
            if (!dropdown) return;

            var rect = triggerBtn.getBoundingClientRect();

            // Clone dropdown into portal — original stays hidden in DOM
            var clone = dropdown.cloneNode(true);
            clone.style.cssText = 'position:fixed;' +
                'top:' + (rect.bottom + 4) + 'px;' +
                'right:' + (window.innerWidth - rect.right) + 'px;' +
                'left:auto;' +
                'opacity:1;visibility:visible;transform:translateY(0);pointer-events:auto;' +
                'background:var(--bg-surface-overlay);border:1px solid var(--border-default);' +
                'border-radius:var(--radius-md);box-shadow:var(--shadow-lg);' +
                'min-width:140px;padding:var(--space-1) 0;z-index:1000;';
            clone.setAttribute('data-portal-dropdown', 'true');

            DropdownManager.portal.appendChild(clone);
            DropdownManager.activeMenu = menu;
            DropdownManager.activeDropdown = clone;
            DropdownManager.activeTrigger = triggerBtn;
            menu.classList.add('open');
        },

        close: function() {
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
        // Legacy cleanup: handle any old-style reparented dropdowns
        document.querySelectorAll('body > .actions-dropdown').forEach(dd => {
            if (dd._originalParent) {
                dd.style.cssText = '';
                dd._originalParent.appendChild(dd);
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
            '<div style="display:flex;gap:var(--space-3);justify-content:flex-end;margin-top:var(--space-5)">' +
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
        document.body.appendChild(overlay);
        return overlay;
    }

    function showDeleteConfirmDialog(title, itemId) {
        const overlay = document.createElement('div');
        overlay.className = 'confirm-overlay';
        overlay.id = 'delete-confirm';
        overlay.innerHTML = '<div class="confirm-dialog">' +
            '<h3>' + escapeHtml(title) + '</h3>' +
            '<p>This action cannot be undone.</p>' +
            '<div style="display:flex;gap:var(--space-3);justify-content:flex-end;margin-top:var(--space-5)">' +
                '<button class="btn btn-secondary" data-confirm-cancel>Cancel</button>' +
                '<button class="btn btn-danger" data-confirm-delete="' + escapeHtml(itemId) + '">Delete</button>' +
            '</div>' +
        '</div>';
        document.body.appendChild(overlay);
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

    // Legacy function kept for backward compat — now delegates to DropdownManager
    function handleMenuToggle(e, menuBtn) {
        if (!menuBtn) menuBtn = e.target.closest('[data-action="menu"]');
        if (!menuBtn) return false;
        var menu = menuBtn.closest('.actions-menu');
        var wasOpen = menu && menu.classList.contains('open');
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
            document.head.appendChild(script);
        });
    }

    // Global menu toggle handler
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

    // Initialize dropdown manager
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

(function(app) {
    'use strict';

    const escapeHtml = app.escapeHtml;

    function renderChecklist(id, items, selected, label, opts) {
        const options = opts || {};
        const selectedSet = {};
        if (Array.isArray(selected)) {
            (selected || []).forEach((s) => {
                const key = typeof s === 'string' ? s : (s.name || s.id || s);
                selectedSet[key] = true;
            });
        } else if (selected && typeof selected === 'object') {
            Object.keys(selected).forEach((k) => {
                if (selected[k]) selectedSet[k] = true;
            });
        }
        const hasItems = items && items.length > 0;
        const listItems = hasItems ? items.map((item) => {
            const val = typeof item === 'string' ? item : (item.name || item.id || item);
            const displayName = typeof item === 'string' ? item : (item.name || item.id || String(item));
            const checked = selectedSet[val] ? ' checked' : '';
            const itemId = id + '-chk-' + val.replace(/[^a-zA-Z0-9_-]/g, '_');
            return '<div class="checklist-item" data-item-name="' + escapeHtml(val.toLowerCase()) + '">' +
                '<input type="checkbox" name="' + escapeHtml(id) + '" value="' + escapeHtml(val) + '"' + checked + ' id="' + escapeHtml(itemId) + '">' +
                '<label for="' + escapeHtml(itemId) + '">' + escapeHtml(displayName) + '</label>' +
            '</div>';
        }).join('') : '<div class="empty-state" style="padding:var(--space-4)"><p>None available.</p></div>';
        let filterRow = '<input type="text" class="field-input" placeholder="Filter..." data-filter-list="' + escapeHtml(id) + '" style="margin-bottom:var(--space-2)">';
        if (options.hasSelectAll) {
            filterRow = '<div style="display:flex;gap:var(--space-2);margin-bottom:var(--space-2)">' +
                '<input type="text" class="field-input" placeholder="Search..." data-filter-list="' + escapeHtml(id) + '" style="flex:1">' +
                '<button type="button" class="btn btn-secondary btn-sm" data-select-all="' + escapeHtml(id) + '">Select All</button>' +
                '<button type="button" class="btn btn-secondary btn-sm" data-deselect-all="' + escapeHtml(id) + '">Deselect All</button>' +
            '</div>';
        }
        const maxHeight = options.hasSelectAll ? '300px' : '200px';
        return '<div class="form-group">' +
            '<label class="field-label">' + escapeHtml(label) + '</label>' +
            filterRow +
            '<div class="checklist-container" data-checklist="' + escapeHtml(id) + '" style="max-height:' + maxHeight + ';overflow-y:auto;border:1px solid var(--border-subtle);border-radius:var(--radius-md);padding:var(--space-2)">' +
                listItems +
            '</div>' +
        '</div>';
    }

    function attachFilterHandlers(root) {
        root.addEventListener('input', (e) => {
            const filterInput = e.target.closest('[data-filter-list]');
            if (!filterInput) return;
            const listId = filterInput.getAttribute('data-filter-list');
            const container = root.querySelector('[data-checklist="' + listId + '"]');
            if (!container) return;
            const q = filterInput.value.toLowerCase();
            const items = container.querySelectorAll('.checklist-item');
            items.forEach((item) => {
                const name = item.getAttribute('data-item-name') || '';
                item.style.display = (q && name.indexOf(q) < 0) ? 'none' : '';
            });
        });
    }

    function getCheckedValues(form, name) {
        const checked = form.querySelectorAll('input[name="' + name + '"]:checked');
        return Array.from(checked).map((cb) => cb.value);
    }
    app.formUtils = {
        renderChecklist,
        attachFilterHandlers,
        getCheckedValues
    };
})(window.AdminApp);

(function(app) {
    'use strict';

    const shared = app.shared;

    function initTableSort() {
        const tables = document.querySelectorAll('.data-table');
        for (let t = 0; t < tables.length; t++) {
            setupTableSort(tables[t]);
        }
    }

    function setupTableSort(table) {
        if (table.getAttribute('data-sort-init')) return;
        table.setAttribute('data-sort-init', '1');
        const headers = table.querySelectorAll('thead th');
        if (!headers.length) return;

        for (let i = 0; i < headers.length; i++) {
            const th = headers[i];
            const text = th.textContent.trim();
            if (!text || th.classList.contains('col-actions') || th.classList.contains('col-chevron')) {
                th.style.cursor = 'default';
                continue;
            }
            if (!th.querySelector('.sort-icon')) {
                const icon = document.createElement('span');
                icon.className = 'sort-icon';
                icon.textContent = '\u25B4';
                th.appendChild(icon);
            }
            th.setAttribute('data-sort-col', i);
            th.addEventListener('click', handleSortClick);
        }
    }

    function handleSortClick(e) {
        const th = e.currentTarget;
        const table = th.closest('.data-table');
        if (!table) return;
        const colIndex = parseInt(th.getAttribute('data-sort-col'), 10);
        const tbody = table.querySelector('tbody');
        if (!tbody) return;

        const wasAsc = th.classList.contains('sorted') && th.getAttribute('data-sort-dir') === 'asc';
        const dir = wasAsc ? 'desc' : 'asc';

        const allHeaders = table.querySelectorAll('thead th');
        for (let i = 0; i < allHeaders.length; i++) {
            allHeaders[i].classList.remove('sorted');
            allHeaders[i].removeAttribute('data-sort-dir');
            const icon = allHeaders[i].querySelector('.sort-icon');
            if (icon) icon.textContent = '\u25B4';
        }

        th.classList.add('sorted');
        th.setAttribute('data-sort-dir', dir);
        const activeIcon = th.querySelector('.sort-icon');
        if (activeIcon) activeIcon.textContent = dir === 'asc' ? '\u25B4' : '\u25BE';

        const allRows = Array.prototype.slice.call(tbody.querySelectorAll('tr'));
        const sortableRows = [];
        const detailMap = {};
        for (let r = 0; r < allRows.length; r++) {
            const detailFor = allRows[r].getAttribute('data-detail-for');
            if (detailFor) {
                detailMap[detailFor] = allRows[r];
            } else {
                sortableRows.push(allRows[r]);
            }
        }

        sortableRows.sort(function(a, b) {
            const aCell = a.cells[colIndex];
            const bCell = b.cells[colIndex];
            if (!aCell || !bCell) return 0;
            const aVal = getSortValue(aCell);
            const bVal = getSortValue(bCell);

            const aNum = parseFloat(aVal);
            const bNum = parseFloat(bVal);
            if (!isNaN(aNum) && !isNaN(bNum)) {
                return dir === 'asc' ? aNum - bNum : bNum - aNum;
            }
            const cmp = aVal.localeCompare(bVal, undefined, { sensitivity: 'base' });
            return dir === 'asc' ? cmp : -cmp;
        });

        for (let j = 0; j < sortableRows.length; j++) {
            tbody.appendChild(sortableRows[j]);
            const eventId = sortableRows[j].getAttribute('data-event-id');
            if (eventId && detailMap[eventId]) {
                tbody.appendChild(detailMap[eventId]);
            }
        }
    }

    function getSortValue(cell) {
        if (cell.title) return cell.title.toLowerCase();
        const sv = cell.getAttribute('data-sort-value');
        if (sv) return sv.toLowerCase();
        return (cell.textContent || '').trim().toLowerCase();
    }

    function initListPage(entityType, searchInputId, opts) {
        opts = opts || {};
        const searchAttr = opts.searchAttr || 'data-name';

        initTableSort();

        app.events.on('input', '#' + searchInputId, function(e, el) {
            let debounceTimer = el._debounceTimer || null;
            clearTimeout(debounceTimer);
            el._debounceTimer = setTimeout(function() {
                const q = el.value.toLowerCase().trim();
                const rows = document.querySelectorAll('.data-table tbody tr');
                for (let i = 0; i < rows.length; i++) {
                    const searchVal = rows[i].getAttribute(searchAttr) || rows[i].textContent.toLowerCase();
                    rows[i].style.display = (!q || searchVal.indexOf(q) !== -1) ? '' : 'none';
                }
            }, 200);
        });

        app.events.on('click', '[data-action="delete"]', function(e, deleteBtn) {
            shared.closeAllMenus();
            const entityId = deleteBtn.getAttribute('data-entity-id');
            const deleteEntityType = deleteBtn.getAttribute('data-entity-type') || entityType;
            showDeleteDialog(deleteEntityType, entityId, opts);
        }, { exclusive: true });

        app.events.on('click', '[data-confirm-delete]', function(e, confirmBtn) {
            const deleteId = confirmBtn.getAttribute('data-confirm-delete');
            performDelete(entityType, deleteId, confirmBtn, opts);
        }, { exclusive: true });

        app.events.on('change', '[data-action="toggle"]', function(e, toggle) {
            const id = toggle.getAttribute('data-entity-id');
            const toggleType = toggle.getAttribute('data-entity-type') || entityType;
            const enabled = toggle.checked;
            const apiPath = opts.toggleApiPath
                ? opts.toggleApiPath(id)
                : '/' + toggleType + 's/' + encodeURIComponent(id);

            const row = toggle.closest('tr');
            if (row) {
                row.setAttribute('data-enabled', enabled ? 'enabled' : 'disabled');
                const statusCell = row.querySelector('.badge-green, .badge-gray');
                if (statusCell && (statusCell.textContent === 'Active' || statusCell.textContent === 'Disabled')) {
                    statusCell.className = 'badge ' + (enabled ? 'badge-green' : 'badge-gray');
                    statusCell.textContent = enabled ? 'Active' : 'Disabled';
                }
            }

            const statsMap = { 'plugin': 'stats-plugins', 'agent': 'stats-agents', 'mcp-server': 'stats-mcp', 'skill': 'stats-skills' };
            const statsEl = document.getElementById(statsMap[toggleType]);
            const statVal = statsEl ? statsEl.querySelector('.config-stat-value') : null;
            if (statVal) {
                const parts = statVal.textContent.split('/');
                if (parts.length === 2) {
                    const cur = parseInt(parts[0], 10) + (enabled ? 1 : -1);
                    statVal.textContent = cur + '/' + parts[1];
                }
            }

            app.api(apiPath, {
                method: 'PUT',
                body: JSON.stringify({ enabled: enabled })
            }).then(function() {
                app.Toast.show(toggleType + ' ' + (enabled ? 'enabled' : 'disabled'), 'success');
            }).catch(function(err) {
                toggle.checked = !enabled;
                if (row) {
                    row.setAttribute('data-enabled', enabled ? 'disabled' : 'enabled');
                    const badge = row.querySelector('.badge-green, .badge-gray');
                    if (badge && (badge.textContent === 'Active' || badge.textContent === 'Disabled')) {
                        badge.className = 'badge ' + (enabled ? 'badge-gray' : 'badge-green');
                        badge.textContent = enabled ? 'Disabled' : 'Active';
                    }
                }
                if (statVal) {
                    const rparts = statVal.textContent.split('/');
                    if (rparts.length === 2) {
                        const rcur = parseInt(rparts[0], 10) + (enabled ? -1 : 1);
                        statVal.textContent = rcur + '/' + rparts[1];
                    }
                }
                app.Toast.show(err.message || 'Failed to update ' + toggleType, 'error');
            });
        });
    }

    function showDeleteDialog(entityType, entityId, opts) {
        const title = 'Delete ' + entityType.charAt(0).toUpperCase() + entityType.slice(1) + '?';
        shared.showDeleteConfirmDialog(title, entityId);
    }

    function performDelete(entityType, entityId, confirmBtn, opts) {
        confirmBtn.disabled = true;
        confirmBtn.textContent = 'Deleting...';
        const apiPath = opts.deleteApiPath
            ? opts.deleteApiPath(entityId)
            : '/' + entityType + 's/' + encodeURIComponent(entityId);
        app.api(apiPath, { method: 'DELETE' }).then(function() {
            app.Toast.show(entityType + ' deleted', 'success');
            shared.closeDeleteConfirm();
            window.location.reload();
        }).catch(function(err) {
            app.Toast.show(err.message || 'Failed to delete ' + entityType, 'error');
            confirmBtn.disabled = false;
            confirmBtn.textContent = 'Delete';
        });
    }

    function initEditForm(formId, opts) {
        opts = opts || {};
        const form = document.getElementById(formId);
        if (!form) return;

        const apiPath = form.getAttribute('data-api-path') || '';
        const entity = form.getAttribute('data-entity') || 'item';
        const idField = form.getAttribute('data-id-field') || 'id';
        const listPath = form.getAttribute('data-list-path') || '/';
        const existingId = form.querySelector('[name="' + idField + '"]');
        const isEdit = existingId && existingId.readOnly && existingId.value;

        form.addEventListener('submit', function(e) {
            e.preventDefault();
            const formData = new FormData(form);
            const body = opts.buildBody ? opts.buildBody(form, formData) : formDataToObject(formData);

            let url, method;
            if (isEdit) {
                url = apiPath + encodeURIComponent(existingId.value);
                method = 'PUT';
            } else {
                url = apiPath.replace(/\/$/, '');
                method = 'POST';
            }

            const submitBtn = form.querySelector('[type="submit"]');
            if (submitBtn) { submitBtn.disabled = true; submitBtn.textContent = 'Saving...'; }

            app.api(url, { method: method, body: JSON.stringify(body) })
                .then(function() {
                    app.Toast.show(entity + ' saved!', 'success');
                    window.location.href = app.BASE + listPath;
                })
                .catch(function(err) {
                    app.Toast.show(err.message || 'Failed to save ' + entity, 'error');
                    if (submitBtn) { submitBtn.disabled = false; submitBtn.textContent = isEdit ? 'Save Changes' : 'Create'; }
                });
        });
    }

    function formDataToObject(formData) {
        const obj = {};
        formData.forEach(function(value, key) {
            if (key === 'tags') {
                obj[key] = value.split(',').map(function(t) { return t.trim(); }).filter(Boolean);
            } else {
                obj[key] = value;
            }
        });
        return obj;
    }

    document.addEventListener('DOMContentLoaded', function() {
        app.events.init();
        initTableSort();
    });

    app.interactions = {
        initListPage: initListPage,
        initEditForm: initEditForm,
        initTableSort: initTableSort
    };
})(window.AdminApp);

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

(function(app) {
    'use strict';

    const OrgCommon = {

        initExpandRows: function(tableSelector, renderCallback) {
            const table = document.querySelector(tableSelector);
            if (!table) return;

            table.addEventListener('click', function(e) {
                if (e.target.closest('[data-no-row-click]') ||
                    e.target.closest('.actions-menu') ||
                    e.target.closest('.btn') ||
                    e.target.closest('a') ||
                    e.target.closest('input') ||
                    e.target.closest('.toggle-switch')) {
                    return;
                }

                const row = e.target.closest('tr.clickable-row');
                if (!row) return;

                const detailRow = row.nextElementSibling;
                if (!detailRow || !detailRow.classList.contains('detail-row')) return;

                OrgCommon.handleRowClick(row, detailRow);

                if (renderCallback && detailRow.classList.contains('visible')) {
                    renderCallback(row, detailRow);
                }
            });
        },

        handleRowClick: function(row, detailRow) {
            const isVisible = detailRow.classList.contains('visible');

            const table = row.closest('table');
            if (table) {
                table.querySelectorAll('tr.detail-row.visible').forEach(function(r) {
                    if (r !== detailRow) {
                        r.classList.remove('visible');
                        const prevRow = r.previousElementSibling;
                        if (prevRow) {
                            const indicator = prevRow.querySelector('.expand-indicator');
                            if (indicator) indicator.classList.remove('expanded');
                        }
                    }
                });
            }

            if (!isVisible) {
                detailRow.classList.add('visible');
                const expandIndicator = row.querySelector('.expand-indicator');
                if (expandIndicator) expandIndicator.classList.add('expanded');
            } else {
                detailRow.classList.remove('visible');
                const collapseIndicator = row.querySelector('.expand-indicator');
                if (collapseIndicator) collapseIndicator.classList.remove('expanded');
            }
        },

        initSidePanel: function(panelId) {
            const panel = document.getElementById(panelId);
            if (!panel) return null;

            const overlayId = panel.getAttribute('data-overlay') || (panelId + '-overlay');
            const overlay = document.getElementById(overlayId);
            const closeBtn = panel.querySelector('[data-panel-close]');

            const api = {
                open: function() {
                    panel.classList.add('open');
                    if (overlay) overlay.classList.add('active');
                },
                close: function() {
                    panel.classList.remove('open');
                    if (overlay) overlay.classList.remove('active');
                },
                setTitle: function(text) {
                    const title = panel.querySelector('[data-panel-title]');
                    if (title) title.textContent = text;
                },
                setBody: function(html) {
                    const body = panel.querySelector('[data-panel-body]');
                    if (body) body.innerHTML = html;
                },
                setFooter: function(html) {
                    const footer = panel.querySelector('[data-panel-footer]');
                    if (footer) footer.innerHTML = html;
                },
                panel: panel
            };

            if (closeBtn) closeBtn.addEventListener('click', api.close);
            if (overlay) overlay.addEventListener('click', api.close);

            return api;
        },

        initAssignPanel: function(config) {
            const panelApi = OrgCommon.initSidePanel(config.panelId);
            if (!panelApi) return null;

            return {
                open: function(entityId, entityName, currentPluginIds) {
                    panelApi.setTitle('Assign ' + (entityName || entityId));

                    const allPlugins = config.allPlugins || [];
                    const currentSet = {};
                    (currentPluginIds || []).forEach(function(id) { currentSet[id] = true; });

                    let html = '<div class="assign-panel-checklist">';
                    if (allPlugins.length === 0) {
                        html += '<p style="color:var(--text-tertiary);font-size:var(--text-sm)">No plugins available.</p>';
                    } else {
                        allPlugins.forEach(function(p) {
                            const checked = currentSet[p.id] ? ' checked' : '';
                            html += '<label class="acl-checkbox-row">' +
                                '<input type="checkbox" name="plugin_id" value="' + app.escapeHtml(p.id) + '"' + checked + '>' +
                                '<span class="acl-checkbox-label">' + app.escapeHtml(p.name || p.id) + '</span>' +
                                '</label>';
                        });
                    }
                    html += '</div>';
                    panelApi.setBody(html);

                    panelApi.setFooter(
                        '<button class="btn btn-secondary" data-panel-close>Cancel</button> ' +
                        '<button class="btn btn-primary" data-assign-save data-entity-id="' + app.escapeHtml(entityId) + '">Save</button>'
                    );

                    const footer = panelApi.panel.querySelector('[data-panel-footer]');
                    if (footer) {
                        const cancelBtn = footer.querySelector('[data-panel-close]');
                        if (cancelBtn) cancelBtn.addEventListener('click', panelApi.close);
                    }

                    panelApi.open();
                },
                close: panelApi.close,
                panel: panelApi
            };
        },

        initEditPanel: function(config) {
            // config: { panelId, fields, apiBasePath, idField, entityLabel }
            // fields: [{name, label, type:'text'|'textarea', rows, required}]
            const panelApi = OrgCommon.initSidePanel(config.panelId);
            if (!panelApi) return null;
            var currentEntityId = null;

            function buildForm(entityData) {
                var html = '<form class="edit-panel-form">';
                (config.fields || []).forEach(function(f) {
                    var val = entityData[f.name] || '';
                    if (Array.isArray(val)) val = val.join(', ');
                    html += '<div class="form-group">';
                    html += '<label class="form-label">' + app.escapeHtml(f.label) + '</label>';
                    if (f.type === 'textarea') {
                        html += '<textarea class="form-control" name="' + f.name + '" rows="' + (f.rows || 10) + '">' + app.escapeHtml(val) + '</textarea>';
                    } else {
                        html += '<input type="text" class="form-control" name="' + f.name + '" value="' + app.escapeHtml(val) + '"' + (f.required ? ' required' : '') + '>';
                    }
                    html += '</div>';
                });
                html += '</form>';
                return html;
            }

            function collectFormData() {
                var form = panelApi.panel.querySelector('.edit-panel-form');
                if (!form) return {};
                var body = {};
                (config.fields || []).forEach(function(f) {
                    var el = form.querySelector('[name="' + f.name + '"]');
                    if (!el) return;
                    var val = el.value;
                    if (f.name === 'tags') {
                        body[f.name] = val.split(',').map(function(t) { return t.trim(); }).filter(Boolean);
                    } else {
                        body[f.name] = val;
                    }
                });
                return body;
            }

            // Wire up save button click (delegated)
            document.addEventListener('click', function(e) {
                var btn = e.target.closest('[data-edit-save]');
                if (!btn) return;
                btn.disabled = true;
                btn.textContent = 'Saving...';
                var body = collectFormData();
                var url = config.apiBasePath + encodeURIComponent(currentEntityId);
                fetch(url, {
                    method: 'PUT',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(body)
                }).then(function(res) {
                    if (res.ok) {
                        app.Toast.show((config.entityLabel || 'Item') + ' updated', 'success');
                        panelApi.close();
                        setTimeout(function() { window.location.reload(); }, 500);
                    } else {
                        res.text().then(function(t) {
                            app.Toast.show('Failed to save: ' + t, 'error');
                        });
                        btn.disabled = false;
                        btn.textContent = 'Save';
                    }
                }).catch(function() {
                    app.Toast.show('Failed to save', 'error');
                    btn.disabled = false;
                    btn.textContent = 'Save';
                });
            });

            return {
                open: function(entityId, entityData) {
                    currentEntityId = entityId;
                    panelApi.setTitle('Edit ' + app.escapeHtml(entityData.name || entityId));
                    panelApi.setBody(buildForm(entityData));
                    panelApi.setFooter(
                        '<button class="btn btn-secondary" data-panel-close>Cancel</button> ' +
                        '<button class="btn btn-primary" data-edit-save>Save</button>'
                    );
                    var footer = panelApi.panel.querySelector('[data-panel-footer]');
                    if (footer) {
                        var cancelBtn = footer.querySelector('[data-panel-close]');
                        if (cancelBtn) cancelBtn.addEventListener('click', panelApi.close);
                    }
                    panelApi.open();
                },
                close: panelApi.close
            };
        },

        initBulkActions: function(tableSelector, barId) {
            const table = document.querySelector(tableSelector);
            if (!table) return null;

            let selected = {};

            function updateCount() {
                var count = Object.keys(selected).length;
                var countEl = document.querySelector('[data-bulk-count]');
                if (countEl) countEl.textContent = count;
                var bar = document.getElementById(barId);
                if (bar) bar.style.display = count > 0 ? 'flex' : 'none';
            }

            table.addEventListener('change', function(e) {
                if (e.target.classList.contains('bulk-select-all')) {
                    const checked = e.target.checked;
                    table.querySelectorAll('.bulk-checkbox').forEach(function(cb) {
                        cb.checked = checked;
                        const id = cb.getAttribute('data-entity-id');
                        if (checked) {
                            selected[id] = true;
                        } else {
                            delete selected[id];
                        }
                    });
                    updateCount();
                    return;
                }

                if (e.target.classList.contains('bulk-checkbox')) {
                    const id = e.target.getAttribute('data-entity-id');
                    if (e.target.checked) {
                        selected[id] = true;
                    } else {
                        delete selected[id];
                    }
                    updateCount();
                }
            });

            return {
                getSelected: function() { return Object.keys(selected); },
                clear: function() {
                    selected = {};
                    table.querySelectorAll('.bulk-checkbox, .bulk-select-all').forEach(function(cb) {
                        cb.checked = false;
                    });
                    updateCount();
                }
            };
        },

        formatJson: function(data) {
            if (typeof data === 'string') {
                try { data = JSON.parse(data); } catch (e) { return app.escapeHtml(data); }
            }
            return '<pre class="json-view">' + app.escapeHtml(JSON.stringify(data, null, 2)) + '</pre>';
        },

        renderRoleBadges: function(roles) {
            if (!roles || !roles.length) {
                return '<span class="badge badge-gray">All</span>';
            }
            const assigned = roles.filter(function(r) { return r.assigned; });
            if (!assigned.length) {
                return '<span class="badge badge-gray">All</span>';
            }
            return assigned.map(function(r) {
                return '<span class="badge badge-blue">' + app.escapeHtml(r.name) + '</span>';
            }).join(' ');
        },

        renderDeptBadges: function(departments) {
            if (!departments || !departments.length) {
                return '<span class="badge badge-gray">None</span>';
            }
            const assigned = departments.filter(function(d) { return d.assigned; });
            if (!assigned.length) {
                return '<span class="badge badge-gray">None</span>';
            }
            return assigned.map(function(d) {
                const cls = d.default_included ? 'badge-yellow' : 'badge-green';
                return '<span class="badge ' + cls + '">' + app.escapeHtml(d.name) + '</span>';
            }).join(' ');
        },

        renderPluginBadges: function(plugins) {
            if (!plugins || !plugins.length) {
                return '<span class="badge badge-gray">None</span>';
            }
            return plugins.map(function(p) {
                const name = typeof p === 'string' ? p : (p.name || p.id || p);
                return '<span class="badge badge-purple">' + app.escapeHtml(name) + '</span>';
            }).join(' ');
        },

        initFilters: function(searchInputId, tableSelector, filters) {
            var table = document.querySelector(tableSelector);
            if (!table) return;

            function applyFilters() {
                var searchInput = document.getElementById(searchInputId);
                var q = (searchInput ? searchInput.value : '').toLowerCase().trim();
                var filterValues = filters.map(function(f) {
                    var sel = document.getElementById(f.selectId);
                    return { attr: f.dataAttr, value: sel ? sel.value : '' };
                });

                table.querySelectorAll('tbody tr.clickable-row').forEach(function(row) {
                    var matchSearch = !q ||
                        (row.getAttribute('data-name') || '').indexOf(q) !== -1 ||
                        (row.getAttribute('data-skill-id') || row.getAttribute('data-agent-id') || '').toLowerCase().indexOf(q) !== -1 ||
                        (row.getAttribute('data-description') || '').indexOf(q) !== -1;

                    var matchFilters = filterValues.every(function(fv) {
                        if (!fv.value) return true;
                        var rowVal = row.getAttribute(fv.attr) || '';
                        return rowVal.indexOf(fv.value) !== -1;
                    });

                    var match = matchSearch && matchFilters;
                    row.style.display = match ? '' : 'none';
                    var detail = row.nextElementSibling;
                    if (detail && detail.classList.contains('detail-row')) {
                        if (!match) { detail.style.display = 'none'; detail.classList.remove('visible'); }
                        else { detail.style.display = ''; }
                    }
                });
            }

            filters.forEach(function(f) {
                var sel = document.getElementById(f.selectId);
                if (sel) sel.addEventListener('change', applyFilters);
            });

            var searchTimer = null;
            var searchInput = document.getElementById(searchInputId);
            if (searchInput) {
                searchInput.addEventListener('input', function() {
                    clearTimeout(searchTimer);
                    searchTimer = setTimeout(applyFilters, 200);
                });
            }

            return { apply: applyFilters };
        },

        formatTimeAgo: function(isoString) {
            if (!isoString) return '--';
            var date = new Date(isoString);
            if (isNaN(date.getTime())) return '--';
            var now = new Date();
            var diff = Math.floor((now - date) / 1000);
            if (diff < 60) return 'just now';
            if (diff < 3600) return Math.floor(diff / 60) + 'm ago';
            if (diff < 86400) return Math.floor(diff / 3600) + 'h ago';
            if (diff < 2592000) return Math.floor(diff / 86400) + 'd ago';
            return date.toLocaleDateString();
        },

        initTimeAgo: function() {
            document.querySelectorAll('.metadata-timestamp').forEach(function(el) {
                var iso = el.getAttribute('title') || el.textContent.trim();
                if (iso && iso !== '--') {
                    el.textContent = OrgCommon.formatTimeAgo(iso);
                    el.setAttribute('title', new Date(iso).toLocaleString());
                }
            });
        }
    };

    app.OrgCommon = OrgCommon;
})(window.AdminApp = window.AdminApp || {});

(function(app) {
    'use strict';

    function openCreatePanel() {
        const overlay = document.getElementById('create-user-overlay');
        const panel = document.getElementById('create-user-panel');
        if (!overlay || !panel) return;
        overlay.classList.add('open');
        panel.classList.add('open');
        const first = panel.querySelector('input');
        if (first) setTimeout(function() { first.focus(); }, 350);
    }
    function closeCreatePanel() {
        const overlay = document.getElementById('create-user-overlay');
        const panel = document.getElementById('create-user-panel');
        if (panel) panel.classList.remove('open');
        if (overlay) overlay.classList.remove('open');
    }
    function resetForm() {
        const fields = ['new-user-id', 'new-user-name', 'new-user-email'];
        for (let i = 0; i < fields.length; i++) {
            const el = document.getElementById(fields[i]);
            if (el) el.value = '';
        }
        const dept = document.getElementById('new-user-dept');
        if (dept) dept.value = '';
        const boxes = document.querySelectorAll('#create-user-panel input[name="roles"]');
        for (let j = 0; j < boxes.length; j++) {
            boxes[j].checked = false;
        }
    }
    function bindCreatePanelEvents(refreshFn) {
        document.addEventListener('click', async function(e) {
            if (e.target.id === 'create-user-overlay') {
                closeCreatePanel();
                return;
            }
            const closeBtn = e.target.closest('#create-user-panel .panel-close');
            if (closeBtn) {
                closeCreatePanel();
                return;
            }
            const cancelBtn = e.target.closest('#create-user-panel [data-action="cancel"]');
            if (cancelBtn) {
                closeCreatePanel();
                return;
            }
            const saveBtn = e.target.closest('#create-user-panel [data-action="save"]');
            if (saveBtn) {
                const userId = document.getElementById('new-user-id').value.trim();
                const displayName = document.getElementById('new-user-name').value.trim();
                const email = document.getElementById('new-user-email').value.trim();
                const deptVal = document.getElementById('new-user-dept').value;
                const roleBoxes = document.querySelectorAll('#create-user-panel input[name="roles"]:checked');
                const roles = Array.from(roleBoxes).map(function(cb) { return cb.value; });
                if (!userId) {
                    app.Toast.show('User ID is required', 'error');
                    return;
                }
                try {
                    await app.api('/users', {
                        method: 'POST',
                        body: JSON.stringify({
                            user_id: userId,
                            display_name: displayName || userId,
                            email: email,
                            department: deptVal,
                            roles: roles
                        })
                    });
                    app.Toast.show('User created', 'success');
                    closeCreatePanel();
                    resetForm();
                    refreshFn();
                } catch (err) {
                    app.Toast.show(err.message || 'Failed to create user', 'error');
                }
            }
        });
    }
    app.usersPanel = {
        open: openCreatePanel,
        close: closeCreatePanel,
        bindEvents: bindCreatePanelEvents
    };
})(window.AdminApp);

(function(app) {
    'use strict';

    const showConfirmDialog = app.shared.showConfirmDialog;
    let activePopupId = null;

    function closeAllPopups() {
        const portal = document.getElementById('user-actions-popup');
        if (portal) {
            portal.classList.remove('open');
            activePopupId = null;
        }
        const triggers = document.querySelectorAll('.btn-actions-trigger.active');
        for (let i = 0; i < triggers.length; i++) {
            triggers[i].classList.remove('active');
            triggers[i].setAttribute('aria-expanded', 'false');
        }
    }

    function getOrCreatePortal() {
        let portal = document.getElementById('user-actions-popup');
        if (!portal) {
            portal = document.createElement('div');
            portal.id = 'user-actions-popup';
            portal.className = 'actions-popup';
            portal.setAttribute('role', 'menu');
            document.body.appendChild(portal);
        }
        return portal;
    }

    function positionPopup(portal, trigger) {
        const rect = trigger.getBoundingClientRect();
        const popupH = portal.offsetHeight || 120;
        const spaceBelow = window.innerHeight - rect.bottom;
        portal.style.top = (spaceBelow < popupH)
            ? (rect.top - popupH) + 'px'
            : (rect.bottom + 4) + 'px';
        const popupW = portal.offsetWidth || 180;
        if (window.innerWidth < popupW + 16) {
            portal.style.left = '8px';
            portal.style.right = '8px';
        } else {
            portal.style.right = (window.innerWidth - rect.right) + 'px';
            portal.style.left = '';
        }
    }

    app.usersInteractions = function() {
        app.events.on('click', '.btn-actions-trigger', function(e, trigger) {
            e.stopPropagation();
            const userId = trigger.dataset.userId;
            const portal = getOrCreatePortal();
            const isOpen = portal.classList.contains('open') && activePopupId === userId;
            closeAllPopups();
            if (isOpen) return;

            const row = trigger.closest('tr');
            const isActive = row && row.querySelector('.badge-green') !== null;
            const toggleLabel = isActive ? 'Deactivate' : 'Activate';
            const toggleIcon = isActive ? '&#10006;' : '&#10004;';
            const toggleClass = isActive ? ' actions-popup-item--danger' : '';

            portal.innerHTML =
                '<button class="actions-popup-item" data-action="edit" data-user-id="' + userId + '"><span class="popup-icon">&#9998;</span> Edit User</button>' +
                '<div class="actions-popup-separator"></div>' +
                '<button class="actions-popup-item' + toggleClass + '" data-action="toggle" data-user-id="' + userId + '" data-is-active="' + isActive + '"><span class="popup-icon">' + toggleIcon + '</span> ' + toggleLabel + '</button>';

            activePopupId = userId;
            portal.classList.add('open');
            trigger.classList.add('active');
            trigger.setAttribute('aria-expanded', 'true');
            positionPopup(portal, trigger);

            portal.querySelectorAll('.actions-popup-item').forEach(function(item) {
                item.addEventListener('click', function(ev) {
                    ev.stopPropagation();
                    const action = item.dataset.action;
                    const itemUserId = item.dataset.userId;
                    closeAllPopups();
                    if (action === 'edit') {
                        window.location.href = app.BASE + '/user/?id=' + encodeURIComponent(itemUserId);
                    } else if (action === 'toggle') {
                        const currentlyActive = item.dataset.isActive === 'true';
                        if (currentlyActive) {
                            showConfirmDialog(
                                'Deactivate User?',
                                'This will prevent the user from accessing the system. You can reactivate them later.',
                                'Deactivate',
                                async function() {
                                    try {
                                        await app.api('/users/' + encodeURIComponent(itemUserId), {
                                            method: 'PUT',
                                            body: JSON.stringify({ is_active: false })
                                        });
                                        app.Toast.show('User deactivated', 'success');
                                        window.location.reload();
                                    } catch (err) {
                                        app.Toast.show(err.message || 'Failed to deactivate user', 'error');
                                    }
                                }
                            );
                        } else {
                            app.api('/users/' + encodeURIComponent(itemUserId), {
                                method: 'PUT',
                                body: JSON.stringify({ is_active: true })
                            }).then(function() {
                                app.Toast.show('User activated', 'success');
                                window.location.reload();
                            }).catch(function(err) {
                                app.Toast.show(err.message || 'Failed to activate user', 'error');
                            });
                        }
                    }
                });
            });
        });

        app.events.on('click', '*', function(e) {
            if (!e.target.closest('.btn-actions-trigger') && !e.target.closest('#user-actions-popup')) {
                closeAllPopups();
            }
        });

        const tableScroll = document.querySelector('.table-scroll');
        if (tableScroll) {
            tableScroll.addEventListener('scroll', closeAllPopups);
        }
    };
})(window.AdminApp);

(function(app) {
    'use strict';

    window.addEventListener('env-saved', function(e) {
        const pid = e.detail && e.detail.pluginId;
        if (!pid) return;
        const containerId = 'env-status-' + pid;
        const container = document.getElementById(containerId);
        if (container) {
            container.removeAttribute('data-loaded');
            container.innerHTML = '<div style="padding:var(--space-4);color:var(--text-tertiary);font-size:var(--text-sm)">Refreshing...</div>';
        }
    });
    app.pluginDetails = { render: function() { return ''; } };
})(window.AdminApp);

(function(app) {
    'use strict';

    const escapeHtml = app.escapeHtml;
    let overlay = null;
    let currentSkillId = null;
    let currentSkillName = '';
    let files = [];
    let selectedFile = null;

    const categoryLabels = {
        script: 'Scripts',
        reference: 'References',
        template: 'Templates',
        diagnostic: 'Diagnostics',
        data: 'Data',
        config: 'Config',
        asset: 'Assets'
    };

    const categoryOrder = ['script', 'reference', 'template', 'diagnostic', 'data', 'config', 'asset'];

    function groupByCategory(fileList) {
        const groups = {};
        fileList.forEach(function(f) {
            const cat = f.category || 'config';
            if (!groups[cat]) groups[cat] = [];
            groups[cat].push(f);
        });
        return groups;
    }

    function renderFileList() {
        if (!files.length) {
            return '<div class="empty-state" style="padding:var(--space-6)"><p>No files found for this skill.</p>' +
                '<p style="font-size:var(--text-sm);color:var(--text-tertiary);margin-top:var(--space-2)">Click "Sync Files" to scan the filesystem.</p></div>';
        }
        const groups = groupByCategory(files);
        let html = '';
        categoryOrder.forEach(function(cat) {
            const group = groups[cat];
            if (!group || !group.length) return;
            html += '<div style="margin-bottom:var(--space-3)">' +
                '<div class="skill-file-category">' +
                escapeHtml(categoryLabels[cat] || cat) + ' (' + group.length + ')' +
                '</div>';
            group.forEach(function(f) {
                const isSelected = selectedFile && selectedFile.id === f.id;
                html += '<div class="skill-file-item' + (isSelected ? ' selected' : '') + '" data-file-id="' + escapeHtml(f.id) + '">' +
                    '<span class="skill-file-name">' + escapeHtml(f.file_path) + '</span>' +
                    (f.language ? '<span class="skill-file-lang">' + escapeHtml(f.language) + '</span>' : '') +
                '</div>';
            });
            html += '</div>';
        });
        return html;
    }

    function validateContent(content, lang) {
        if (!content || !lang) return null;
        lang = lang.toLowerCase();
        try {
            if (lang === 'json') {
                JSON.parse(content);
                return null;
            }
            if (lang === 'yaml' || lang === 'yml') {
                const lines = content.split('\n');
                for (let i = 0; i < lines.length; i++) {
                    const line = lines[i];
                    if (line.trim() === '' || line.trim().charAt(0) === '#') continue;
                    if (/\t/.test(line.match(/^(\s*)/)[1])) {
                        return 'Line ' + (i + 1) + ': tabs not allowed in YAML, use spaces';
                    }
                }
                return null;
            }
            if (lang === 'python') {
                return checkBrackets(content, [['(', ')'], ['[', ']'], ['{', '}']]);
            }
            if (lang === 'bash' || lang === 'shell') {
                return checkBrackets(content, [['(', ')'], ['[', ']'], ['{', '}']]);
            }
        } catch (e) {
            return e.message;
        }
        return null;
    }

    function checkBrackets(content, pairs) {
        const stack = [];
        const closeMap = {};
        const openSet = {};
        pairs.forEach(function(p) { closeMap[p[1]] = p[0]; openSet[p[0]] = p[1]; });
        let inStr = false;
        let strChar = '';
        let escaped = false;
        for (let i = 0; i < content.length; i++) {
            const ch = content[i];
            if (escaped) { escaped = false; continue; }
            if (ch === '\\') { escaped = true; continue; }
            if (inStr) {
                if (ch === strChar) inStr = false;
                continue;
            }
            if (ch === '"' || ch === "'") { inStr = true; strChar = ch; continue; }
            if (ch === '#' && content.charAt(i - 1) !== '$') {
                const nl = content.indexOf('\n', i);
                if (nl === -1) break;
                i = nl;
                continue;
            }
            if (openSet[ch]) { stack.push(ch); }
            else if (closeMap[ch]) {
                if (!stack.length) {
                    const line = content.substring(0, i).split('\n').length;
                    return 'Line ' + line + ': unexpected \'' + ch + '\'';
                }
                const top = stack.pop();
                if (top !== closeMap[ch]) {
                    const line2 = content.substring(0, i).split('\n').length;
                    return 'Line ' + line2 + ': expected \'' + openSet[top] + '\' but found \'' + ch + '\'';
                }
            }
        }
        if (stack.length) {
            return 'Unclosed \'' + stack[stack.length - 1] + '\'';
        }
        return null;
    }

    function renderEditor() {
        if (!selectedFile) {
            return '<div style="display:flex;align-items:center;justify-content:center;height:100%;color:var(--text-tertiary);font-size:var(--text-sm)">Select a file to view its contents</div>';
        }
        return '<div style="display:flex;flex-direction:column;height:100%">' +
            '<div style="display:flex;align-items:center;gap:var(--space-2);padding:var(--space-2) var(--space-3);border-bottom:1px solid var(--border-subtle);flex-shrink:0">' +
                '<span style="font-family:monospace;font-size:var(--text-sm);font-weight:600">' + escapeHtml(selectedFile.file_path) + '</span>' +
                '<span class="badge badge-blue" style="font-size:var(--text-xs)">' + escapeHtml(selectedFile.language || 'text') + '</span>' +
                (selectedFile.executable ? '<span class="badge badge-green" style="font-size:var(--text-xs)">executable</span>' : '') +
                '<span style="margin-left:auto;font-size:var(--text-xs);color:var(--text-tertiary)">' + selectedFile.size_bytes + ' bytes</span>' +
            '</div>' +
            '<textarea id="skill-file-editor" style="flex:1;width:100%;border:none;padding:var(--space-3);font-family:monospace;font-size:var(--text-sm);line-height:1.5;resize:none;background:var(--bg-surface);color:var(--text-primary);outline:none;box-sizing:border-box">' +
                escapeHtml(selectedFile.content || '') +
            '</textarea>' +
            '<div style="display:flex;align-items:center;padding:var(--space-2) var(--space-3);border-top:1px solid var(--border-subtle);flex-shrink:0">' +
                '<span id="skill-file-validation" style="font-size:var(--text-xs);flex:1"></span>' +
                '<button class="btn btn-primary btn-sm" id="skill-file-save" style="font-size:var(--text-xs)">Save</button>' +
            '</div>' +
        '</div>';
    }

    function renderModal() {
        return '<div style="display:flex;flex-direction:column;height:100%">' +
            '<div style="display:flex;align-items:center;padding:var(--space-4);border-bottom:1px solid var(--border-subtle);flex-shrink:0">' +
                '<h2 style="margin:0;font-size:var(--text-lg);font-weight:600;color:var(--text-primary)">' + escapeHtml(currentSkillName) + ' - Files</h2>' +
                '<div style="margin-left:auto;display:flex;gap:var(--space-2)">' +
                    '<button class="btn btn-secondary btn-sm" id="skill-files-sync" style="font-size:var(--text-xs)">Sync Files</button>' +
                    '<button class="btn btn-secondary btn-sm" id="skill-files-close" style="font-size:var(--text-xs)">Close</button>' +
                '</div>' +
            '</div>' +
            '<div style="display:flex;flex:1;min-height:0">' +
                '<div id="skill-files-list" style="width:280px;overflow-y:auto;border-right:1px solid var(--border-subtle);padding:var(--space-2) 0">' +
                    renderFileList() +
                '</div>' +
                '<div id="skill-files-editor" style="flex:1;min-width:0;overflow:hidden">' +
                    renderEditor() +
                '</div>' +
            '</div>' +
        '</div>';
    }

    function updatePanel() {
        const panel = overlay && overlay.querySelector('.skill-files-panel');
        if (panel) panel.innerHTML = renderModal();
        bindEvents();
    }

    function runValidation() {
        if (!overlay || !selectedFile) return;
        const editor = overlay.querySelector('#skill-file-editor');
        const badge = overlay.querySelector('#skill-file-validation');
        if (!editor || !badge) return;
        const err = validateContent(editor.value, selectedFile.language);
        if (err) {
            badge.textContent = err;
            badge.style.color = 'var(--danger)';
        } else {
            badge.textContent = '';
        }
    }

    function bindEditorValidation() {
        if (!overlay) return;
        const editor = overlay.querySelector('#skill-file-editor');
        if (editor) {
            editor.addEventListener('input', runValidation);
            runValidation();
        }
    }

    function handleFileClick(e) {
        const item = e.currentTarget;
        const fileId = item.getAttribute('data-file-id');
        selectedFile = files.find(function(f) { return f.id === fileId; }) || null;
        const listEl = overlay.querySelector('#skill-files-list');
        const editorEl = overlay.querySelector('#skill-files-editor');
        if (listEl) listEl.innerHTML = renderFileList();
        if (editorEl) editorEl.innerHTML = renderEditor();
        bindFileItems();
        const newSaveBtn = overlay.querySelector('#skill-file-save');
        if (newSaveBtn) newSaveBtn.addEventListener('click', handleSave);
        bindEditorValidation();
    }

    function bindFileItems() {
        if (!overlay) return;
        const fileItems = overlay.querySelectorAll('.skill-file-item');
        fileItems.forEach(function(item) {
            item.addEventListener('click', handleFileClick);
        });
    }

    function bindEvents() {
        if (!overlay) return;

        const closeBtn = overlay.querySelector('#skill-files-close');
        if (closeBtn) closeBtn.addEventListener('click', close);

        const syncBtn = overlay.querySelector('#skill-files-sync');
        if (syncBtn) syncBtn.addEventListener('click', handleSync);

        const saveBtn = overlay.querySelector('#skill-file-save');
        if (saveBtn) saveBtn.addEventListener('click', handleSave);

        bindFileItems();
        bindEditorValidation();
    }

    async function handleSync() {
        const syncBtn = overlay && overlay.querySelector('#skill-files-sync');
        if (syncBtn) {
            syncBtn.disabled = true;
            syncBtn.textContent = 'Syncing...';
        }
        try {
            const result = await app.api('/skills/sync-files', { method: 'POST' });
            app.Toast.show('Synced: ' + (result.created || 0) + ' created, ' + (result.updated || 0) + ' updated', 'success');
            await loadFiles();
            updatePanel();
        } catch (err) {
            app.Toast.show(err.message || 'Sync failed', 'error');
            if (syncBtn) {
                syncBtn.disabled = false;
                syncBtn.textContent = 'Sync Files';
            }
        }
    }

    async function handleSave() {
        if (!selectedFile) return;
        const editor = overlay && overlay.querySelector('#skill-file-editor');
        if (!editor) return;
        const content = editor.value;
        const err = validateContent(content, selectedFile.language);
        if (err) {
            app.Toast.show('Fix validation error before saving: ' + err, 'error');
            return;
        }
        const saveBtn = overlay.querySelector('#skill-file-save');
        if (saveBtn) {
            saveBtn.disabled = true;
            saveBtn.textContent = 'Saving...';
        }
        try {
            await app.api('/skills/' + encodeURIComponent(currentSkillId) + '/files/' + selectedFile.file_path, {
                method: 'PUT',
                body: JSON.stringify({ content: content }),
                headers: { 'Content-Type': 'application/json' }
            });
            selectedFile.content = content;
            selectedFile.size_bytes = new Blob([content]).size;
            app.Toast.show('File saved', 'success');
        } catch (err) {
            app.Toast.show(err.message || 'Save failed', 'error');
        } finally {
            if (saveBtn) {
                saveBtn.disabled = false;
                saveBtn.textContent = 'Save';
            }
        }
    }

    async function loadFiles() {
        try {
            files = await app.api('/skills/' + encodeURIComponent(currentSkillId) + '/files');
            if (!Array.isArray(files)) files = [];
        } catch (err) {
            files = [];
            app.Toast.show(err.message || 'Failed to load files', 'error');
        }
    }

    function close() {
        if (overlay) {
            overlay.remove();
            overlay = null;
        }
        currentSkillId = null;
        currentSkillName = '';
        files = [];
        selectedFile = null;
    }

    async function open(skillId, skillName) {
        close();
        currentSkillId = skillId;
        currentSkillName = skillName || skillId;

        overlay = document.createElement('div');
        overlay.className = 'confirm-overlay';
        overlay.style.cssText = 'display:flex;align-items:center;justify-content:center;z-index:1000';
        overlay.innerHTML = '<div class="skill-files-panel" style="background:var(--bg-surface);border-radius:var(--radius-lg);width:90vw;max-width:1100px;height:80vh;overflow:hidden;box-shadow:var(--shadow-lg);display:flex;flex-direction:column">' +
            '<div style="display:flex;align-items:center;justify-content:center;height:100%;color:var(--text-tertiary)">Loading files...</div>' +
        '</div>';
        document.body.appendChild(overlay);

        overlay.addEventListener('click', function(e) {
            if (e.target === overlay) close();
        });

        await loadFiles();
        updatePanel();
    }

    app.skillFiles = {
        open: open,
        close: close
    };
})(window.AdminApp);

(function(app) {
    'use strict';

    const escapeHtml = app.escapeHtml;
    let overlay = null;
    let currentPluginId = null;
    let currentPluginName = '';
    let envVars = [];
    let varDefs = [];

    function mergeDefsWithValues(defs, stored) {
        const merged = [];
        const storedMap = {};
        stored.forEach(function(v) { storedMap[v.var_name] = v; });

        defs.forEach(function(def) {
            const existing = storedMap[def.name];
            merged.push({
                name: def.name,
                description: def.description || '',
                required: def.required !== false,
                secret: def.secret || false,
                example: def.example || '',
                value: existing ? existing.var_value : '',
                fromDef: true
            });
            delete storedMap[def.name];
        });

        Object.keys(storedMap).forEach(function(key) {
            const s = storedMap[key];
            merged.push({
                name: s.var_name,
                description: '',
                required: false,
                secret: s.is_secret,
                example: '',
                value: s.var_value,
                fromDef: false
            });
        });

        return merged;
    }

    function renderVarList(vars) {
        if (!vars.length) {
            return '<div class="empty-state" style="padding:var(--space-6)"><p>No environment variables defined for this plugin.</p></div>';
        }
        let html = '';
        vars.forEach(function(v, i) {
            const inputType = v.secret ? 'password' : 'text';
            const placeholder = v.example ? v.example : '';
            const requiredBadge = v.required ? ' <span class="badge badge-red">required</span>' : '';
            const secretBadge = v.secret ? ' <span class="badge badge-gray">secret</span>' : '';
            html += '<div class="form-group">' +
                '<label>' + escapeHtml(v.name) + requiredBadge + secretBadge + '</label>' +
                (v.description ? '<p style="margin:0 0 var(--space-1);font-size:var(--text-xs);color:var(--text-tertiary)">' + escapeHtml(v.description) + '</p>' : '') +
                '<input type="' + inputType + '" class="plugin-env-input" data-var-index="' + i + '" data-var-name="' + escapeHtml(v.name) + '" data-is-secret="' + (v.secret ? '1' : '0') + '" ' +
                    'value="' + escapeHtml(v.value) + '" placeholder="' + escapeHtml(placeholder) + '">' +
            '</div>';
        });
        return html;
    }

    function renderModal(vars) {
        return '<h3 style="margin:0 0 var(--space-4)">' + escapeHtml(currentPluginName) + ' — Environment Variables</h3>' +
            '<div style="max-height:60vh;overflow-y:auto">' +
                renderVarList(vars) +
            '</div>' +
            '<div class="form-actions" style="display:flex;gap:var(--space-3);justify-content:flex-end;margin-top:var(--space-4)">' +
                '<button class="btn btn-secondary" id="plugin-env-close">Close</button>' +
                '<button class="btn btn-primary" id="plugin-env-save">Save</button>' +
            '</div>';
    }

    function updatePanel(vars) {
        const panel = overlay && overlay.querySelector('.confirm-dialog');
        if (panel) panel.innerHTML = renderModal(vars);
        bindEvents(vars);
    }

    function bindEvents(vars) {
        if (!overlay) return;

        const closeBtn = overlay.querySelector('#plugin-env-close');
        if (closeBtn) closeBtn.addEventListener('click', close);

        const saveBtn = overlay.querySelector('#plugin-env-save');
        if (saveBtn) saveBtn.addEventListener('click', function() { handleSave(vars); });
    }

    async function handleSave(vars) {
        const saveBtn = overlay && overlay.querySelector('#plugin-env-save');
        if (saveBtn) {
            saveBtn.disabled = true;
            saveBtn.textContent = 'Saving...';
        }
        try {
            const inputs = overlay.querySelectorAll('.plugin-env-input');
            const payload = [];
            inputs.forEach(function(input) {
                const name = input.getAttribute('data-var-name');
                const isSecret = input.getAttribute('data-is-secret') === '1';
                const value = input.value;
                if (isSecret && value === '••••••••') return;
                payload.push({ var_name: name, var_value: value, is_secret: isSecret });
            });
            await app.api('/plugins/' + encodeURIComponent(currentPluginId) + '/env', {
                method: 'PUT',
                body: JSON.stringify({ variables: payload }),
                headers: { 'Content-Type': 'application/json' }
            });
            window.dispatchEvent(new CustomEvent('env-saved', { detail: { pluginId: currentPluginId } }));
            if (saveBtn) {
                saveBtn.textContent = 'Saved';
                saveBtn.style.background = 'var(--success)';
                saveBtn.style.borderColor = 'var(--success)';
            }
            app.Toast.show('Environment variables saved', 'success');
            setTimeout(function() { close(); }, 600);
        } catch (err) {
            app.Toast.show(err.message || 'Save failed', 'error');
            if (saveBtn) {
                saveBtn.disabled = false;
                saveBtn.textContent = 'Save';
            }
        }
    }

    async function loadAndRender() {
        try {
            const data = await app.api('/plugins/' + encodeURIComponent(currentPluginId) + '/env');
            envVars = data.stored || [];
            varDefs = data.definitions || [];
            const merged = mergeDefsWithValues(varDefs, envVars);
            updatePanel(merged);
        } catch (err) {
            envVars = [];
            varDefs = [];
            app.Toast.show(err.message || 'Failed to load env vars', 'error');
            updatePanel([]);
        }
    }

    function close() {
        if (overlay) {
            overlay.remove();
            overlay = null;
        }
        currentPluginId = null;
        currentPluginName = '';
        envVars = [];
        varDefs = [];
    }

    async function open(pluginId, pluginName) {
        close();
        currentPluginId = pluginId;
        currentPluginName = pluginName || pluginId;

        overlay = document.createElement('div');
        overlay.className = 'confirm-overlay';
        overlay.innerHTML = '<div class="confirm-dialog" style="width:560px;max-width:90vw">' +
            '<div style="display:flex;align-items:center;justify-content:center;padding:var(--space-6);color:var(--text-tertiary)">Loading...</div>' +
        '</div>';
        document.body.appendChild(overlay);

        overlay.addEventListener('click', function(e) {
            if (e.target === overlay) close();
        });

        await loadAndRender();
    }

    app.pluginEnv = {
        open: open,
        close: close
    };
})(window.AdminApp);

(function(app) {
    'use strict';

    const pluginEnvValid = {};

    function updateGenerateButtons(pluginId) {
        const btns = document.querySelectorAll('[data-generate-plugin="' + pluginId + '"]');
        const envReady = pluginEnvValid[pluginId] === true;
        btns.forEach(function(btn) {
            if (!envReady) {
                btn.disabled = true;
                btn.title = pluginEnvValid[pluginId] === false
                    ? 'Configure required environment variables first'
                    : 'Checking environment variables...';
                btn.style.opacity = '0.4';
                btn.style.cursor = 'not-allowed';
            } else {
                btn.disabled = false;
                btn.title = '';
                btn.style.opacity = '';
                btn.style.cursor = '';
            }
        });
    }

    function showDeleteConfirm(pluginId) {
        const overlay = document.createElement('div');
        overlay.className = 'confirm-overlay';
        overlay.id = 'delete-confirm';
        overlay.innerHTML = '<div class="confirm-dialog">' +
            '<h3 style="margin:0 0 var(--space-3)">Delete Plugin?</h3>' +
            '<p style="margin:0 0 var(--space-2);color:var(--text-secondary);font-size:var(--text-sm)">You are about to delete <strong>' + app.escapeHtml(pluginId) + '</strong>.</p>' +
            '<p style="margin:0 0 var(--space-5);color:var(--text-secondary);font-size:var(--text-sm)">This will remove the plugin directory and all its configuration. This action cannot be undone.</p>' +
            '<div style="display:flex;gap:var(--space-3);justify-content:flex-end">' +
                '<button class="btn btn-secondary" data-confirm-cancel>Cancel</button>' +
                '<button class="btn btn-danger" data-confirm-delete="' + app.escapeHtml(pluginId) + '">Delete Plugin</button>' +
            '</div>' +
        '</div>';
        document.body.appendChild(overlay);
        overlay.addEventListener('click', async function(e) {
            if (e.target === overlay || e.target.closest('[data-confirm-cancel]')) {
                overlay.remove();
                return;
            }
            const confirmBtn = e.target.closest('[data-confirm-delete]');
            if (confirmBtn) {
                const pid = confirmBtn.getAttribute('data-confirm-delete');
                confirmBtn.disabled = true;
                confirmBtn.textContent = 'Deleting...';
                try {
                    await app.api('/plugins/' + encodeURIComponent(pid), { method: 'DELETE' });
                    app.Toast.show('Plugin deleted', 'success');
                    overlay.remove();
                    window.location.reload();
                } catch (err) {
                    app.Toast.show(err.message || 'Failed to delete plugin', 'error');
                    confirmBtn.disabled = false;
                    confirmBtn.textContent = 'Delete Plugin';
                }
            }
        });
    }

    async function handleExport(pluginId, btn, platform) {
        platform = platform || 'unix';
        if (pluginEnvValid[pluginId] !== true) {
            const msg = pluginEnvValid[pluginId] === false
                ? 'Configure required environment variables before generating'
                : 'Still checking environment variables, please try again';
            app.Toast.show(msg, 'error');
            return;
        }
        if (btn) { btn.disabled = true; btn.textContent = 'Generating...'; }
        try {
            const data = await app.api('/export?plugin=' + encodeURIComponent(pluginId) + '&platform=' + encodeURIComponent(platform));
            const JSZip = await app.shared.loadJSZip();
            const zip = new JSZip();
            const items = data.plugins || data.bundles || [];
            const bundle = items.find(function(b) { return b.id === pluginId || b.plugin_id === pluginId; });
            if (!bundle || !bundle.files) throw new Error('No files found in export');
            bundle.files.forEach(function(f) {
                const opts = f.executable ? { unixPermissions: '755' } : {};
                zip.file(f.path, f.content, opts);
            });
            const blob = await zip.generateAsync({ type: 'blob', platform: 'UNIX' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url; a.download = pluginId + '.zip'; a.click();
            URL.revokeObjectURL(url);
            app.Toast.show('Plugin zip generated', 'success');
        } catch (err) {
            app.Toast.show(err.message || 'Export failed', 'error');
        } finally {
            if (btn) { btn.disabled = false; btn.textContent = 'Generate'; }
        }
    }

    function openPanel() {
        document.getElementById('config-overlay').classList.add('open');
        document.getElementById('config-detail-panel').classList.add('open');
    }

    function closePanel() {
        document.getElementById('config-overlay').classList.remove('open');
        document.getElementById('config-detail-panel').classList.remove('open');
    }

    function buildPluginPanel(pluginId) {
        const el = document.querySelector('[data-plugin-detail="' + pluginId + '"]');
        if (!el) return;
        let data;
        try { data = JSON.parse(el.textContent); } catch (e) { return; }

        document.getElementById('panel-title').textContent = data.name || pluginId;

        let html = '<div class="config-panel-section">' +
            '<h4>Overview</h4>' +
            '<div class="config-overview-grid">' +
                '<span class="config-overview-label">ID</span><span class="config-overview-value"><code>' + app.escapeHtml(data.id) + '</code></span>' +
                '<span class="config-overview-label">Status</span><span class="config-overview-value">' +
                    (data.enabled ? '<span class="badge badge-green">Enabled</span>' : '<span class="badge badge-gray">Disabled</span>') + '</span>' +
                '<span class="config-overview-label">Version</span><span class="config-overview-value">' + app.escapeHtml(data.version || '—') + '</span>' +
                '<span class="config-overview-label">Category</span><span class="config-overview-value">' + app.escapeHtml(data.category || '—') + '</span>' +
                '<span class="config-overview-label">Author</span><span class="config-overview-value">' + app.escapeHtml(data.author_name || '—') + '</span>' +
                '<span class="config-overview-label">Description</span><span class="config-overview-value">' + app.escapeHtml(data.description || '—') + '</span>' +
            '</div>' +
        '</div>';

        html += '<div class="config-panel-section">' +
            '<h4>Environment</h4>' +
            '<div id="panel-env-status">Loading...</div>' +
        '</div>';

        document.getElementById('panel-body').innerHTML = html;

        let footer = '';
        if (data.id !== 'custom') {
            footer = '<a href="/admin/org/plugins/edit/?id=' + encodeURIComponent(data.id) + '" class="btn btn-primary">Edit Plugin</a>' +
                ' <button class="btn btn-secondary" data-open-env="' + app.escapeHtml(data.id) + '" data-plugin-name="' + app.escapeHtml(data.name) + '">Configure Env</button>';
        }
        document.getElementById('panel-footer').innerHTML = footer;

        openPanel();

        if (data.id !== 'custom') {
            loadEnvStatus(data.id, document.getElementById('panel-env-status'));
        } else {
            document.getElementById('panel-env-status').innerHTML = '<div class="empty-state"><p>N/A</p></div>';
        }
    }

    async function forkAgent(agentId) {
        try {
            const data = await app.api('/agents/' + encodeURIComponent(agentId));
            const customAgentId = data.id + '-custom-' + Date.now();
            await app.api('/user-agents', {
                method: 'POST',
                body: JSON.stringify({
                    agent_id: customAgentId,
                    name: (data.name || agentId) + ' (Custom)',
                    description: data.description || '',
                    system_prompt: data.system_prompt || '',
                    base_agent_id: data.id
                })
            });
            app.Toast.show('Agent customized — your copy has been created', 'success');
            await app.api('/agents/' + encodeURIComponent(agentId), {
                method: 'PUT',
                body: JSON.stringify({ enabled: false })
            });
            window.location.reload();
        } catch (err) {
            app.Toast.show(err.message || 'Failed to customize agent', 'error');
        }
    }

    function getSkillData(skillId) {
        const details = document.querySelectorAll('[data-plugin-detail]');
        for (let i = 0; i < details.length; i++) {
            try {
                const data = JSON.parse(details[i].textContent);
                if (data.skills) {
                    const found = data.skills.find(function(s) { return s.id === skillId; });
                    if (found) return found;
                }
            } catch (e) {}
        }
        return null;
    }

    async function forkSkill(skillId, btn) {
        const data = getSkillData(skillId);
        if (!data) {
            app.Toast.show('Skill data not found', 'error');
            return;
        }
        if (!confirm('This will create a custom copy of "' + data.name + '" and disable the original system skill. Continue?')) {
            return;
        }
        const origText = btn ? btn.textContent : '';
        if (btn) { btn.disabled = true; btn.textContent = 'Customizing...'; }
        try {
            const customId = data.id + '-custom-' + Date.now();
            await app.api('/skills', {
                method: 'POST',
                body: JSON.stringify({
                    skill_id: customId,
                    name: data.name + ' (Custom)',
                    description: data.description || '',
                    content: '',
                    tags: [],
                    base_skill_id: data.id
                })
            });
            app.Toast.show('Skill customized — your copy has been created', 'success');
            await app.api('/skills/' + encodeURIComponent(skillId), {
                method: 'PUT',
                body: JSON.stringify({ enabled: false })
            });
            window.location.reload();
        } catch (err) {
            if (btn) { btn.disabled = false; btn.textContent = origText; }
            app.Toast.show(err.message || 'Failed to customize skill', 'error');
        }
    }

    function toggleDetailRow(pluginId, section) {
        const detailRow = document.querySelector('tr[data-detail-for="' + pluginId + '"]');
        if (!detailRow) return;

        const isVisible = detailRow.classList.contains('visible');

        document.querySelectorAll('tr.detail-row.visible').forEach(function(r) {
            if (r !== detailRow) {
                r.classList.remove('visible');
                const otherId = r.getAttribute('data-detail-for');
                const otherIndicator = document.querySelector('[data-expand-for="' + otherId + '"]');
                if (otherIndicator) otherIndicator.classList.remove('expanded');
            }
        });

        const indicator = document.querySelector('[data-expand-for="' + pluginId + '"]');

        if (!isVisible) {
            detailRow.classList.add('visible');
            if (indicator) indicator.classList.add('expanded');
            if (section) {
                detailRow.querySelectorAll('.detail-section').forEach(function(s) {
                    s.classList.remove('active');
                });
                const target = detailRow.querySelector('[data-section="' + section + '"]');
                if (target) {
                    target.classList.add('active');
                    target.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
                }
            }
        } else {
            detailRow.classList.remove('visible');
            if (indicator) indicator.classList.remove('expanded');
        }
    }

    function applyFilters() {
        const searchVal = (document.getElementById('plugin-search').value || '').toLowerCase();
        const categoryVal = document.getElementById('category-filter').value.toLowerCase();
        const rows = document.querySelectorAll('#plugins-table tr.clickable-row');
        rows.forEach(function(row) {
            const name = row.getAttribute('data-name') || '';
            const category = (row.getAttribute('data-category') || '').toLowerCase();
            const matchSearch = !searchVal || name.indexOf(searchVal) >= 0;
            const matchCategory = !categoryVal || category === categoryVal;
            row.style.display = (matchSearch && matchCategory) ? '' : 'none';
            const detailFor = row.getAttribute('data-entity-id');
            if (detailFor) {
                const detailRow = document.querySelector('tr[data-detail-for="' + detailFor + '"]');
                if (detailRow && row.style.display === 'none') {
                    detailRow.classList.remove('visible');
                }
            }
        });
    }

    app.initPluginsConfig = function() {
        const bulkActions = app.OrgCommon ? app.OrgCommon.initBulkActions('#plugins-table', 'bulk-actions-btn') : null;

        const pluginRows = document.querySelectorAll('#plugins-table tr[data-entity-type="plugin"]');
        pluginRows.forEach(function(row) {
            const pid = row.getAttribute('data-entity-id');
            if (!pid || pid === 'custom') return;
            updateGenerateButtons(pid);
            app.api('/plugins/' + encodeURIComponent(pid) + '/env').then(function(envData) {
                pluginEnvValid[pid] = envData.valid !== false;
                updateGenerateButtons(pid);
            }).catch(function() {});
        });

        app.shared.createDebouncedSearch(document, 'plugin-search', function() {
            applyFilters();
        });

        document.getElementById('category-filter').addEventListener('change', function() {
            applyFilters();
        });

        // Remove item from plugin
        app.events.on('click', '[data-remove-from-plugin]', function(e, btn) {
            e.stopPropagation();
            const itemId = btn.getAttribute('data-remove-from-plugin');
            const resourceType = btn.getAttribute('data-resource-type');
            const pluginId = btn.getAttribute('data-plugin-id');
            if (!pluginId || pluginId === 'custom') return;

            const detailEl = document.querySelector('[data-plugin-detail="' + pluginId + '"]');
            if (!detailEl) return;
            let data;
            try { data = JSON.parse(detailEl.textContent); } catch (ex) { return; }

            // Build updated array without the removed item
            const apiField = resourceType === 'mcp_servers' ? 'mcp_servers' : resourceType;
            let currentIds;
            if (resourceType === 'skills') {
                currentIds = (data.skills || []).map(function(s) { return s.id; });
            } else if (resourceType === 'agents') {
                currentIds = (data.agents || []).map(function(a) { return a.id; });
            } else if (resourceType === 'mcp_servers') {
                currentIds = data.mcp_servers || [];
            } else if (resourceType === 'hooks') {
                currentIds = (data.hooks || []).map(function(h) { return h.id; });
            } else {
                return;
            }

            const updatedIds = currentIds.filter(function(id) { return id !== itemId; });
            const body = {};
            body[apiField] = updatedIds;

            btn.disabled = true;
            app.api('/plugins/' + encodeURIComponent(pluginId), {
                method: 'PUT',
                body: JSON.stringify(body)
            }).then(function() {
                // Remove the row from DOM
                const row = btn.closest('tr');
                if (row) row.remove();
                // Update the count
                const countEl = document.querySelector('[data-count="' + resourceType + '"][data-for-plugin="' + pluginId + '"]');
                if (countEl) countEl.textContent = updatedIds.length;
                // Update embedded JSON
                if (resourceType === 'skills') {
                    data.skills = data.skills.filter(function(s) { return s.id !== itemId; });
                } else if (resourceType === 'agents') {
                    data.agents = data.agents.filter(function(a) { return a.id !== itemId; });
                } else if (resourceType === 'mcp_servers') {
                    data.mcp_servers = updatedIds;
                } else if (resourceType === 'hooks') {
                    data.hooks = data.hooks.filter(function(h) { return h.id !== itemId; });
                }
                detailEl.textContent = JSON.stringify(data);
                app.Toast.show('Removed from plugin', 'success');
            }).catch(function(err) {
                btn.disabled = false;
                app.Toast.show(err.message || 'Failed to remove', 'error');
            });
        });

        // Add item to plugin
        app.events.on('click', '[data-add-to-plugin]', function(e, btn) {
            e.stopPropagation();
            const resourceType = btn.getAttribute('data-add-to-plugin');
            const pluginId = btn.getAttribute('data-plugin-id');
            if (!pluginId || pluginId === 'custom') return;

            const detailEl = document.querySelector('[data-plugin-detail="' + pluginId + '"]');
            if (!detailEl) return;
            let data;
            try { data = JSON.parse(detailEl.textContent); } catch (ex) { return; }

            // Map resource type to API path for fetching all available items
            const apiMap = { skills: '/skills', agents: '/agents', mcp_servers: '/mcp-servers', hooks: '/hooks' };
            const apiPath = apiMap[resourceType];
            if (!apiPath) return;

            // Get current IDs
            let currentIds;
            if (resourceType === 'skills') {
                currentIds = (data.skills || []).map(function(s) { return s.id; });
            } else if (resourceType === 'agents') {
                currentIds = (data.agents || []).map(function(a) { return a.id; });
            } else if (resourceType === 'mcp_servers') {
                currentIds = data.mcp_servers || [];
            } else if (resourceType === 'hooks') {
                currentIds = (data.hooks || []).map(function(h) { return h.id; });
            }
            const currentSet = {};
            currentIds.forEach(function(id) { currentSet[id] = true; });

            btn.disabled = true;
            btn.textContent = 'Loading...';
            app.api(apiPath).then(function(allItems) {
                btn.disabled = false;
                btn.textContent = '+ Add ' + resourceType.charAt(0).toUpperCase() + resourceType.slice(1).replace('_', ' ');
                const items = Array.isArray(allItems) ? allItems : (allItems.items || allItems.data || []);
                const available = items.filter(function(item) {
                    const id = typeof item === 'string' ? item : (item.id || item.skill_id || item.agent_id);
                    return id && !currentSet[id];
                });

                if (available.length === 0) {
                    app.Toast.show('No additional ' + resourceType.replace('_', ' ') + ' available', 'info');
                    return;
                }

                // Build popup
                const overlay = document.createElement('div');
                overlay.className = 'confirm-overlay';
                let checklistHtml = '<div class="add-checklist">';
                available.forEach(function(item) {
                    const id = typeof item === 'string' ? item : (item.id || item.skill_id || item.agent_id);
                    const name = typeof item === 'string' ? item : (item.name || item.id || item.skill_id);
                    checklistHtml += '<label><input type="checkbox" value="' + app.escapeHtml(id) + '"> ' + app.escapeHtml(name) + '</label>';
                });
                checklistHtml += '</div>';

                overlay.innerHTML = '<div class="confirm-dialog">' +
                    '<h3 style="margin:0 0 var(--space-3)">Add ' + resourceType.replace('_', ' ') + '</h3>' +
                    checklistHtml +
                    '<div style="display:flex;gap:var(--space-3);justify-content:flex-end;margin-top:var(--space-3)">' +
                        '<button class="btn btn-secondary" data-add-cancel>Cancel</button>' +
                        '<button class="btn btn-primary" data-add-confirm>Add Selected</button>' +
                    '</div>' +
                '</div>';
                document.body.appendChild(overlay);

                overlay.addEventListener('click', function(ev) {
                    if (ev.target === overlay || ev.target.closest('[data-add-cancel]')) {
                        overlay.remove();
                        return;
                    }
                    const confirmBtn = ev.target.closest('[data-add-confirm]');
                    if (!confirmBtn) return;

                    const checked = overlay.querySelectorAll('.add-checklist input:checked');
                    if (checked.length === 0) {
                        overlay.remove();
                        return;
                    }
                    const newIds = [];
                    checked.forEach(function(cb) { newIds.push(cb.value); });
                    const mergedIds = currentIds.concat(newIds);

                    const body = {};
                    const apiField = resourceType === 'mcp_servers' ? 'mcp_servers' : resourceType;
                    body[apiField] = mergedIds;

                    confirmBtn.disabled = true;
                    confirmBtn.textContent = 'Saving...';
                    app.api('/plugins/' + encodeURIComponent(pluginId), {
                        method: 'PUT',
                        body: JSON.stringify(body)
                    }).then(function() {
                        overlay.remove();
                        app.Toast.show('Added to plugin', 'success');
                        window.location.reload();
                    }).catch(function(err) {
                        confirmBtn.disabled = false;
                        confirmBtn.textContent = 'Add Selected';
                        app.Toast.show(err.message || 'Failed to add', 'error');
                    });
                });
            }).catch(function(err) {
                btn.disabled = false;
                btn.textContent = '+ Add ' + resourceType.charAt(0).toUpperCase() + resourceType.slice(1).replace('_', ' ');
                app.Toast.show(err.message || 'Failed to load available items', 'error');
            });
        });

        app.events.on('click', '[data-expand-section]', function(e, expandBadge) {
            e.stopPropagation();
            const section = expandBadge.getAttribute('data-expand-section');
            const pluginId = expandBadge.getAttribute('data-plugin-id');
            toggleDetailRow(pluginId, section);
        });

        app.events.on('click', '[data-browse-skill]', function(e, el) {
            e.stopPropagation();
            e.preventDefault();
            const skillId = el.getAttribute('data-browse-skill');
            const skillName = el.getAttribute('data-skill-name') || skillId;
            if (app.skillFiles) app.skillFiles.open(skillId, skillName);
        });

        app.events.on('click', '[data-toggle-json]', function(e, jsonToggle) {
            e.stopPropagation();
            const pid = jsonToggle.getAttribute('data-toggle-json');
            const jsonView = document.querySelector('[data-json-for="' + pid + '"]');
            if (jsonView) {
                if (jsonView.style.display === 'none') {
                    if (!jsonView.textContent.trim()) {
                        const detailEl = document.querySelector('[data-plugin-detail="' + pid + '"]');
                        if (detailEl) {
                            try {
                                const d = JSON.parse(detailEl.textContent);
                                jsonView.textContent = JSON.stringify(d, null, 2);
                            } catch (ex) {}
                        }
                    }
                    jsonView.style.display = '';
                    jsonToggle.textContent = 'Hide JSON';
                } else {
                    jsonView.style.display = 'none';
                    jsonToggle.textContent = 'Show JSON';
                }
            }
        });

        app.events.on('click', 'tr.clickable-row', function(e, row) {
            if (e.target.closest('[data-no-row-click]') || e.target.closest('[data-action="toggle"]') || e.target.closest('.actions-menu') || e.target.closest('.btn') || e.target.closest('a') || e.target.closest('input')) return;
            const entityId = row.getAttribute('data-entity-id');
            toggleDetailRow(entityId);
        });

        app.events.on('click', '[data-open-env]', function(e, envBtn) {
            e.stopPropagation();
            const envPluginId = envBtn.getAttribute('data-open-env');
            const pluginName = envBtn.getAttribute('data-plugin-name') || envPluginId;
            if (app.pluginEnv) app.pluginEnv.open(envPluginId, pluginName);
        });

        app.events.on('click', '[data-generate-plugin]', function(e, generateBtn) {
            e.stopPropagation();
            const platform = generateBtn.getAttribute('data-platform') || 'unix';
            handleExport(generateBtn.getAttribute('data-generate-plugin'), generateBtn, platform);
        });

        app.events.on('click', '[data-delete-plugin]', function(e, deletePluginBtn) {
            e.stopPropagation();
            app.shared.closeAllMenus();
            showDeleteConfirm(deletePluginBtn.getAttribute('data-delete-plugin'));
        });

        document.getElementById('panel-close').addEventListener('click', closePanel);
        document.getElementById('config-overlay').addEventListener('click', closePanel);

        app.events.on('click', '#export-marketplace-btn', async function(e, btn) {
            btn.disabled = true;
            btn.textContent = 'Generating...';
            try {
                const data = await app.api('/export?platform=unix');
                const JSZip = await app.shared.loadJSZip();
                const zip = new JSZip();
                const items = data.plugins || [];
                if (data.marketplace && data.marketplace.content) {
                    zip.file(data.marketplace.path, data.marketplace.content);
                }
                for (let i = 0; i < items.length; i++) {
                    const bundle = items[i];
                    for (let j = 0; j < bundle.files.length; j++) {
                        const f = bundle.files[j];
                        const opts = f.executable ? { unixPermissions: '755' } : {};
                        zip.file('plugins/' + bundle.id + '/' + f.path, f.content, opts);
                    }
                }
                const blob = await zip.generateAsync({ type: 'blob', platform: 'UNIX' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url; a.download = 'foodles-marketplace.zip'; a.click();
                URL.revokeObjectURL(url);
                app.Toast.show('Marketplace zip generated', 'success');
            } catch (err) {
                app.Toast.show(err.message || 'Export failed', 'error');
            } finally {
                btn.disabled = false;
                btn.textContent = 'Export';
            }
        });

        window.addEventListener('env-saved', function(e) {
            const pid = e.detail && e.detail.pluginId;
            if (!pid) return;
            app.api('/plugins/' + encodeURIComponent(pid) + '/env').then(function(envData) {
                pluginEnvValid[pid] = envData.valid !== false;
                updateGenerateButtons(pid);
            }).catch(function() {});
        });
    };

    app.initPluginsList = app.initPluginsConfig;

    function loadEnvStatus(pluginId, container) {
        container.innerHTML = '<div style="padding:var(--space-4);color:var(--text-tertiary);font-size:var(--text-sm)">Loading variables...</div>';
        app.api('/plugins/' + encodeURIComponent(pluginId) + '/env').then(function(data) {
            const defs = data.definitions || [];
            const stored = data.stored || [];
            if (!defs.length && !stored.length) {
                container.innerHTML = '<div class="empty-state"><p>No environment variables defined for this plugin.</p></div>';
                return;
            }
            const storedMap = {};
            stored.forEach(function(v) { storedMap[v.var_name] = v; });
            let html = '';
            defs.forEach(function(def) {
                const s = storedMap[def.name];
                const hasValue = s && s.var_value && s.var_value !== '';
                const valueBadge = hasValue
                    ? '<span class="badge badge-green">configured</span>'
                    : '<span class="badge badge-red">not set</span>';
                let maskedVal = '';
                if (hasValue) {
                    maskedVal = s.is_secret ? '--------' : app.escapeHtml(s.var_value);
                }
                const requiredBadge = (def.required !== false && !hasValue) ? ' <span class="badge badge-yellow">required</span>' : '';
                const secretBadge = def.secret ? ' <span class="badge badge-gray">secret</span>' : '';
                html += '<div class="detail-item">' +
                    '<div class="detail-item-info">' +
                        '<div class="detail-item-name">' +
                            '<code style="background:var(--bg-surface-raised);padding:1px 6px;border-radius:var(--radius-xs);font-size:var(--text-sm)">' + app.escapeHtml(def.name) + '</code> ' +
                            valueBadge + requiredBadge + secretBadge +
                        '</div>' +
                        '<div class="detail-item-desc" style="font-size:var(--text-sm);color:var(--text-secondary);margin-top:var(--space-1)">' +
                            (def.description ? app.escapeHtml(def.description) : '') +
                            (maskedVal ? ' <span style="font-family:monospace;color:var(--text-tertiary)">' + maskedVal + '</span>' : '') +
                        '</div>' +
                    '</div>' +
                '</div>';
            });
            html += '<div style="padding:var(--space-3) 0">' +
                '<button class="btn btn-primary btn-sm" data-open-env="' + app.escapeHtml(pluginId) + '" data-plugin-name="' + app.escapeHtml(pluginId) + '">Configure</button>' +
            '</div>';
            container.innerHTML = html;
        }).catch(function() {
            container.innerHTML = '<div class="empty-state"><p>Failed to load environment variables.</p></div>';
        });
    }
})(window.AdminApp);

(function(app) {
    'use strict';

    const escapeHtml = app.escapeHtml;
    let plugins = [];

    function showVisibilityModal(pluginId) {
        const plugin = plugins.find(function(p) { return p.id === pluginId; });
        if (!plugin) return;
        const rules = plugin.visibility_rules || [];

        const overlay = document.createElement('div');
        overlay.className = 'confirm-overlay';
        overlay.id = 'visibility-modal';

        const rulesListHtml = renderRulesList(rules);

        overlay.innerHTML = '<div class="confirm-dialog" style="max-width:500px">' +
            '<h3 style="margin:0 0 var(--space-3)">Edit Visibility - ' + escapeHtml(plugin.name) + '</h3>' +
            '<div id="visibility-rules-list">' + rulesListHtml + '</div>' +
            '<div style="margin-top:var(--space-4);padding-top:var(--space-3);border-top:1px solid var(--border-primary)">' +
                '<strong style="font-size:var(--text-sm)">Add Rule</strong>' +
                '<div style="display:flex;gap:var(--space-2);margin-top:var(--space-2);flex-wrap:wrap">' +
                    '<select id="vis-rule-type" class="btn btn-secondary" style="cursor:pointer;font-size:var(--text-sm)">' +
                        '<option value="department">Department</option>' +
                        '<option value="user">User</option>' +
                    '</select>' +
                    '<input type="text" id="vis-rule-value" class="search-input" placeholder="Value..." style="flex:1;min-width:120px;font-size:var(--text-sm)">' +
                    '<select id="vis-rule-access" class="btn btn-secondary" style="cursor:pointer;font-size:var(--text-sm)">' +
                        '<option value="allow">Allow</option>' +
                        '<option value="deny">Deny</option>' +
                    '</select>' +
                    '<button class="btn btn-secondary" id="vis-add-rule" style="font-size:var(--text-sm)">Add</button>' +
                '</div>' +
            '</div>' +
            '<div style="display:flex;gap:var(--space-3);justify-content:flex-end;margin-top:var(--space-4)">' +
                '<button class="btn btn-secondary" data-confirm-cancel>Cancel</button>' +
                '<button class="btn btn-primary" id="vis-save">Save</button>' +
            '</div>' +
        '</div>';

        document.body.appendChild(overlay);

        const modalRules = rules.slice();

        function refreshRulesList() {
            const container = overlay.querySelector('#visibility-rules-list');
            if (container) container.innerHTML = renderRulesList(modalRules);
        }

        overlay.addEventListener('click', async function(e) {
            if (e.target === overlay || e.target.closest('[data-confirm-cancel]')) {
                overlay.remove();
                return;
            }
            const removeBtn = e.target.closest('[data-remove-rule]');
            if (removeBtn) {
                modalRules.splice(parseInt(removeBtn.getAttribute('data-remove-rule'), 10), 1);
                refreshRulesList();
                return;
            }
            if (e.target.closest('#vis-add-rule')) {
                const ruleValue = overlay.querySelector('#vis-rule-value').value.trim();
                if (!ruleValue) { app.Toast.show('Rule value is required', 'error'); return; }
                modalRules.push({
                    rule_type: overlay.querySelector('#vis-rule-type').value,
                    rule_value: ruleValue,
                    access: overlay.querySelector('#vis-rule-access').value
                });
                overlay.querySelector('#vis-rule-value').value = '';
                refreshRulesList();
                return;
            }
            if (e.target.closest('#vis-save')) {
                const saveBtn = overlay.querySelector('#vis-save');
                saveBtn.disabled = true;
                saveBtn.textContent = 'Saving...';
                try {
                    await app.api('/marketplace-plugins/' + encodeURIComponent(pluginId) + '/visibility', {
                        method: 'PUT',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ rules: modalRules })
                    });
                    app.Toast.show('Visibility updated', 'success');
                    overlay.remove();
                    window.location.reload();
                } catch (err) {
                    app.Toast.show(err.message || 'Failed to save visibility', 'error');
                    saveBtn.disabled = false;
                    saveBtn.textContent = 'Save';
                }
            }
        });
    }

    function renderRulesList(rules) {
        if (!rules.length) return '<p style="font-size:var(--text-sm);color:var(--text-tertiary)">No rules configured</p>';
        return rules.map(function(rule, idx) {
            return '<div style="display:flex;align-items:center;gap:var(--space-2);padding:var(--space-1) 0;font-size:var(--text-sm)">' +
                '<span class="badge ' + (rule.access === 'allow' ? 'badge-yellow' : 'badge-red') + '">' + escapeHtml(rule.rule_type) + ': ' + escapeHtml(rule.rule_value) + ' (' + escapeHtml(rule.access) + ')</span>' +
                '<button class="btn btn-danger" style="font-size:var(--text-xs);padding:2px 6px" data-remove-rule="' + idx + '">Remove</button>' +
            '</div>';
        }).join('');
    }

    app.initMarketplace = function(selector, pluginsData) {
        const root = document.querySelector(selector);
        if (!root) return;
        plugins = pluginsData || [];

        const searchInput = document.getElementById('mkt-search');
        if (searchInput) {
            const debounceTimer = null;
            searchInput.addEventListener('input', function() {
                clearTimeout(debounceTimer);
                debounceTimer = setTimeout(function() {
                    const q = searchInput.value.toLowerCase().trim();
                    const cards = root.querySelectorAll('.plugin-card[data-plugin-id]');
                    for (let i = 0; i < cards.length; i++) {
                        const name = cards[i].getAttribute('data-search-name') || '';
                        const desc = cards[i].getAttribute('data-search-desc') || '';
                        const cat = cards[i].getAttribute('data-search-cat') || '';
                        cards[i].style.display = (!q || name.indexOf(q) >= 0 || desc.indexOf(q) >= 0 || cat.indexOf(q) >= 0) ? '' : 'none';
                    }
                }, 200);
            });
        }

        const sortSelect = document.getElementById('mkt-sort');
        if (sortSelect) {
            sortSelect.addEventListener('change', function() {
                const url = new URL(window.location.href);
                url.searchParams.set('sort', sortSelect.value);
                window.location.href = url.toString();
            });
        }

        root.addEventListener('click', async function(e) {
            const toggleBtn = e.target.closest('[data-toggle-plugin]');
            if (toggleBtn) {
                const card = toggleBtn.closest('.plugin-card');
                const details = card && card.querySelector('.plugin-details');
                if (details) {
                    const isHidden = details.style.display === 'none';
                    details.style.display = isHidden ? '' : 'none';
                    const icon = toggleBtn.querySelector('.expand-icon');
                    if (icon) icon.style.transform = isHidden ? 'rotate(180deg)' : '';
                }
                return;
            }

            const visBtn = e.target.closest('[data-edit-visibility]');
            if (visBtn) {
                e.stopPropagation();
                showVisibilityModal(visBtn.getAttribute('data-edit-visibility'));
                return;
            }

            const loadUsersBtn = e.target.closest('[data-load-users]');
            if (loadUsersBtn) {
                e.stopPropagation();
                const pluginId = loadUsersBtn.getAttribute('data-load-users');
                loadUsersBtn.disabled = true;
                loadUsersBtn.textContent = 'Loading...';
                try {
                    const usersData = await app.api('/marketplace-plugins/' + encodeURIComponent(pluginId) + '/users');
                    const users = usersData.users || usersData || [];
                    const container = root.querySelector('[data-users-for="' + pluginId + '"]');
                    if (container) {
                        if (users.length === 0) {
                            container.innerHTML = '<div style="margin-top:var(--space-2);font-size:var(--text-xs);color:var(--text-tertiary)">No users found</div>';
                        } else {
                            container.innerHTML = '<div style="margin-top:var(--space-2);display:flex;flex-direction:column;gap:var(--space-1)">' +
                                users.map(function(u) {
                                    return '<div style="display:flex;align-items:center;gap:var(--space-2);font-size:var(--text-xs);padding:var(--space-1) 0;border-bottom:1px solid var(--border-primary)">' +
                                        '<span style="font-weight:600;color:var(--text-primary)">' + escapeHtml(u.display_name || 'Unknown') + '</span>' +
                                        (u.department ? '<span class="badge badge-blue">' + escapeHtml(u.department) + '</span>' : '') +
                                        '<span class="badge badge-gray">' + (u.event_count || 0) + ' events</span>' +
                                        (u.last_used ? '<span style="color:var(--text-tertiary)">' + new Date(u.last_used).toLocaleDateString() + '</span>' : '') +
                                    '</div>';
                                }).join('') +
                            '</div>';
                        }
                    }
                    loadUsersBtn.style.display = 'none';
                } catch (err) {
                    app.Toast.show(err.message || 'Failed to load users', 'error');
                    loadUsersBtn.disabled = false;
                    loadUsersBtn.textContent = 'Load Users';
                }
                return;
            }
        });
    };
})(window.AdminApp);

(function(app) {
    'use strict';

    const escapeHtml = app.escapeHtml;
    const versionDetails = {};
    const diffCache = {};
    let activeDiff = null;

    function marketplaceApi(userId, path) {
        const url = '/api/public/marketplace/' + encodeURIComponent(userId) + path;
        return fetch(url, { headers: { 'Content-Type': 'application/json' } })
            .then(function(resp) {
                if (!resp.ok) return resp.text().then(function(t) { throw new Error(t || resp.statusText); });
                return resp.json();
            });
    }

    function renderSkillRow(skill, versionId) {
        const hasBase = skill.base_skill_id && skill.base_skill_id !== 'null';
        const compareBtn = '';
        if (hasBase) {
            const isActive = activeDiff && activeDiff.versionId === versionId && activeDiff.skillId === skill.skill_id;
            compareBtn = '<button class="btn btn-secondary btn-sm" data-compare-skill="' + escapeHtml(skill.skill_id) +
                '" data-compare-version="' + escapeHtml(versionId) +
                '" data-base-skill="' + escapeHtml(skill.base_skill_id) +
                '" style="font-size:var(--text-xs);padding:2px 8px;white-space:nowrap"' +
                (isActive ? ' disabled' : '') + '>' +
                (isActive ? 'Viewing Diff' : 'Compare to Core') + '</button>';
        }
        const enabledBadge = skill.enabled === false
            ? '<span class="badge badge-red">disabled</span>'
            : '<span class="badge badge-green">enabled</span>';
        const baseBadge = hasBase
            ? '<span class="badge badge-yellow">customized</span>'
            : '<span class="badge badge-gray">custom</span>';

        return '<div class="detail-item">' +
            '<div class="detail-item-info">' +
                '<div class="detail-item-name">' +
                    escapeHtml(skill.name || skill.skill_id) +
                    ' ' + baseBadge + ' ' + enabledBadge +
                '</div>' +
                '<div style="font-size:var(--text-xs);color:var(--text-tertiary);margin-top:2px">' +
                    '<code style="background:var(--bg-surface-raised);padding:1px 6px;border-radius:var(--radius-xs)">' + escapeHtml(skill.skill_id) + '</code>' +
                    (skill.version ? ' <span>v' + escapeHtml(skill.version) + '</span>' : '') +
                    (skill.description ? ' &mdash; ' + escapeHtml(app.shared.truncate(skill.description, 80)) : '') +
                '</div>' +
            '</div>' +
            compareBtn +
        '</div>';
    }

    function renderDiffPanel(userSkill, coreSkill) {
        const userLines = (userSkill.content || '').split('\n');
        const coreLines = (coreSkill.content || '').split('\n');
        const maxLen = Math.max(userLines.length, coreLines.length);
        let diffHtml = '';
        for (let i = 0; i < maxLen; i++) {
            const coreLine = i < coreLines.length ? coreLines[i] : '';
            const userLine = i < userLines.length ? userLines[i] : '';
            const lineNum = i + 1;
            if (coreLine === userLine) {
                diffHtml += '<div class="diff-line diff-unchanged"><span class="diff-linenum">' + lineNum + '</span><span class="diff-text">' + escapeHtml(coreLine) + '</span></div>';
            } else {
                if (coreLine) diffHtml += '<div class="diff-line diff-removed"><span class="diff-linenum">' + lineNum + '</span><span class="diff-text">- ' + escapeHtml(coreLine) + '</span></div>';
                if (userLine) diffHtml += '<div class="diff-line diff-added"><span class="diff-linenum">' + lineNum + '</span><span class="diff-text">+ ' + escapeHtml(userLine) + '</span></div>';
            }
        }

        let metaDiff = '';
        if ((userSkill.name || '') !== (coreSkill.name || '')) {
            metaDiff += '<div style="margin-bottom:var(--space-2)"><strong>Name:</strong> <span class="diff-removed" style="padding:1px 4px">' + escapeHtml(coreSkill.name || '') + '</span> &rarr; <span class="diff-added" style="padding:1px 4px">' + escapeHtml(userSkill.name || '') + '</span></div>';
        }
        if ((userSkill.description || '') !== (coreSkill.description || '')) {
            metaDiff += '<div style="margin-bottom:var(--space-2)"><strong>Description:</strong> <span class="diff-removed" style="padding:1px 4px">' + escapeHtml(coreSkill.description || '') + '</span> &rarr; <span class="diff-added" style="padding:1px 4px">' + escapeHtml(userSkill.description || '') + '</span></div>';
        }

        return '<div class="diff-panel">' +
            '<div class="diff-panel-header">' +
                '<h4 style="margin:0;font-size:var(--text-sm);font-weight:600">Diff: ' + escapeHtml(userSkill.skill_id) + '</h4>' +
                '<div style="display:flex;gap:var(--space-3);font-size:var(--text-xs)">' +
                    '<span><span class="badge badge-blue">core</span> Base skill</span>' +
                    '<span><span class="badge badge-green">user</span> User version</span>' +
                '</div>' +
                '<button class="btn btn-secondary btn-sm" data-close-diff style="margin-left:auto;font-size:var(--text-xs);padding:2px 8px">Close</button>' +
            '</div>' +
            (metaDiff ? '<div style="padding:var(--space-3) var(--space-4);border-bottom:1px solid var(--border-subtle);font-size:var(--text-sm)">' + metaDiff + '</div>' : '') +
            '<div class="diff-content">' + (diffHtml || '<div style="padding:var(--space-4);color:var(--text-tertiary);text-align:center">Content is identical</div>') + '</div>' +
        '</div>';
    }

    function renderVersionDetails(detailsContainer, versionId) {
        const detail = versionDetails[versionId];
        if (!detail || detail === 'loading') return;
        if (detail === 'error') {
            detailsContainer.innerHTML = '<div style="padding:var(--space-4)"><div class="empty-state"><p>Failed to load version details.</p></div></div>';
            return;
        }
        let skills = [];
        if (Array.isArray(detail.skills_snapshot)) {
            skills = detail.skills_snapshot;
        } else if (typeof detail.skills_snapshot === 'string') {
            try { skills = JSON.parse(detail.skills_snapshot); } catch(e) { skills = []; }
        }
        const skillsHtml = skills.length
            ? skills.map(function(s) { return renderSkillRow(s, versionId); }).join('')
            : '<div class="empty-state" style="padding:var(--space-4)"><p>No skills in this snapshot.</p></div>';

        let diffHtml = '';
        if (activeDiff && activeDiff.versionId === versionId && diffCache[activeDiff.cacheKey]) {
            const userSkill = skills.find(function(s) { return s.skill_id === activeDiff.skillId; });
            if (userSkill) diffHtml = renderDiffPanel(userSkill, diffCache[activeDiff.cacheKey]);
        }

        detailsContainer.innerHTML =
            '<div style="padding:var(--space-4)">' +
                '<div style="font-size:var(--text-sm);font-weight:600;margin-bottom:var(--space-2);color:var(--text-secondary)">Skills Snapshot (' + skills.length + ')</div>' +
                skillsHtml +
            '</div>' +
            diffHtml;
    }

    async function loadVersionDetail(versionId, userId, detailsContainer) {
        if (versionDetails[versionId] && versionDetails[versionId] !== 'loading') {
            renderVersionDetails(detailsContainer, versionId);
            return;
        }
        versionDetails[versionId] = 'loading';
        try {
            const detail = await marketplaceApi(userId, '/versions/' + encodeURIComponent(versionId));
            versionDetails[versionId] = detail;
        } catch(err) {
            versionDetails[versionId] = 'error';
        }
        renderVersionDetails(detailsContainer, versionId);
    }

    async function loadCoreDiff(skillId, baseSkillId, versionId, detailsContainer) {
        const cacheKey = baseSkillId + ':' + skillId;
        if (!diffCache[cacheKey]) {
            try {
                const coreData = await app.api('/skills/' + encodeURIComponent(baseSkillId) + '/base-content');
                diffCache[cacheKey] = coreData;
            } catch(err) {
                app.Toast.show('Failed to load core skill: ' + (err.message || 'Unknown error'), 'error');
                return;
            }
        }
        activeDiff = { versionId: versionId, skillId: skillId, cacheKey: cacheKey };
        renderVersionDetails(detailsContainer, versionId);
    }

    async function handleRestore(versionId, versionNum, userId) {
        if (!confirm('Restore to version ' + versionNum + '? Current state will be saved as a new version.')) return;
        try {
            const result = await fetch('/api/public/marketplace/' + encodeURIComponent(userId) + '/restore/' + encodeURIComponent(versionId), {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' }
            });
            if (!result.ok) throw new Error(await result.text() || 'Restore failed');
            const data = await result.json();
            app.Toast.show('Restored to version ' + data.restored_version + '. ' + data.skills_restored + ' skills restored.', 'success');
            window.location.reload();
        } catch (err) {
            app.Toast.show(err.message || 'Restore failed', 'error');
        }
    }

    app.initMarketplaceVersions = function(selector) {
        const root = document.querySelector(selector);
        if (!root) return;

        let activeTab = 'versions';
        const changelogLoaded = {};

        root.addEventListener('click', async function(e) {
            const tabBtn = e.target.closest('[data-tab]');
            if (tabBtn) {
                const newTab = tabBtn.getAttribute('data-tab');
                if (activeTab === newTab) return;
                activeTab = newTab;
                root.querySelectorAll('[data-tab]').forEach(function(btn) {
                    btn.className = btn.getAttribute('data-tab') === activeTab ? 'btn btn-primary' : 'btn btn-secondary';
                });
                document.getElementById('mv-versions-tab').style.display = activeTab === 'versions' ? '' : 'none';
                document.getElementById('mv-changelog-tab').style.display = activeTab === 'changelog' ? '' : 'none';
                if (activeTab === 'changelog') {
                    const selectedUserId = document.getElementById('mv-user-select').value;
                    if (selectedUserId && !changelogLoaded[selectedUserId]) {
                        await loadChangelog(selectedUserId);
                    }
                }
                return;
            }

            const toggleBtn = e.target.closest('[data-toggle-version]');
            if (toggleBtn && !e.target.closest('[data-restore-version]') && !e.target.closest('[data-compare-skill]')) {
                const card = toggleBtn.closest('.plugin-card');
                const details = card && card.querySelector('.plugin-details');
                if (details) {
                    const isHidden = details.style.display === 'none';
                    details.style.display = isHidden ? '' : 'none';
                    const icon = toggleBtn.querySelector('.expand-icon');
                    if (icon) icon.style.transform = isHidden ? 'rotate(180deg)' : '';
                    if (isHidden) {
                        const vid = toggleBtn.getAttribute('data-toggle-version');
                        const vUserId = toggleBtn.getAttribute('data-version-user');
                        loadVersionDetail(vid, vUserId, details);
                    }
                }
                return;
            }

            const compareBtn = e.target.closest('[data-compare-skill]');
            if (compareBtn) {
                e.stopPropagation();
                const skillId = compareBtn.getAttribute('data-compare-skill');
                const baseSkillId = compareBtn.getAttribute('data-base-skill');
                const compareVersionId = compareBtn.getAttribute('data-compare-version');
                const versionCard = compareBtn.closest('.version-card');
                const detailsEl = versionCard && versionCard.querySelector('.plugin-details');
                if (detailsEl) await loadCoreDiff(skillId, baseSkillId, compareVersionId, detailsEl);
                return;
            }

            if (e.target.closest('[data-close-diff]')) {
                e.stopPropagation();
                const diffVersionCard = e.target.closest('.version-card');
                const diffDetails = diffVersionCard && diffVersionCard.querySelector('.plugin-details');
                activeDiff = null;
                if (diffDetails) {
                    const diffVid = diffVersionCard.querySelector('[data-toggle-version]');
                    if (diffVid) renderVersionDetails(diffDetails, diffVid.getAttribute('data-toggle-version'));
                }
                return;
            }

            const restoreBtn = e.target.closest('[data-restore-version]');
            if (restoreBtn) {
                e.stopPropagation();
                await handleRestore(
                    restoreBtn.getAttribute('data-restore-version'),
                    restoreBtn.getAttribute('data-restore-num'),
                    restoreBtn.getAttribute('data-restore-user')
                );
                return;
            }
        });

        root.addEventListener('change', function(e) {
            if (e.target.id === 'mv-user-select') {
                const userId = e.target.value;
                const groups = root.querySelectorAll('.version-user-group');
                groups.forEach(function(group) {
                    const versions = group.querySelectorAll('[data-version-user]');
                    let hasMatch = !userId;
                    versions.forEach(function(v) {
                        if (v.getAttribute('data-version-user') === userId) hasMatch = true;
                    });
                    group.style.display = hasMatch ? '' : 'none';
                });
                if (activeTab === 'changelog' && userId) {
                    loadChangelog(userId);
                }
            }
        });

        async function loadChangelog(userId) {
            const container = document.getElementById('mv-changelog-tab');
            if (!container) return;
            if (!userId) {
                container.innerHTML = '<div class="empty-state"><p>Select a user to view changelog.</p></div>';
                return;
            }
            container.innerHTML = '<div class="loading-center"><div class="loading-spinner" role="status"><span class="sr-only">Loading...</span></div></div>';
            try {
                const changelog = await marketplaceApi(userId, '/changelog');
                changelogLoaded[userId] = true;
                if (!changelog || !changelog.length) {
                    container.innerHTML = '<div class="empty-state"><p>No changelog entries found for this user.</p></div>';
                    return;
                }
                const rows = changelog.map(function(entry) {
                    let actionClass = '';
                    switch(entry.action) {
                        case 'added': actionClass = 'badge-green'; break;
                        case 'updated': actionClass = 'badge-yellow'; break;
                        case 'deleted': actionClass = 'badge-red'; break;
                        case 'restored': actionClass = 'badge-blue'; break;
                        default: actionClass = 'badge-gray';
                    }
                    return '<tr>' +
                        '<td><span class="badge ' + actionClass + '">' + escapeHtml(entry.action) + '</span></td>' +
                        '<td><code style="background:var(--bg-surface-raised);padding:1px 4px;border-radius:var(--radius-xs);font-size:var(--text-xs)">' + escapeHtml(entry.skill_id) + '</code></td>' +
                        '<td>' + escapeHtml(entry.skill_name) + '</td>' +
                        '<td style="color:var(--text-secondary)">' + escapeHtml(entry.detail) + '</td>' +
                        '<td><span title="' + escapeHtml(app.formatDate(entry.created_at)) + '">' + escapeHtml(app.formatRelativeTime(entry.created_at)) + '</span></td>' +
                    '</tr>';
                }).join('');
                container.innerHTML = '<div class="table-container"><div class="table-scroll">' +
                    '<table class="data-table">' +
                        '<thead><tr><th>Action</th><th>Skill ID</th><th>Name</th><th>Detail</th><th>Time</th></tr></thead>' +
                        '<tbody>' + rows + '</tbody>' +
                    '</table>' +
                '</div></div>';
            } catch(err) {
                container.innerHTML = '<div class="empty-state"><p>Failed to load changelog.</p></div>';
            }
        }

        const urlUserId = new URLSearchParams(window.location.search).get('user_id');
        if (urlUserId) {
            const select = document.getElementById('mv-user-select');
            if (select) {
                select.value = urlUserId;
                select.dispatchEvent(new Event('change'));
            }
        }
    };
})(window.AdminApp);

(function(app) {
    'use strict';

    app.initPluginEditForm = function() {
        const form = document.getElementById('plugin-edit-form');
        if (!form) return;

        const pluginIdInput = form.querySelector('input[name="plugin_id"]');
        const pluginId = pluginIdInput ? pluginIdInput.value : '';

        form.addEventListener('submit', async function(e) {
            e.preventDefault();
            const formData = new FormData(form);
            const keywordsRaw = formData.get('keywords') || '';
            const keywords = keywordsRaw.split(',').map(function(t) { return t.trim(); }).filter(Boolean);
            const body = {
                name: formData.get('name'),
                description: formData.get('description') || '',
                version: formData.get('version') || '0.1.0',
                category: formData.get('category') || '',
                enabled: !!form.querySelector('input[name="enabled"]').checked,
                keywords: keywords,
                author: { name: formData.get('author_name') || '' },
                roles: app.formUtils.getCheckedValues(form, 'roles'),
                skills: app.formUtils.getCheckedValues(form, 'skills'),
                agents: app.formUtils.getCheckedValues(form, 'agents'),
                mcp_servers: app.formUtils.getCheckedValues(form, 'mcp_servers')
            };
            const submitBtn = form.querySelector('[type="submit"]');
            if (submitBtn) { submitBtn.disabled = true; submitBtn.textContent = 'Saving...'; }
            try {
                await app.api('/plugins/' + encodeURIComponent(pluginId), {
                    method: 'PUT',
                    body: JSON.stringify(body)
                });
                app.Toast.show('Plugin saved!', 'success');
                window.location.href = app.BASE + '/plugins/';
            } catch (err) {
                app.Toast.show(err.message || 'Failed to save plugin', 'error');
                if (submitBtn) { submitBtn.disabled = false; submitBtn.textContent = 'Save Changes'; }
            }
        });

        const deleteBtn = document.getElementById('btn-delete-plugin');
        if (deleteBtn) {
            deleteBtn.addEventListener('click', function() {
                app.shared.showConfirmDialog('Delete Plugin?', 'Are you sure you want to delete this plugin? This cannot be undone.', 'Delete', async function() {
                    deleteBtn.disabled = true;
                    deleteBtn.textContent = 'Deleting...';
                    try {
                        await app.api('/plugins/' + encodeURIComponent(pluginId), { method: 'DELETE' });
                        app.Toast.show('Plugin deleted', 'success');
                        window.location.href = app.BASE + '/plugins/';
                    } catch (err) {
                        app.Toast.show(err.message || 'Failed to delete plugin', 'error');
                        deleteBtn.disabled = false;
                        deleteBtn.textContent = 'Delete';
                    }
                });
            });
        }

        app.formUtils.attachFilterHandlers(form);
    };
})(window.AdminApp);

(function(app) {
    'use strict';

    app.pluginWizardSteps = {
        renderCurrentStep: function() { return ''; }
    };
})(window.AdminApp);

(function(app) {
    'use strict';

    const escapeHtml = app.escapeHtml;
    const TOTAL_STEPS = 7;
    const state = {
        step: 1,
        form: { plugin_id: '', name: '', description: '', version: '0.1.0', category: '', author_name: '', keywords: '', roles: {} },
        selectedSkills: {},
        selectedAgents: {},
        selectedMcpServers: {},
        hooks: []
    };
    let root = null;

    function getTemplate(id) {
        const tpl = document.getElementById(id);
        return tpl ? tpl.content.cloneNode(true) : document.createDocumentFragment();
    }

    function renderStepIndicator() {
        const labels = ['Basic Info', 'Skills', 'Agents', 'MCP Servers', 'Hooks', 'Roles & Access', 'Review'];
        const container = document.getElementById('wizard-step-indicator');
        if (!container) return;
        let html = '<div class="wizard-steps" style="display:flex;gap:var(--space-1);margin-bottom:var(--space-6);flex-wrap:wrap">';
        for (let i = 1; i <= TOTAL_STEPS; i++) {
            const isActive = i === state.step;
            const isDone = i < state.step;
            const bgColor = isActive ? 'var(--accent)' : (isDone ? 'var(--success)' : 'var(--bg-tertiary)');
            const textColor = (isActive || isDone) ? '#fff' : 'var(--text-tertiary)';
            html += '<div style="display:flex;align-items:center;gap:var(--space-2);padding:var(--space-2) var(--space-3);border-radius:var(--radius-md);background:' + bgColor + ';color:' + textColor + ';font-size:var(--text-sm);font-weight:' + (isActive ? '600' : '400') + '">' +
                '<span style="width:20px;height:20px;border-radius:50%;background:rgba(255,255,255,0.2);display:inline-flex;align-items:center;justify-content:center;font-size:var(--text-xs)">' + i + '</span>' +
                '<span>' + escapeHtml(labels[i - 1]) + '</span>' +
            '</div>';
        }
        html += '</div>';
        container.innerHTML = html;
    }

    function renderNav() {
        const nav = document.getElementById('wizard-nav');
        if (!nav) return;
        let html = '<div style="display:flex;gap:var(--space-3);margin-top:var(--space-6)">';
        if (state.step > 1) html += '<button type="button" class="btn btn-secondary" id="wizard-prev">Previous</button>';
        if (state.step < TOTAL_STEPS) html += '<button type="button" class="btn btn-primary" id="wizard-next">Next</button>';
        if (state.step === TOTAL_STEPS) html += '<button type="button" class="btn btn-primary" id="wizard-create">Create Plugin</button>';
        html += '</div>';
        nav.innerHTML = html;
    }

    function saveCurrentStepState() {
        if (!root) return;
        if (state.step === 1) {
            ['plugin_id', 'name', 'description', 'version', 'category'].forEach(function(name) {
                const input = root.querySelector('[name="' + name + '"]');
                if (input) state.form[name] = input.tagName === 'TEXTAREA' ? input.value : input.value;
            });
        }
        if (state.step === 2) {
            state.selectedSkills = {};
            root.querySelectorAll('input[name="wizard-skills"]:checked').forEach(function(cb) { state.selectedSkills[cb.value] = true; });
        }
        if (state.step === 3) {
            state.selectedAgents = {};
            root.querySelectorAll('input[name="wizard-agents"]:checked').forEach(function(cb) { state.selectedAgents[cb.value] = true; });
        }
        if (state.step === 4) {
            state.selectedMcpServers = {};
            root.querySelectorAll('input[name="wizard-mcp"]:checked').forEach(function(cb) { state.selectedMcpServers[cb.value] = true; });
        }
        if (state.step === 5) {
            const entries = root.querySelectorAll('.hook-entry');
            state.hooks = [];
            entries.forEach(function(entry) {
                state.hooks.push({
                    event: (entry.querySelector('[name="hook_event"]') || {}).value || 'PostToolUse',
                    matcher: (entry.querySelector('[name="hook_matcher"]') || {}).value || '*',
                    command: (entry.querySelector('[name="hook_command"]') || {}).value || '',
                    async: !!(entry.querySelector('[name="hook_async"]') || {}).checked
                });
            });
        }
        if (state.step === 6) {
            state.form.roles = {};
            root.querySelectorAll('input[name="wizard-roles"]:checked').forEach(function(cb) { state.form.roles[cb.value] = true; });
            const authorInput = root.querySelector('[name="author_name"]');
            if (authorInput) state.form.author_name = authorInput.value;
            const keywordsInput = root.querySelector('[name="keywords"]');
            if (keywordsInput) state.form.keywords = keywordsInput.value;
        }
    }

    function renderStep() {
        const contentEl = document.getElementById('wizard-step-content');
        if (!contentEl) return;
        contentEl.innerHTML = '';

        if (state.step === 7) {
            const frag = getTemplate('tpl-step-7');
            contentEl.appendChild(frag);
            renderReview();
        } else if (state.step === 5) {
            const frag5 = getTemplate('tpl-step-5');
            contentEl.appendChild(frag5);
            renderHooks();
        } else {
            const frag2 = getTemplate('tpl-step-' + state.step);
            contentEl.appendChild(frag2);
            restoreStepState();
        }

        renderStepIndicator();
        renderNav();
        app.formUtils.attachFilterHandlers(contentEl);
    }

    function restoreStepState() {
        if (state.step === 1) {
            ['plugin_id', 'name', 'description', 'version', 'category'].forEach(function(name) {
                const input = root.querySelector('[name="' + name + '"]');
                if (input && state.form[name]) {
                    if (input.tagName === 'TEXTAREA') input.value = state.form[name];
                    else input.value = state.form[name];
                }
            });
        }
        if (state.step === 2) {
            Object.keys(state.selectedSkills).forEach(function(val) {
                if (!state.selectedSkills[val]) return;
                const cb = root.querySelector('input[name="wizard-skills"][value="' + val + '"]');
                if (cb) cb.checked = true;
            });
        }
        if (state.step === 3) {
            Object.keys(state.selectedAgents).forEach(function(val) {
                if (!state.selectedAgents[val]) return;
                const cb = root.querySelector('input[name="wizard-agents"][value="' + val + '"]');
                if (cb) cb.checked = true;
            });
        }
        if (state.step === 4) {
            Object.keys(state.selectedMcpServers).forEach(function(val) {
                if (!state.selectedMcpServers[val]) return;
                const cb = root.querySelector('input[name="wizard-mcp"][value="' + val + '"]');
                if (cb) cb.checked = true;
            });
        }
        if (state.step === 6) {
            Object.keys(state.form.roles).forEach(function(val) {
                if (!state.form.roles[val]) return;
                const cb = root.querySelector('input[name="wizard-roles"][value="' + val + '"]');
                if (cb) cb.checked = true;
            });
            const authorInput = root.querySelector('[name="author_name"]');
            if (authorInput && state.form.author_name) authorInput.value = state.form.author_name;
            const keywordsInput = root.querySelector('[name="keywords"]');
            if (keywordsInput && state.form.keywords) keywordsInput.value = state.form.keywords;
        }
    }

    function renderHooks() {
        const list = document.getElementById('hooks-list');
        if (!list) return;
        list.innerHTML = '';
        state.hooks.forEach(function(hook) {
            const frag = getTemplate('tpl-hook-entry');
            const entry = frag.querySelector('.hook-entry');
            if (entry) {
                const eventSel = entry.querySelector('[name="hook_event"]');
                if (eventSel) eventSel.value = hook.event || 'PostToolUse';
                const matcherIn = entry.querySelector('[name="hook_matcher"]');
                if (matcherIn) matcherIn.value = hook.matcher || '*';
                const commandIn = entry.querySelector('[name="hook_command"]');
                if (commandIn) commandIn.value = hook.command || '';
                const asyncCb = entry.querySelector('[name="hook_async"]');
                if (asyncCb) asyncCb.checked = !!hook.async;
            }
            list.appendChild(frag);
        });
    }

    function renderReview() {
        const el = document.getElementById('wizard-review');
        if (!el) return;
        const f = state.form;
        const selectedSkills = Object.keys(state.selectedSkills).filter(function(k) { return state.selectedSkills[k]; });
        const selectedAgents = Object.keys(state.selectedAgents).filter(function(k) { return state.selectedAgents[k]; });
        const selectedMcp = Object.keys(state.selectedMcpServers).filter(function(k) { return state.selectedMcpServers[k]; });
        const selectedRoles = Object.keys(f.roles).filter(function(k) { return f.roles[k]; });
        function badgeList(items, emptyMsg) {
            if (!items.length) return '<span style="color:var(--text-tertiary)">' + escapeHtml(emptyMsg) + '</span>';
            return items.map(function(i) { return '<span class="badge badge-blue" style="margin:var(--space-1)">' + escapeHtml(i) + '</span>'; }).join('');
        }
        el.innerHTML =
            '<strong>Plugin ID:</strong><span>' + escapeHtml(f.plugin_id || '-') + '</span>' +
            '<strong>Name:</strong><span>' + escapeHtml(f.name || '-') + '</span>' +
            '<strong>Description:</strong><span>' + escapeHtml(f.description || '-') + '</span>' +
            '<strong>Version:</strong><span>' + escapeHtml(f.version || '0.1.0') + '</span>' +
            '<strong>Category:</strong><span>' + escapeHtml(f.category || '-') + '</span>' +
            '<strong>Author:</strong><span>' + escapeHtml(f.author_name || '-') + '</span>' +
            '<strong>Keywords:</strong><span>' + escapeHtml(f.keywords || '-') + '</span>' +
            '<strong>Roles:</strong><div>' + badgeList(selectedRoles, 'None selected') + '</div>' +
            '<strong>Skills (' + selectedSkills.length + '):</strong><div style="display:flex;flex-wrap:wrap">' + badgeList(selectedSkills, 'None selected') + '</div>' +
            '<strong>Agents (' + selectedAgents.length + '):</strong><div style="display:flex;flex-wrap:wrap">' + badgeList(selectedAgents, 'None selected') + '</div>' +
            '<strong>MCP (' + selectedMcp.length + '):</strong><div style="display:flex;flex-wrap:wrap">' + badgeList(selectedMcp, 'None selected') + '</div>' +
            '<strong>Hooks (' + state.hooks.length + '):</strong><span>' + (state.hooks.length > 0 ? state.hooks.map(function(h) { return escapeHtml(h.event + ': ' + (h.command || '?')); }).join(', ') : 'None') + '</span>';
    }

    function validateStep1() {
        const pid = state.form.plugin_id;
        const name = state.form.name;
        if (!pid || !pid.trim()) { app.Toast.show('Plugin ID is required', 'error'); return false; }
        if (!/^[a-z0-9]+(-[a-z0-9]+)*$/.test(pid.trim())) { app.Toast.show('Plugin ID must be kebab-case (e.g. my-plugin)', 'error'); return false; }
        if (!name || !name.trim()) { app.Toast.show('Name is required', 'error'); return false; }
        return true;
    }

    function buildPluginBody() {
        const f = state.form;
        return {
            id: f.plugin_id.trim(),
            name: f.name.trim(),
            description: f.description || '',
            version: f.version || '0.1.0',
            category: f.category || '',
            enabled: true,
            keywords: (f.keywords || '').split(',').map(function(t) { return t.trim(); }).filter(Boolean),
            author: { name: f.author_name || '' },
            roles: Object.keys(f.roles).filter(function(k) { return f.roles[k]; }),
            skills: Object.keys(state.selectedSkills).filter(function(k) { return state.selectedSkills[k]; }),
            agents: Object.keys(state.selectedAgents).filter(function(k) { return state.selectedAgents[k]; }),
            mcp_servers: Object.keys(state.selectedMcpServers).filter(function(k) { return state.selectedMcpServers[k]; }),
            hooks: state.hooks.filter(function(h) { return h.command; }).map(function(h) {
                return { event: h.event || 'PostToolUse', matcher: h.matcher || '*', command: h.command, async: !!h.async };
            })
        };
    }

    async function createPlugin() {
        const body = buildPluginBody();
        const btn = root.querySelector('#wizard-create');
        if (btn) { btn.disabled = true; btn.textContent = 'Creating...'; }
        try {
            await app.api('/plugins', { method: 'POST', body: JSON.stringify(body) });
            app.Toast.show('Plugin created!', 'success');
            window.location.href = app.BASE + '/plugins/';
        } catch (err) {
            app.Toast.show(err.message || 'Failed to create plugin', 'error');
            if (btn) { btn.disabled = false; btn.textContent = 'Create Plugin'; }
        }
    }

    function showImportModal() {
        const modal = document.getElementById('import-modal');
        if (modal) { modal.style.display = 'flex'; const urlInput = modal.querySelector('#import-url'); if (urlInput) urlInput.focus(); }
    }
    function hideImportModal() {
        const modal = document.getElementById('import-modal');
        if (modal) { modal.style.display = 'none'; const err = modal.querySelector('#import-error'); if (err) err.style.display = 'none'; }
    }
    async function submitImport() {
        const urlInput = document.getElementById('import-url');
        const errorEl = document.getElementById('import-error');
        const submitBtn = document.getElementById('import-submit');
        const targetSelect = document.getElementById('import-target');
        if (!urlInput || !submitBtn) return;
        const url = urlInput.value.trim();
        if (!url) { if (errorEl) { errorEl.textContent = 'URL is required'; errorEl.style.display = 'block'; } return; }
        const importTarget = targetSelect ? targetSelect.value : 'site';
        submitBtn.disabled = true; submitBtn.textContent = 'Importing...';
        if (errorEl) errorEl.style.display = 'none';
        try {
            await app.api('/plugins/import', { method: 'POST', body: JSON.stringify({ url: url, import_target: importTarget }) });
            app.Toast.show('Plugin imported successfully!', 'success');
            window.location.href = app.BASE + '/plugins/';
        } catch (err) {
            if (errorEl) { errorEl.textContent = err.message || 'Failed to import plugin'; errorEl.style.display = 'block'; }
            submitBtn.disabled = false; submitBtn.textContent = 'Import';
        }
    }

    app.initPluginWizard = function() {
        root = document.getElementById('plugin-create-content');
        if (!root) return;

        renderStep();

        root.addEventListener('click', function(e) {
            if (e.target.closest('#btn-import-url')) { showImportModal(); return; }
            if (e.target.closest('#import-cancel')) { hideImportModal(); return; }
            if (e.target.closest('#import-submit')) { submitImport(); return; }
            if (e.target.id === 'import-modal') { hideImportModal(); return; }
            if (e.target.closest('#wizard-next')) {
                saveCurrentStepState();
                if (state.step === 1 && !validateStep1()) return;
                if (state.step < TOTAL_STEPS) { state.step++; renderStep(); }
                return;
            }
            if (e.target.closest('#wizard-prev')) {
                saveCurrentStepState();
                if (state.step > 1) { state.step--; renderStep(); }
                return;
            }
            if (e.target.closest('#wizard-create')) { saveCurrentStepState(); createPlugin(); return; }
            if (e.target.closest('#btn-add-hook')) {
                saveCurrentStepState();
                state.hooks.push({ event: 'PostToolUse', matcher: '*', command: '', async: false });
                renderHooks();
                return;
            }
            const removeBtn = e.target.closest('[data-remove-hook]');
            if (removeBtn) {
                saveCurrentStepState();
                const entry = removeBtn.closest('.hook-entry');
                const hookList = document.getElementById('hooks-list');
                if (entry && hookList) {
                    const hookEntries = Array.from(hookList.querySelectorAll('.hook-entry'));
                    const idx = hookEntries.indexOf(entry);
                    if (idx >= 0) state.hooks.splice(idx, 1);
                    renderHooks();
                }
                return;
            }
            const selectAllBtn = e.target.closest('[data-select-all]');
            if (selectAllBtn) {
                const listId = selectAllBtn.getAttribute('data-select-all');
                const container = root.querySelector('[data-checklist="' + listId + '"]');
                if (container) container.querySelectorAll('input[type="checkbox"]').forEach(function(cb) { cb.checked = true; });
                return;
            }
            const deselectAllBtn = e.target.closest('[data-deselect-all]');
            if (deselectAllBtn) {
                const listId2 = deselectAllBtn.getAttribute('data-deselect-all');
                const container2 = root.querySelector('[data-checklist="' + listId2 + '"]');
                if (container2) container2.querySelectorAll('input[type="checkbox"]').forEach(function(cb) { cb.checked = false; });
                return;
            }
        });

        root.addEventListener('keydown', function(e) {
            if (e.key === 'Enter' && e.target.id === 'import-url') { e.preventDefault(); submitImport(); }
            if (e.key === 'Escape') hideImportModal();
        });
    };
})(window.AdminApp);

(function(app) {
    'use strict';

    function copyToClipboard(text, btn) {
        navigator.clipboard.writeText(text).then(function() {
            const orig = btn.textContent;
            btn.textContent = 'Copied!';
            btn.classList.add('copied');
            setTimeout(function() {
                btn.textContent = orig;
                btn.classList.remove('copied');
            }, 2000);
        }).catch(function() {
            app.Toast.show('Failed to copy to clipboard', 'error');
        });
    }

    const SAFE_PATH_RE = /^[a-zA-Z0-9_\-./]+$/;
    function safeDelimiter(idx) {
        return 'EOF_SP_' + idx;
    }
    function sanitizePath(p) {
        if (!SAFE_PATH_RE.test(p)) {
            throw new Error('Invalid file path: ' + p);
        }
        return p;
    }
    function generateInstallScript(data) {
        const lines = ['#!/bin/bash', '# Install script for Foodles plugins', 'set -e', ''];
        const plugins = data.plugins || [];
        let delimIdx = 0;
        for (let i = 0; i < plugins.length; i++) {
            const plugin = plugins[i];
            const files = plugin.files || [];
            const safeId = sanitizePath(plugin.id);
            lines.push('# Plugin: ' + safeId);
            lines.push('echo "Installing plugin: ' + safeId + '"');
            for (let j = 0; j < files.length; j++) {
                const file = files[j];
                const safePath = sanitizePath(file.path);
                const filePath = '~/.claude/plugins/' + safeId + '/' + safePath;
                const dirPath = filePath.substring(0, filePath.lastIndexOf('/'));
                const delim = safeDelimiter(delimIdx++);
                lines.push('mkdir -p "' + dirPath + '"');
                lines.push("cat > \"" + filePath + "\" << '" + delim + "'");
                lines.push(file.content);
                lines.push(delim);
                if (file.executable) {
                    lines.push('chmod +x "' + filePath + '"');
                }
                lines.push('');
            }
        }
        if (data.marketplace) {
            const mktPath = sanitizePath(data.marketplace.path);
            const mktDelim = safeDelimiter(delimIdx++);
            lines.push('# Marketplace manifest');
            lines.push('mkdir -p ~/.claude/plugins/.claude-plugin');
            lines.push("cat > ~/.claude/plugins/" + mktPath + " << '" + mktDelim + "'");
            lines.push(data.marketplace.content);
            lines.push(mktDelim);
        }
        lines.push('');
        lines.push('echo "All plugins installed successfully."');
        return lines.join('\n');
    }

    function toggleBundle(idx) {
        const details = document.getElementById('bundle-details-' + idx);
        const icon = document.getElementById('bundle-icon-' + idx);
        if (!details) return;
        const open = details.style.display !== 'none';
        details.style.display = open ? 'none' : 'block';
        if (icon) icon.innerHTML = open ? '&#9654;' : '&#9660;';
    }

    async function downloadZip(data) {
        const btn = document.getElementById('btn-download-zip');
        if (!btn) return;
        const origHtml = btn.innerHTML;
        btn.innerHTML = 'Generating...';
        btn.disabled = true;
        try {
            const JSZip = await app.shared.loadJSZip();
            const zip = new JSZip();
            const plugins = data.plugins || [];
            plugins.forEach(function(plugin) {
                const folder = zip.folder(plugin.id);
                (plugin.files || []).forEach(function(file) {
                    const opts = file.executable ? { unixPermissions: '755' } : {};
                    folder.file(file.path, file.content, opts);
                });
            });
            if (data.marketplace) {
                zip.file(data.marketplace.path, data.marketplace.content);
            }
            const blob = await zip.generateAsync({ type: 'blob', platform: 'UNIX' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = 'foodles-plugins.zip';
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
            btn.innerHTML = origHtml;
            btn.disabled = false;
            app.Toast.show('ZIP downloaded successfully', 'success');
        } catch (err) {
            btn.innerHTML = origHtml;
            btn.disabled = false;
            app.Toast.show('Failed to generate ZIP: ' + err.message, 'error');
        }
    }

    app.exportInteractions = function(exportData) {
        if (!exportData) return;

        app.events.on('click', '#btn-download-zip', function() {
            downloadZip(exportData);
        });

        app.events.on('click', '[data-action="toggle-bundle"]', function(e, el) {
            toggleBundle(el.getAttribute('data-idx'));
        });

        app.events.on('click', '[data-action="copy-content"]', function(e, el) {
            const pluginIdx = parseInt(el.getAttribute('data-plugin-idx'), 10);
            const fileIdx = parseInt(el.getAttribute('data-file-idx'), 10);
            const plugin = (exportData.plugins || [])[pluginIdx];
            if (plugin) {
                const file = (plugin.files || [])[fileIdx];
                if (file) copyToClipboard(file.content, el);
            }
        });

        app.events.on('click', '[data-action="copy-script"]', function(e, el) {
            const script = generateInstallScript(exportData);
            copyToClipboard(script, el);
        });
    };
})(window.AdminApp);

(function(app) {
    'use strict';

    function buildQueryString() {
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
    }

    app.auditInteractions = function() {
        app.events.on('click', '#btn-preview', function() {
            const qs = buildQueryString();
            window.location.href = app.BASE + '/audit/?' + qs;
        });

        app.events.on('click', '#btn-download', function() {
            const format = document.getElementById('audit-format').value;
            const downloadQs = buildQueryString();
            if (format === 'csv') {
                window.open(app.API_BASE + '/export/usage?' + downloadQs.replace(/from=([^&]+)/, function(m, v) { return 'from=' + v + 'T00:00:00Z'; }).replace(/to=([^&]+)/, function(m, v) { return 'to=' + v + 'T23:59:59Z'; }), '_blank');
            } else {
                app.api('/export/usage?' + downloadQs.replace(/from=([^&]+)/, function(m, v) { return 'from=' + v + 'T00:00:00Z'; }).replace(/to=([^&]+)/, function(m, v) { return 'to=' + v + 'T23:59:59Z'; })).then(function(rows) {
                    const blob = new Blob([JSON.stringify(rows, null, 2)], { type: 'application/json' });
                    const url = URL.createObjectURL(blob);
                    const a = document.createElement('a');
                    a.href = url;
                    a.download = 'usage-export.json';
                    a.click();
                    URL.revokeObjectURL(url);
                }).catch(function(err) {
                    app.Toast.show(err.message || 'Download failed', 'error');
                });
            }
        });
    };
})(window.AdminApp);

(function(app) {
    'use strict';

    const escapeHtml = app.escapeHtml;
    const CATEGORY_ICONS = {
        'First Steps': '\u26A1',
        'Milestones': '\uD83D\uDCCA',
        'Exploration': '\uD83D\uDD0D',
        'Creation': '\u2728',
        'Streaks': '\uD83D\uDD25',
        'Ranks': '\uD83C\uDFC6',
        'Special': '\u2B50'
    };
    function groupByCategory(items) {
        const groups = {};
        items.forEach(function(a) {
            const cat = a.category || 'Other';
            if (!groups[cat]) groups[cat] = [];
            groups[cat].push(a);
        });
        return groups;
    }
    function renderAchievementCard(a) {
        const unlocked = a.total_unlocked > 0;
        const cls = unlocked ? 'achievement-card unlocked' : 'achievement-card locked';
        const pct = unlocked ? 100 : (a.unlock_percentage || 0);
        const icon = CATEGORY_ICONS[a.category] || '\u2B50';
        const bar = '<div class="unlock-bar"><div class="unlock-bar-fill" style="width:' + pct + '%"></div></div>';
        return '<div class="' + cls + '">' +
            '<div class="achievement-icon">' + icon + '</div>' +
            '<div style="font-weight:600;font-size:var(--text-sm);color:var(--text-primary)">' + escapeHtml(a.name) + '</div>' +
            '<div style="font-size:var(--text-xs);color:var(--text-tertiary);margin-top:var(--space-1)">' + escapeHtml(a.description) + '</div>' +
            '<div style="font-size:var(--text-xs);color:var(--text-tertiary);margin-top:var(--space-1)">' + a.total_unlocked + ' unlocked</div>' +
            bar +
        '</div>';
    }
    function renderAchievementsContent(data) {
        const items = Array.isArray(data) ? data : (data.achievements || []);
        if (!items.length) {
            return '<div class="empty-state"><p>No achievements defined.</p></div>';
        }
        const groups = groupByCategory(items);
        const categories = Object.keys(groups);
        let html = '';
        categories.forEach(function(cat) {
            const cards = groups[cat].map(renderAchievementCard).join('');
            html += '<div style="margin-bottom:var(--space-6)">' +
                '<div class="section-title">' + escapeHtml(cat) + '</div>' +
                '<div class="achievement-grid">' + cards + '</div>' +
            '</div>';
        });
        return html;
    }
    app.renderAchievements = function() {
        const root = document.getElementById('achievements-content');
        if (!root) return;
        root.innerHTML = '<div class="loading-center"><div class="loading-spinner"></div></div>';
        app.api('/gamification/achievements').then(function(data) {
            root.innerHTML = renderAchievementsContent(data);
        }).catch(function(err) {
            root.innerHTML = '<div class="empty-state"><p>Failed to load achievements.</p></div>';
            app.Toast.show(err.message || 'Failed to load achievements', 'error');
        });
    };
})(window.AdminApp);

(function(app) {
    'use strict';

    function getSkillDetail(skillId) {
        const el = document.querySelector('script[data-skill-detail="' + skillId + '"]');
        if (!el) return null;
        try { return JSON.parse(el.textContent); } catch (e) { return null; }
    }

    function getAllPlugins() {
        const el = document.getElementById('all-plugins-data');
        if (!el) return [];
        try { return JSON.parse(el.textContent) || []; } catch (e) { return []; }
    }

    function renderSkillExpand(skillId) {
        const data = getSkillDetail(skillId);
        if (!data) return '<p class="text-muted">No detail data available.</p>';

        let html = '<div class="detail-section">';
        html += '<strong>Description</strong>';
        html += '<p style="margin:var(--space-1) 0;color:var(--text-secondary);font-size:var(--text-sm)">' + app.escapeHtml(data.description || 'No description') + '</p>';
        html += '</div>';

        if (data.command) {
            html += '<div class="detail-section">';
            html += '<strong>Command</strong>';
            html += '<pre style="margin:var(--space-1) 0;font-size:var(--text-xs);background:var(--bg-surface-raised);padding:var(--space-2);border-radius:var(--radius-sm);overflow-x:auto">' + app.escapeHtml(data.command) + '</pre>';
            html += '</div>';
        }

        if (data.tags && data.tags.length) {
            html += '<div class="detail-section">';
            html += '<strong>Tags</strong><br>';
            html += '<div class="badge-row" style="margin-top:var(--space-1)">';
            data.tags.forEach(function(tag) {
                html += '<span class="badge badge-gray">' + app.escapeHtml(tag) + '</span>';
            });
            html += '</div></div>';
        }

        html += '<div class="detail-section">';
        html += '<details><summary style="cursor:pointer;font-size:var(--text-sm);color:var(--text-secondary)">JSON Config</summary>';
        html += app.OrgCommon.formatJson(data);
        html += '</details></div>';

        return html;
    }

    function initExpandRows() {
        app.OrgCommon.initExpandRows('.data-table', function(row, detailRow) {
            var content = detailRow.querySelector('[data-skill-expand]');
            if (content && !content.hasAttribute('data-loaded')) {
                var skillId = content.getAttribute('data-skill-expand');
                content.innerHTML = renderSkillExpand(skillId);
                content.setAttribute('data-loaded', 'true');
            }
        });
    }

    function initDeleteHandlers() {
        document.addEventListener('click', function(e) {
            const btn = e.target.closest('[data-delete-skill]');
            if (!btn) return;
            const skillId = btn.getAttribute('data-delete-skill');
            if (!confirm('Are you sure you want to delete skill "' + skillId + '"? This cannot be undone.')) return;

            fetch('/api/admin/skills/' + encodeURIComponent(skillId), { method: 'DELETE' })
                .then(function(res) {
                    if (res.ok) {
                        app.Toast.show('Skill deleted', 'success');
                        setTimeout(function() { window.location.reload(); }, 500);
                    } else {
                        app.Toast.show('Failed to delete skill', 'error');
                    }
                })
                .catch(function() {
                    app.Toast.show('Failed to delete skill', 'error');
                });
        });
    }

    function initForkHandlers() {
        document.addEventListener('click', function(e) {
            const btn = e.target.closest('[data-fork-skill]');
            if (!btn) return;
            const skillId = btn.getAttribute('data-fork-skill');
            const data = getSkillDetail(skillId);
            if (!data) return;

            const newId = prompt('Enter a new ID for the customized skill:', skillId + '-custom');
            if (!newId) return;

            fetch('/api/admin/skills', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    skill_id: newId,
                    name: (data.name || skillId) + ' (Custom)',
                    description: data.description || '',
                    base_skill_id: skillId
                })
            })
            .then(function(res) {
                if (res.ok) {
                    app.Toast.show('Skill customized', 'success');
                    setTimeout(function() { window.location.reload(); }, 500);
                } else {
                    app.Toast.show('Failed to customize skill', 'error');
                }
            })
            .catch(function() {
                app.Toast.show('Failed to customize skill', 'error');
            });
        });
    }

    function initAssignPanel() {
        const allPlugins = getAllPlugins();
        const assignApi = app.OrgCommon.initAssignPanel({
            panelId: 'assign-panel',
            allPlugins: allPlugins
        });
        if (!assignApi) return;

        document.addEventListener('click', function(e) {
            const btn = e.target.closest('[data-assign-skill]');
            if (!btn) return;
            const skillId = btn.getAttribute('data-assign-skill');
            const skillName = btn.getAttribute('data-skill-name') || skillId;
            const data = getSkillDetail(skillId);
            const currentPluginIds = data && data.assigned_plugin_ids ? data.assigned_plugin_ids : [];
            assignApi.open(skillId, skillName, currentPluginIds);
        });

        document.addEventListener('click', function(e) {
            const btn = e.target.closest('[data-assign-save]');
            if (!btn) return;
            const entityId = btn.getAttribute('data-entity-id');
            const checkboxes = document.querySelectorAll('#assign-panel input[name="plugin_id"]');
            const selectedPlugins = [];
            checkboxes.forEach(function(cb) {
                if (cb.checked) selectedPlugins.push(cb.value);
            });

            btn.disabled = true;
            btn.textContent = 'Saving...';

            const promises = allPlugins.map(function(plugin) {
                return fetch('/api/admin/plugins/' + encodeURIComponent(plugin.id) + '/skills')
                    .then(function(res) { return res.json(); })
                    .then(function(currentSkills) {
                        let skillIds = (currentSkills || []).slice();
                        const shouldInclude = selectedPlugins.indexOf(plugin.id) !== -1;
                        const hasSkill = skillIds.indexOf(entityId) !== -1;

                        if (shouldInclude && !hasSkill) {
                            skillIds.push(entityId);
                        } else if (!shouldInclude && hasSkill) {
                            skillIds = skillIds.filter(function(s) { return s !== entityId; });
                        } else {
                            return Promise.resolve();
                        }

                        return fetch('/api/admin/plugins/' + encodeURIComponent(plugin.id) + '/skills', {
                            method: 'PUT',
                            headers: { 'Content-Type': 'application/json' },
                            body: JSON.stringify({ skill_ids: skillIds })
                        });
                    });
            });

            Promise.all(promises)
                .then(function() {
                    app.Toast.show('Plugin assignments updated', 'success');
                    assignApi.close();
                    setTimeout(function() { window.location.reload(); }, 500);
                })
                .catch(function() {
                    app.Toast.show('Failed to update assignments', 'error');
                    btn.disabled = false;
                    btn.textContent = 'Save';
                });
        });
    }

    function initEditPanel() {
        var editPanel = app.OrgCommon.initEditPanel({
            panelId: 'edit-panel',
            entityLabel: 'Skill',
            apiBasePath: '/api/public/skills/',
            idField: 'skill_id',
            fields: [
                { name: 'name', label: 'Name', type: 'text', required: true },
                { name: 'description', label: 'Description', type: 'text' },
                { name: 'content', label: 'Content', type: 'textarea', rows: 15 },
                { name: 'tags', label: 'Tags (comma-separated)', type: 'text' },
                { name: 'category_id', label: 'Category', type: 'text' }
            ]
        });

        document.addEventListener('click', function(e) {
            var btn = e.target.closest('[data-edit-skill]');
            if (!btn) return;
            var skillId = btn.getAttribute('data-edit-skill');
            var data = getSkillDetail(skillId);
            if (data && editPanel) editPanel.open(skillId, data);
        });
    }

    function initBulkHandlers() {
        var bulk = app.OrgCommon.initBulkActions('.data-table', 'bulk-bar');
        if (!bulk) return;

        var allPlugins = getAllPlugins();
        var assignApi = app.OrgCommon.initAssignPanel({
            panelId: 'assign-panel',
            allPlugins: allPlugins
        });

        // Bulk delete
        var deleteBtn = document.getElementById('bulk-delete-btn');
        if (deleteBtn) {
            deleteBtn.addEventListener('click', function() {
                var ids = bulk.getSelected();
                if (!ids.length) return;
                if (!confirm('Delete ' + ids.length + ' skill(s)? This action cannot be undone.')) return;
                Promise.all(ids.map(function(id) {
                    return fetch('/api/admin/skills/' + encodeURIComponent(id), { method: 'DELETE' });
                })).then(function() {
                    app.Toast.show(ids.length + ' skills deleted', 'success');
                    setTimeout(function() { window.location.reload(); }, 500);
                }).catch(function() {
                    app.Toast.show('Failed to delete some skills', 'error');
                });
            });
        }

        // Bulk assign to plugin
        var assignBtn = document.getElementById('bulk-assign-btn');
        if (assignBtn && assignApi) {
            assignBtn.addEventListener('click', function() {
                var ids = bulk.getSelected();
                if (!ids.length) return;
                assignApi.open(ids.join(','), ids.length + ' skills', []);
            });
        }

        // Bulk set category
        var categoryBtn = document.getElementById('bulk-category-btn');
        if (categoryBtn) {
            categoryBtn.addEventListener('click', function() {
                var ids = bulk.getSelected();
                if (!ids.length) return;
                var category = prompt('Enter category for ' + ids.length + ' skill(s):');
                if (category === null) return;
                Promise.all(ids.map(function(id) {
                    return fetch('/api/public/skills/' + encodeURIComponent(id), {
                        method: 'PUT',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ category_id: category })
                    });
                })).then(function() {
                    app.Toast.show('Category updated for ' + ids.length + ' skills', 'success');
                    setTimeout(function() { window.location.reload(); }, 500);
                }).catch(function() {
                    app.Toast.show('Failed to update category', 'error');
                });
            });
        }
    }

    app.initOrgSkills = function() {
        initExpandRows();
        app.OrgCommon.initFilters('skill-search', '.data-table', [
            { selectId: 'source-filter', dataAttr: 'data-source' },
            { selectId: 'plugin-filter', dataAttr: 'data-plugins' },
            { selectId: 'tag-filter', dataAttr: 'data-tags' }
        ]);
        app.OrgCommon.initTimeAgo();
        initDeleteHandlers();
        initForkHandlers();
        initAssignPanel();
        initEditPanel();
        initBulkHandlers();
    };

})(window.AdminApp = window.AdminApp || {});

(function(app) {
    'use strict';

    function getAgentDetail(agentId) {
        const el = document.querySelector('script[data-agent-detail="' + agentId + '"]');
        if (!el) return null;
        try { return JSON.parse(el.textContent); } catch (e) { return null; }
    }

    function getAllPlugins() {
        const el = document.getElementById('all-plugins-data');
        if (!el) return [];
        try { return JSON.parse(el.textContent) || []; } catch (e) { return []; }
    }

    function renderAgentExpand(agentId) {
        const data = getAgentDetail(agentId);
        if (!data) return '<p class="text-muted">No detail data available.</p>';

        let html = '';

        if (data.system_prompt) {
            html += '<div class="detail-section">';
            html += '<strong>System Prompt</strong>';
            html += '<pre style="margin:var(--space-1) 0;max-height:200px;overflow:auto;font-size:var(--text-xs);background:var(--bg-surface-raised);padding:var(--space-2);border-radius:var(--radius-sm);white-space:pre-wrap;word-break:break-word">' + app.escapeHtml(data.system_prompt) + '</pre>';
            html += '</div>';
        }

        if (data.port || data.endpoint) {
            html += '<div class="detail-section">';
            html += '<strong>Connection</strong>';
            html += '<div style="margin:var(--space-1) 0;font-size:var(--text-sm);color:var(--text-secondary)">';
            if (data.port) html += '<div>Port: <code class="code-inline">' + app.escapeHtml(String(data.port)) + '</code></div>';
            if (data.endpoint) html += '<div>Endpoint: <code class="code-inline">' + app.escapeHtml(data.endpoint) + '</code></div>';
            html += '</div></div>';
        }

        if ((data.skill_count && data.skill_count > 0) || (data.mcp_count && data.mcp_count > 0)) {
            html += '<div class="detail-section">';
            html += '<strong>Capabilities</strong>';
            html += '<div class="badge-row" style="margin-top:var(--space-1)">';
            if (data.skill_count > 0) {
                html += '<span class="badge badge-green">' + data.skill_count + ' skill' + (data.skill_count !== 1 ? 's' : '') + '</span>';
            }
            if (data.mcp_count > 0) {
                html += '<span class="badge badge-yellow">' + data.mcp_count + ' MCP server' + (data.mcp_count !== 1 ? 's' : '') + '</span>';
            }
            html += '</div></div>';
        }

        html += '<div class="detail-section">';
        html += '<details><summary style="cursor:pointer;font-size:var(--text-sm);color:var(--text-secondary)">JSON Config</summary>';
        html += app.OrgCommon.formatJson(data);
        html += '</details></div>';

        return html;
    }

    function initExpandRows() {
        app.OrgCommon.initExpandRows('.data-table', function(row, detailRow) {
            var content = detailRow.querySelector('[data-agent-expand]');
            if (content && !content.hasAttribute('data-loaded')) {
                var agentId = content.getAttribute('data-agent-expand');
                content.innerHTML = renderAgentExpand(agentId);
                content.setAttribute('data-loaded', 'true');
            }
        });
    }

    function initDeleteHandlers() {
        document.addEventListener('click', function(e) {
            const btn = e.target.closest('[data-delete-agent]');
            if (!btn) return;
            const agentId = btn.getAttribute('data-delete-agent');
            if (!confirm('Are you sure you want to delete agent "' + agentId + '"? This cannot be undone.')) return;

            fetch('/api/admin/agents/' + encodeURIComponent(agentId), { method: 'DELETE' })
                .then(function(res) {
                    if (res.ok) {
                        app.Toast.show('Agent deleted', 'success');
                        setTimeout(function() { window.location.reload(); }, 500);
                    } else {
                        app.Toast.show('Failed to delete agent', 'error');
                    }
                })
                .catch(function() {
                    app.Toast.show('Failed to delete agent', 'error');
                });
        });
    }

    function initForkHandlers() {
        document.addEventListener('click', function(e) {
            const btn = e.target.closest('[data-fork-agent]');
            if (!btn) return;
            const agentId = btn.getAttribute('data-fork-agent');
            const data = getAgentDetail(agentId);
            if (!data) return;

            const newId = prompt('Enter a new ID for the customized agent:', agentId + '-custom');
            if (!newId) return;

            fetch('/api/admin/agents', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    id: newId,
                    name: (data.name || agentId) + ' (Custom)',
                    description: data.description || '',
                    system_prompt: data.system_prompt || '',
                    enabled: true
                })
            })
            .then(function(res) {
                if (res.ok) {
                    app.Toast.show('Agent customized', 'success');
                    setTimeout(function() { window.location.reload(); }, 500);
                } else {
                    app.Toast.show('Failed to customize agent', 'error');
                }
            })
            .catch(function() {
                app.Toast.show('Failed to customize agent', 'error');
            });
        });
    }

    function initAssignPanel() {
        const allPlugins = getAllPlugins();
        const assignApi = app.OrgCommon.initAssignPanel({
            panelId: 'assign-panel',
            allPlugins: allPlugins
        });
        if (!assignApi) return;

        document.addEventListener('click', function(e) {
            const btn = e.target.closest('[data-assign-agent]');
            if (!btn) return;
            const agentId = btn.getAttribute('data-assign-agent');
            const agentName = btn.getAttribute('data-agent-name') || agentId;
            const data = getAgentDetail(agentId);
            const currentPluginIds = data && data.assigned_plugin_ids ? data.assigned_plugin_ids : [];
            assignApi.open(agentId, agentName, currentPluginIds);
        });

        document.addEventListener('click', function(e) {
            const btn = e.target.closest('[data-assign-save]');
            if (!btn) return;
            const entityId = btn.getAttribute('data-entity-id');
            const checkboxes = document.querySelectorAll('#assign-panel input[name="plugin_id"]');
            const selectedPlugins = [];
            checkboxes.forEach(function(cb) {
                if (cb.checked) selectedPlugins.push(cb.value);
            });

            btn.disabled = true;
            btn.textContent = 'Saving...';

            const promises = allPlugins.map(function(plugin) {
                return fetch('/api/admin/plugins/' + encodeURIComponent(plugin.id) + '/agents')
                    .then(function(res) { return res.json(); })
                    .then(function(currentAgents) {
                        let agentIds = (currentAgents || []).slice();
                        const shouldInclude = selectedPlugins.indexOf(plugin.id) !== -1;
                        const hasAgent = agentIds.indexOf(entityId) !== -1;

                        if (shouldInclude && !hasAgent) {
                            agentIds.push(entityId);
                        } else if (!shouldInclude && hasAgent) {
                            agentIds = agentIds.filter(function(a) { return a !== entityId; });
                        } else {
                            return Promise.resolve();
                        }

                        return fetch('/api/admin/plugins/' + encodeURIComponent(plugin.id) + '/agents', {
                            method: 'PUT',
                            headers: { 'Content-Type': 'application/json' },
                            body: JSON.stringify({ agent_ids: agentIds })
                        });
                    });
            });

            Promise.all(promises)
                .then(function() {
                    app.Toast.show('Plugin assignments updated', 'success');
                    assignApi.close();
                    setTimeout(function() { window.location.reload(); }, 500);
                })
                .catch(function() {
                    app.Toast.show('Failed to update assignments', 'error');
                    btn.disabled = false;
                    btn.textContent = 'Save';
                });
        });
    }

    function initEditPanel() {
        var editPanel = app.OrgCommon.initEditPanel({
            panelId: 'edit-panel',
            entityLabel: 'Agent',
            apiBasePath: '/api/public/agents/',
            idField: 'id',
            fields: [
                { name: 'name', label: 'Name', type: 'text', required: true },
                { name: 'description', label: 'Description', type: 'text' },
                { name: 'system_prompt', label: 'System Prompt', type: 'textarea', rows: 15 }
            ]
        });

        document.addEventListener('click', function(e) {
            var btn = e.target.closest('[data-edit-agent]');
            if (!btn) return;
            var agentId = btn.getAttribute('data-edit-agent');
            var data = getAgentDetail(agentId);
            if (data && editPanel) editPanel.open(agentId, data);
        });
    }

    function initBulkHandlers() {
        var bulk = app.OrgCommon.initBulkActions('.data-table', 'bulk-bar');
        if (!bulk) return;

        var allPlugins = getAllPlugins();
        var assignApi = app.OrgCommon.initAssignPanel({
            panelId: 'assign-panel',
            allPlugins: allPlugins
        });

        // Bulk delete
        var deleteBtn = document.getElementById('bulk-delete-btn');
        if (deleteBtn) {
            deleteBtn.addEventListener('click', function() {
                var ids = bulk.getSelected();
                if (!ids.length) return;
                if (!confirm('Delete ' + ids.length + ' agent(s)? This action cannot be undone.')) return;
                Promise.all(ids.map(function(id) {
                    return fetch('/api/admin/agents/' + encodeURIComponent(id), { method: 'DELETE' });
                })).then(function() {
                    app.Toast.show(ids.length + ' agents deleted', 'success');
                    setTimeout(function() { window.location.reload(); }, 500);
                }).catch(function() {
                    app.Toast.show('Failed to delete some agents', 'error');
                });
            });
        }

        // Bulk assign to plugin
        var assignBtn = document.getElementById('bulk-assign-btn');
        if (assignBtn && assignApi) {
            assignBtn.addEventListener('click', function() {
                var ids = bulk.getSelected();
                if (!ids.length) return;
                assignApi.open(ids.join(','), ids.length + ' agents', []);
            });
        }
    }

    app.initOrgAgents = function() {
        initExpandRows();
        app.OrgCommon.initFilters('agent-search', '.data-table', [
            { selectId: 'plugin-filter', dataAttr: 'data-plugins' }
        ]);
        app.OrgCommon.initTimeAgo();
        initDeleteHandlers();
        initForkHandlers();
        initAssignPanel();
        initEditPanel();
        initBulkHandlers();
    };

})(window.AdminApp = window.AdminApp || {});

(function(app) {
    'use strict';

    app.initOrgHooks = function() {
        const OrgCommon = app.OrgCommon;
        if (!OrgCommon) return;

        OrgCommon.initExpandRows('.data-table');

        const searchInput = document.getElementById('hook-search');
        if (searchInput) {
            searchInput.addEventListener('input', function() {
                const query = this.value.toLowerCase();
                const rows = document.querySelectorAll('.data-table tbody tr.clickable-row');
                rows.forEach(function(row) {
                    const name = (row.getAttribute('data-name') || '').toLowerCase();
                    const match = !query || name.indexOf(query) !== -1;
                    row.style.display = match ? '' : 'none';
                    const detail = row.nextElementSibling;
                    if (detail && detail.classList.contains('detail-row')) {
                        if (!match) {
                            detail.classList.remove('visible');
                            detail.style.display = 'none';
                        } else {
                            detail.style.display = '';
                        }
                    }
                });
            });
        }

        document.addEventListener('click', function(e) {
            const toggle = e.target.closest('[data-hook-json]');
            if (!toggle) return;
            const hookId = toggle.getAttribute('data-hook-json');
            const container = document.querySelector('[data-hook-json-container="' + hookId + '"]');
            if (!container) return;

            if (container.style.display === 'none') {
                const script = document.querySelector('script[data-hook-detail="' + hookId + '"]');
                if (script) {
                    try {
                        const data = JSON.parse(script.textContent);
                        container.innerHTML = OrgCommon.formatJson(data);
                    } catch (err) {
                        container.innerHTML = '<span class="text-muted">Failed to parse JSON</span>';
                    }
                }
                container.style.display = 'block';
                toggle.textContent = 'Hide JSON';
            } else {
                container.style.display = 'none';
                toggle.textContent = 'Show JSON';
            }
        });

        document.addEventListener('click', function(e) {
            const deleteBtn = e.target.closest('[data-action="delete"][data-entity-type="hook"]');
            if (!deleteBtn) return;
            const id = deleteBtn.getAttribute('data-entity-id');
            if (!confirm('Delete this hook? This cannot be undone.')) return;

            app.api('/hooks/' + encodeURIComponent(id), {
                method: 'DELETE'
            }).then(function() {
                app.Toast.show('Hook deleted', 'success');
                const row = document.querySelector('tr[data-entity-id="' + id + '"].clickable-row');
                if (row) {
                    const detail = row.nextElementSibling;
                    if (detail && detail.classList.contains('detail-row')) {
                        detail.remove();
                    }
                    row.remove();
                }
            }).catch(function(err) {
                app.Toast.show(err.message || 'Failed to delete hook', 'error');
            });
        });

        document.addEventListener('click', function(e) {
            const detailsBtn = e.target.closest('[data-hook-details]');
            if (!detailsBtn) return;
            const hookId = detailsBtn.getAttribute('data-hook-details');
            const row = document.querySelector('tr[data-entity-id="' + hookId + '"].clickable-row');
            if (!row) return;
            const detailRow = row.nextElementSibling;
            if (!detailRow || !detailRow.classList.contains('detail-row')) return;
            OrgCommon.handleRowClick(row, detailRow);
        });
    };

})(window.AdminApp = window.AdminApp || {});

(function(app) {
    'use strict';

    app.initOrgMcpServers = function() {
        const OrgCommon = app.OrgCommon;
        if (!OrgCommon) return;

        OrgCommon.initExpandRows('.data-table');

        const searchInput = document.getElementById('mcp-search');
        if (searchInput) {
            searchInput.addEventListener('input', function() {
                const query = this.value.toLowerCase();
                const rows = document.querySelectorAll('.data-table tbody tr.clickable-row');
                rows.forEach(function(row) {
                    const name = (row.getAttribute('data-name') || '').toLowerCase();
                    const match = !query || name.indexOf(query) !== -1;
                    row.style.display = match ? '' : 'none';
                    const detail = row.nextElementSibling;
                    if (detail && detail.classList.contains('detail-row')) {
                        if (!match) {
                            detail.classList.remove('visible');
                            detail.style.display = 'none';
                        } else {
                            detail.style.display = '';
                        }
                    }
                });
            });
        }

        document.addEventListener('click', function(e) {
            const toggle = e.target.closest('[data-mcp-json]');
            if (!toggle) return;
            const mcpId = toggle.getAttribute('data-mcp-json');
            const container = document.querySelector('[data-mcp-json-container="' + mcpId + '"]');
            if (!container) return;

            if (container.style.display === 'none') {
                const script = document.querySelector('script[data-mcp-detail="' + mcpId + '"]');
                if (script) {
                    try {
                        const data = JSON.parse(script.textContent);
                        container.innerHTML = OrgCommon.formatJson(data);
                    } catch (err) {
                        container.innerHTML = '<span class="text-muted">Failed to parse JSON</span>';
                    }
                }
                container.style.display = 'block';
                toggle.textContent = 'Hide JSON';
            } else {
                container.style.display = 'none';
                toggle.textContent = 'Show JSON';
            }
        });

        document.addEventListener('click', function(e) {
            const deleteBtn = e.target.closest('[data-action="delete"][data-entity-type="mcp-server"]');
            if (!deleteBtn) return;
            const id = deleteBtn.getAttribute('data-entity-id');
            if (!confirm('Delete MCP server "' + id + '"? This cannot be undone.')) return;

            app.api('/mcp-servers/' + encodeURIComponent(id), {
                method: 'DELETE'
            }).then(function() {
                app.Toast.show('MCP server deleted', 'success');
                const row = document.querySelector('tr[data-entity-id="' + id + '"].clickable-row');
                if (row) {
                    const detail = row.nextElementSibling;
                    if (detail && detail.classList.contains('detail-row')) {
                        detail.remove();
                    }
                    row.remove();
                }
            }).catch(function(err) {
                app.Toast.show(err.message || 'Failed to delete MCP server', 'error');
            });
        });

        let allPlugins = [];
        document.querySelectorAll('script[data-mcp-detail]').forEach(function(script) {
            try {
                const data = JSON.parse(script.textContent);
                if (data.assigned_plugins) {
                    data.assigned_plugins.forEach(function(p) {
                        if (!allPlugins.some(function(existing) { return existing.id === p.id; })) {
                            allPlugins.push(p);
                        }
                    });
                }
            } catch (e) {}
        });

        const assignPanel = OrgCommon.initAssignPanel({
            panelId: 'assign-panel',
            allPlugins: allPlugins
        });

        document.addEventListener('click', function(e) {
            const assignBtn = e.target.closest('[data-assign-mcp]');
            if (!assignBtn) return;
            const mcpId = assignBtn.getAttribute('data-assign-mcp');
            const mcpName = assignBtn.getAttribute('data-mcp-name') || mcpId;

            let currentPluginIds = [];
            const script = document.querySelector('script[data-mcp-detail="' + mcpId + '"]');
            if (script) {
                try {
                    const data = JSON.parse(script.textContent);
                    if (data.assigned_plugins) {
                        currentPluginIds = data.assigned_plugins.map(function(p) { return p.id; });
                    }
                } catch (e) {}
            }

            if (assignPanel) {
                assignPanel.open(mcpId, mcpName, currentPluginIds);
            }
        });
    };

})(window.AdminApp = window.AdminApp || {});

(function(app) {
    'use strict';

    let activeTab = 'plugins';
    let selectedEntities = {};
    let currentPanelEntity = null;

    app.initAccessControl = function() {
        initTabs();
        initSearch();
        initFilters();
        initRowClicks();
        initCheckboxes();
        initSidePanel();
        initBulkPanel();
        updateCoverage();
    };

    function initTabs() {
        const tabBar = document.getElementById('acl-tabs');
        if (!tabBar) return;
        tabBar.addEventListener('click', function(e) {
            const btn = e.target.closest('[data-acl-tab]');
            if (!btn) return;
            activeTab = btn.getAttribute('data-acl-tab');
            tabBar.querySelectorAll('.tab-btn').forEach(function(b) {
                b.classList.toggle('active', b === btn);
            });
            document.querySelectorAll('[data-acl-panel]').forEach(function(panel) {
                panel.style.display = panel.getAttribute('data-acl-panel') === activeTab ? '' : 'none';
            });
            clearSelection();
            updateCoverage();
        });
    }

    function initSearch() {
        const input = document.getElementById('acl-search');
        if (!input) return;
        input.addEventListener('input', debounce(function() {
            filterRows();
        }, 200));
    }

    function initFilters() {
        const roleFilter = document.getElementById('acl-role-filter');
        const deptFilter = document.getElementById('acl-dept-filter');
        if (roleFilter) roleFilter.addEventListener('change', filterRows);
        if (deptFilter) deptFilter.addEventListener('change', filterRows);
    }

    function filterRows() {
        const query = (document.getElementById('acl-search').value || '').toLowerCase();
        const roleFilter = document.getElementById('acl-role-filter').value;
        const deptFilter = document.getElementById('acl-dept-filter').value;

        const panel = document.querySelector('[data-acl-panel="' + activeTab + '"]');
        if (!panel) return;

        panel.querySelectorAll('.acl-entity-row').forEach(function(row) {
            const name = row.getAttribute('data-name') || '';
            const matchesSearch = !query || name.indexOf(query) !== -1;

            let matchesRole = true;
            let matchesDept = true;

            if (roleFilter || deptFilter) {
                const entityType = row.getAttribute('data-entity-type');
                const entityId = row.getAttribute('data-entity-id');
                const data = getEntityData(entityType, entityId);
                if (data) {
                    if (roleFilter) {
                        matchesRole = data.roles && data.roles.some(function(r) {
                            return r.name === roleFilter && r.assigned;
                        });
                    }
                    if (deptFilter) {
                        matchesDept = data.departments && data.departments.some(function(d) {
                            return d.name === deptFilter && d.assigned;
                        });
                    }
                }
            }

            row.style.display = (matchesSearch && matchesRole && matchesDept) ? '' : 'none';
        });
    }

    function initRowClicks() {
        app.events.on('click', '.acl-entity-row', function(e, row) {
            if (e.target.closest('[data-no-row-click]') || e.target.tagName === 'INPUT') return;
            const entityType = row.getAttribute('data-entity-type');
            const entityId = row.getAttribute('data-entity-id');
            openSidePanel(entityType, entityId);
        });
    }

    function initCheckboxes() {
        app.events.on('change', '.acl-select-all', function(e, selectAll) {
            const tabTarget = selectAll.getAttribute('data-acl-tab-target');
            const panel = document.querySelector('[data-acl-panel="' + tabTarget + '"]');
            if (!panel) return;
            panel.querySelectorAll('.acl-entity-checkbox').forEach(function(cb) {
                cb.checked = selectAll.checked;
                const key = cb.getAttribute('data-entity-type') + ':' + cb.getAttribute('data-entity-id');
                if (cb.checked) {
                    selectedEntities[key] = true;
                } else {
                    delete selectedEntities[key];
                }
            });
            updateSelectionCount();
        });

        app.events.on('change', '.acl-entity-checkbox', function(e, cb) {
            const key = cb.getAttribute('data-entity-type') + ':' + cb.getAttribute('data-entity-id');
            if (cb.checked) {
                selectedEntities[key] = true;
            } else {
                delete selectedEntities[key];
            }
            updateSelectionCount();
        });
    }

    function clearSelection() {
        selectedEntities = {};
        document.querySelectorAll('.acl-entity-checkbox, .acl-select-all').forEach(function(cb) {
            cb.checked = false;
        });
        updateSelectionCount();
    }

    function updateSelectionCount() {
        const count = Object.keys(selectedEntities).length;
        const el = document.getElementById('acl-selection-count');
        if (el) el.textContent = count;
        const btn = document.getElementById('acl-bulk-assign');
        if (btn) btn.disabled = count === 0;
    }

    function initSidePanel() {
        const closeBtn = document.getElementById('acl-panel-close');
        const cancelBtn = document.getElementById('acl-panel-cancel');
        const overlay = document.getElementById('acl-overlay');
        const saveBtn = document.getElementById('acl-panel-save');

        if (closeBtn) closeBtn.addEventListener('click', closeSidePanel);
        if (cancelBtn) cancelBtn.addEventListener('click', closeSidePanel);
        if (overlay) overlay.addEventListener('click', closeSidePanel);
        if (saveBtn) saveBtn.addEventListener('click', savePanelRules);
    }

    function openSidePanel(entityType, entityId) {
        const data = getEntityData(entityType, entityId);
        if (!data) return;

        currentPanelEntity = { type: entityType, id: entityId };

        const title = document.getElementById('acl-panel-title');
        if (title) title.textContent = data.name || data.id;

        const body = document.getElementById('acl-panel-body');
        if (body) body.innerHTML = buildPanelContent(data, entityType);

        if (body) {
            body.querySelectorAll('input[name="department"]').forEach(function(cb) {
                cb.addEventListener('change', function() {
                    const row = cb.closest('.acl-dept-row');
                    if (!row) return;
                    const toggle = row.querySelector('.acl-default-toggle');
                    const defaultCb = row.querySelector('input[name="default_included"]');
                    if (toggle) toggle.classList.toggle('disabled', !cb.checked);
                    if (defaultCb) defaultCb.disabled = !cb.checked;
                    if (!cb.checked && defaultCb) defaultCb.checked = false;
                });
            });
        }

        document.getElementById('acl-overlay').classList.add('active');
        document.getElementById('acl-detail-panel').classList.add('open');
    }

    function closeSidePanel() {
        currentPanelEntity = null;
        const overlay = document.getElementById('acl-overlay');
        const panel = document.getElementById('acl-detail-panel');
        if (overlay) overlay.classList.remove('active');
        if (panel) panel.classList.remove('open');
    }

    function buildPanelContent(entity, entityType) {
        let html = '';

        html += '<div class="acl-entity-info">';
        html += '<div class="cell-primary">' + escapeHtml(entity.name || entity.id) + '</div>';
        if (entity.description) {
            html += '<div class="cell-secondary">' + escapeHtml(entity.description) + '</div>';
        }
        html += '<div style="margin-top:var(--space-2)">';
        html += '<span class="badge badge-blue">' + escapeHtml(entityType.replace('_', ' ')) + '</span> ';
        html += entity.enabled ? '<span class="badge badge-green">Active</span>' : '<span class="badge badge-gray">Disabled</span>';
        html += '</div></div>';

        html += '<div class="acl-panel-section">';
        html += '<h3 class="acl-panel-section-title">Roles</h3>';
        html += '<p class="acl-panel-section-desc">Select which roles can access this entity. Empty means accessible to all.</p>';
        if (entity.roles && entity.roles.length) {
            entity.roles.forEach(function(role) {
                html += '<label class="acl-checkbox-row">' +
                    '<input type="checkbox" name="role" value="' + escapeHtml(role.name) + '"' +
                    (role.assigned ? ' checked' : '') + '>' +
                    '<span class="acl-checkbox-label">' + escapeHtml(role.name) + '</span>' +
                    '</label>';
            });
        } else {
            html += '<p style="color:var(--text-tertiary);font-size:var(--text-sm)">No roles defined.</p>';
        }
        html += '</div>';

        html += '<div class="acl-panel-section">';
        html += '<h3 class="acl-panel-section-title">Departments</h3>';
        html += '<p class="acl-panel-section-desc">Assign to departments. "Default" means auto-enabled and enforced for all department members.</p>';
        if (entity.departments && entity.departments.length) {
            entity.departments.forEach(function(dept) {
                const assigned = dept.assigned;
                const defaultIncluded = dept.default_included;
                html += '<div class="acl-dept-row">' +
                    '<label class="acl-checkbox-row" style="flex:1">' +
                    '<input type="checkbox" name="department" value="' + escapeHtml(dept.name) + '"' +
                    (assigned ? ' checked' : '') + '>' +
                    '<span class="acl-checkbox-label">' + escapeHtml(dept.name) +
                    ' <span class="acl-dept-count">(' + dept.user_count + ' members)</span></span>' +
                    '</label>' +
                    '<label class="acl-default-toggle' + (assigned ? '' : ' disabled') + '">' +
                    '<input type="checkbox" name="default_included" value="' + escapeHtml(dept.name) + '"' +
                    (defaultIncluded ? ' checked' : '') +
                    (!assigned ? ' disabled' : '') + '>' +
                    '<span class="acl-toggle-label">Default</span>' +
                    '</label>' +
                    '</div>';
            });
        } else {
            html += '<p style="color:var(--text-tertiary);font-size:var(--text-sm)">No departments found. Create users with departments first.</p>';
        }
        html += '</div>';

        return html;
    }

    function savePanelRules() {
        if (!currentPanelEntity) return;

        const body = document.getElementById('acl-panel-body');
        if (!body) return;

        const rules = [];

        body.querySelectorAll('input[name="role"]:checked').forEach(function(cb) {
            rules.push({
                rule_type: 'role',
                rule_value: cb.value,
                access: 'allow',
                default_included: false
            });
        });

        body.querySelectorAll('input[name="department"]:checked').forEach(function(cb) {
            const deptName = cb.value;
            const defaultCb = body.querySelector('input[name="default_included"][value="' + deptName + '"]');
            rules.push({
                rule_type: 'department',
                rule_value: deptName,
                access: 'allow',
                default_included: defaultCb ? defaultCb.checked : false
            });
        });

        const entityType = currentPanelEntity.type;
        const entityId = currentPanelEntity.id;

        const saveBtn = document.getElementById('acl-panel-save');
        if (saveBtn) {
            saveBtn.disabled = true;
            saveBtn.textContent = 'Saving...';
        }

        app.api('/access-control/entity/' + encodeURIComponent(entityType) + '/' + encodeURIComponent(entityId), {
            method: 'PUT',
            body: JSON.stringify({ rules: rules, sync_yaml: entityType === 'plugin' })
        }).then(function() {
            if (app.Toast) app.Toast.show('Access rules updated', 'success');
            closeSidePanel();
            window.location.reload();
        }).catch(function(err) {
            if (app.Toast) app.Toast.show(err.message || 'Failed to save rules', 'error');
            if (saveBtn) {
                saveBtn.disabled = false;
                saveBtn.textContent = 'Save Changes';
            }
        });
    }

    function initBulkPanel() {
        const openBtn = document.getElementById('acl-bulk-assign');
        const closeBtn = document.getElementById('acl-bulk-close');
        const cancelBtn = document.getElementById('acl-bulk-cancel');
        const overlay = document.getElementById('acl-bulk-overlay');
        const applyBtn = document.getElementById('acl-bulk-apply');

        if (openBtn) openBtn.addEventListener('click', openBulkPanel);
        if (closeBtn) closeBtn.addEventListener('click', closeBulkPanel);
        if (cancelBtn) cancelBtn.addEventListener('click', closeBulkPanel);
        if (overlay) overlay.addEventListener('click', closeBulkPanel);
        if (applyBtn) applyBtn.addEventListener('click', applyBulk);
    }

    function openBulkPanel() {
        const count = Object.keys(selectedEntities).length;
        if (count === 0) return;

        const firstKey = Object.keys(selectedEntities)[0];
        const parts = firstKey.split(':');
        const data = getEntityData(parts[0], parts[1]);

        const body = document.getElementById('acl-bulk-body');
        if (!body) return;

        let html = '<p style="margin-bottom:var(--space-4);color:var(--text-secondary);font-size:var(--text-sm)">Applying to <strong>' + count + '</strong> selected entities. This will replace existing rules.</p>';

        html += '<div class="acl-panel-section">';
        html += '<h3 class="acl-panel-section-title">Roles</h3>';
        if (data && data.roles) {
            data.roles.forEach(function(role) {
                html += '<label class="acl-checkbox-row">' +
                    '<input type="checkbox" name="role" value="' + escapeHtml(role.name) + '">' +
                    '<span class="acl-checkbox-label">' + escapeHtml(role.name) + '</span>' +
                    '</label>';
            });
        }
        html += '</div>';

        html += '<div class="acl-panel-section">';
        html += '<h3 class="acl-panel-section-title">Departments</h3>';
        if (data && data.departments) {
            data.departments.forEach(function(dept) {
                html += '<div class="acl-dept-row">' +
                    '<label class="acl-checkbox-row" style="flex:1">' +
                    '<input type="checkbox" name="department" value="' + escapeHtml(dept.name) + '">' +
                    '<span class="acl-checkbox-label">' + escapeHtml(dept.name) +
                    ' <span class="acl-dept-count">(' + dept.user_count + ' members)</span></span>' +
                    '</label>' +
                    '<label class="acl-default-toggle disabled">' +
                    '<input type="checkbox" name="default_included" value="' + escapeHtml(dept.name) + '" disabled>' +
                    '<span class="acl-toggle-label">Default</span>' +
                    '</label>' +
                    '</div>';
            });
        }
        html += '</div>';

        body.innerHTML = html;

        body.querySelectorAll('input[name="department"]').forEach(function(cb) {
            cb.addEventListener('change', function() {
                const row = cb.closest('.acl-dept-row');
                if (!row) return;
                const toggle = row.querySelector('.acl-default-toggle');
                const defaultCb = row.querySelector('input[name="default_included"]');
                if (toggle) toggle.classList.toggle('disabled', !cb.checked);
                if (defaultCb) defaultCb.disabled = !cb.checked;
                if (!cb.checked && defaultCb) defaultCb.checked = false;
            });
        });

        document.getElementById('acl-bulk-overlay').classList.add('active');
        document.getElementById('acl-bulk-panel').classList.add('open');
    }

    function closeBulkPanel() {
        const overlay = document.getElementById('acl-bulk-overlay');
        const panel = document.getElementById('acl-bulk-panel');
        if (overlay) overlay.classList.remove('active');
        if (panel) panel.classList.remove('open');
    }

    function applyBulk() {
        const body = document.getElementById('acl-bulk-body');
        if (!body) return;

        const entities = [];
        Object.keys(selectedEntities).forEach(function(key) {
            const parts = key.split(':');
            entities.push({ entity_type: parts[0], entity_id: parts[1] });
        });

        const rules = [];
        body.querySelectorAll('input[name="role"]:checked').forEach(function(cb) {
            rules.push({ rule_type: 'role', rule_value: cb.value, access: 'allow', default_included: false });
        });
        body.querySelectorAll('input[name="department"]:checked').forEach(function(cb) {
            const deptName = cb.value;
            const defaultCb = body.querySelector('input[name="default_included"][value="' + deptName + '"]');
            rules.push({
                rule_type: 'department',
                rule_value: deptName,
                access: 'allow',
                default_included: defaultCb ? defaultCb.checked : false
            });
        });

        const hasPlugins = entities.some(function(e) { return e.entity_type === 'plugin'; });

        const applyBtn = document.getElementById('acl-bulk-apply');
        if (applyBtn) {
            applyBtn.disabled = true;
            applyBtn.textContent = 'Applying...';
        }

        app.api('/access-control/bulk', {
            method: 'PUT',
            body: JSON.stringify({ entities: entities, rules: rules, sync_yaml: hasPlugins })
        }).then(function() {
            if (app.Toast) app.Toast.show('Bulk assign complete', 'success');
            closeBulkPanel();
            window.location.reload();
        }).catch(function(err) {
            if (app.Toast) app.Toast.show(err.message || 'Bulk assign failed', 'error');
            if (applyBtn) {
                applyBtn.disabled = false;
                applyBtn.textContent = 'Apply to Selected';
            }
        });
    }

    function updateCoverage() {
        const panel = document.querySelector('[data-acl-panel="' + activeTab + '"]');
        if (!panel) return;
        const rows = panel.querySelectorAll('.acl-entity-row');
        const total = rows.length;
        let covered = 0;
        rows.forEach(function(r) {
            const indicator = r.querySelector('.acl-coverage-indicator');
            if (indicator) {
                const parts = indicator.textContent.trim().split('/');
                if (parts[0] && parseInt(parts[0], 10) > 0) covered++;
            }
        });
        const el = document.getElementById('acl-coverage-text');
        if (el) {
            const label = activeTab === 'mcp' ? 'MCP servers' : activeTab;
            el.textContent = covered + ' of ' + total + ' ' + label + ' have department assignments';
        }
    }

    function getEntityData(entityType, entityId) {
        const el = document.querySelector('[data-acl-entity="' + entityType + '-' + entityId + '"]');
        if (!el) return null;
        try {
            return JSON.parse(el.textContent);
        } catch (e) {
            return null;
        }
    }

    function escapeHtml(str) {
        if (!str) return '';
        return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
    }

    function debounce(fn, ms) {
        let timer;
        return function() {
            clearTimeout(timer);
            const args = arguments;
            const ctx = this;
            timer = setTimeout(function() { fn.apply(ctx, args); }, ms);
        };
    }

})(window.AdminApp);

(function(app) {
    'use strict';

    app.initOrgMarketplaces = function() {
        const searchInput = document.getElementById('mkt-search');
        const deptFilter = document.getElementById('mkt-dept-filter');
        const table = document.getElementById('mkt-table');

        if (window.AdminApp.OrgCommon) {
            AdminApp.OrgCommon.initExpandRows('#mkt-table');
        }

        // Build department lookup from embedded JSON data
        const mktDepts = {};
        document.querySelectorAll('script[data-marketplace-detail]').forEach(function(el) {
            try {
                const data = JSON.parse(el.textContent);
                const id = el.getAttribute('data-marketplace-detail');
                mktDepts[id] = (data.departments || [])
                    .filter(function(d) { return d.assigned; })
                    .map(function(d) { return d.name; });
            } catch (e) { /* skip */ }
        });

        function filterRows() {
            if (!table) return;
            const query = (searchInput ? searchInput.value : '').toLowerCase();
            const dept = deptFilter ? deptFilter.value : '';
            const rows = table.querySelectorAll('tbody tr.clickable-row');
            rows.forEach(function(row) {
                const name = row.getAttribute('data-name') || '';
                const entityId = row.getAttribute('data-entity-id') || '';
                const matchName = !query || name.indexOf(query) !== -1;
                const matchDept = !dept || (mktDepts[entityId] && mktDepts[entityId].indexOf(dept) !== -1);
                const visible = matchName && matchDept;
                row.style.display = visible ? '' : 'none';
                const detailRow = table.querySelector('tr[data-detail-for="' + entityId + '"]');
                if (detailRow && !visible) {
                    detailRow.classList.remove('visible');
                    detailRow.style.display = 'none';
                }
            });
        }

        if (searchInput) {
            let debounceTimer;
            searchInput.addEventListener('input', function() {
                clearTimeout(debounceTimer);
                debounceTimer = setTimeout(filterRows, 200);
            });
        }
        if (deptFilter) {
            deptFilter.addEventListener('change', filterRows);
        }

        app.events.on('click', '[data-toggle-json]', function(e, jsonBtn) {
            const id = jsonBtn.getAttribute('data-toggle-json');
            const container = document.querySelector('[data-json-container="' + id + '"]');
            if (container) {
                if (container.style.display === 'none') {
                    const dataEl = document.querySelector('script[data-marketplace-detail="' + id + '"]');
                    if (dataEl) {
                        try {
                            const data = JSON.parse(dataEl.textContent);
                            container.innerHTML = AdminApp.OrgCommon ? AdminApp.OrgCommon.formatJson(data) : '<pre>' + app.escapeHtml(JSON.stringify(data, null, 2)) + '</pre>';
                        } catch (err) {
                            container.innerHTML = '<p>Error parsing JSON</p>';
                        }
                    }
                    container.style.display = 'block';
                    jsonBtn.textContent = 'Hide JSON';
                } else {
                    container.style.display = 'none';
                    jsonBtn.textContent = 'Show JSON';
                }
            }
        });

        app.events.on('click', '.actions-trigger', function(e, trigger) {
            e.stopPropagation();
            const menu = trigger.closest('.actions-menu');
            const dd = menu ? menu.querySelector('.actions-dropdown') : null;
            if (dd) {
                const isOpen = dd.classList.contains('open');
                app.shared.closeAllMenus();
                if (!isOpen) dd.classList.add('open');
            }
        });

        app.events.on('click', '[data-delete-marketplace]', function(e, deleteBtn) {
            const id = deleteBtn.getAttribute('data-delete-marketplace');
            showDeleteConfirm(id);
        });

        app.events.on('click', '[data-copy-install-link]', function(e, btn) {
            var id = btn.getAttribute('data-copy-install-link');
            var siteUrl = window.location.origin;
            var installUrl = siteUrl + '/api/public/marketplace/org/' + encodeURIComponent(id) + '.git';
            navigator.clipboard.writeText(installUrl).then(function() {
                app.Toast.show('Install link copied to clipboard', 'success');
            }).catch(function() {
                var textarea = document.createElement('textarea');
                textarea.value = installUrl;
                document.body.appendChild(textarea);
                textarea.select();
                document.execCommand('copy');
                document.body.removeChild(textarea);
                app.Toast.show('Install link copied to clipboard', 'success');
            });
            app.shared.closeAllMenus();
        });

        app.events.on('click', '[data-sync-marketplace]', function(e, btn) {
            var id = btn.getAttribute('data-sync-marketplace');
            btn.disabled = true;
            var origText = btn.textContent;
            btn.textContent = 'Syncing...';
            app.shared.closeAllMenus();
            fetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id) + '/sync', {
                method: 'POST',
                credentials: 'include'
            })
            .then(function(resp) { return resp.json().then(function(data) { return { ok: resp.ok, data: data }; }); })
            .then(function(result) {
                if (result.ok) {
                    var msg = 'Sync completed: ' + (result.data.plugins_synced || 0) + ' plugins';
                    if (!result.data.changed) msg = 'Already up to date';
                    app.Toast.show(msg, 'success');
                    if (result.data.changed) setTimeout(function() { window.location.reload(); }, 1000);
                } else {
                    app.Toast.show(result.data.error || 'Sync failed', 'error');
                }
                btn.disabled = false;
                btn.textContent = origText;
            })
            .catch(function() {
                app.Toast.show('Network error during sync', 'error');
                btn.disabled = false;
                btn.textContent = origText;
            });
        });

        app.events.on('click', '[data-publish-marketplace]', function(e, btn) {
            var id = btn.getAttribute('data-publish-marketplace');
            app.shared.closeAllMenus();
            var overlay = document.createElement('div');
            overlay.className = 'confirm-overlay';
            overlay.innerHTML = '<div class="confirm-dialog">' +
                '<h3 style="margin:0 0 var(--space-3)">Publish to GitHub?</h3>' +
                '<p style="margin:0 0 var(--space-4);color:var(--text-secondary);font-size:var(--text-sm)">This will push the current marketplace plugins to the linked GitHub repository. Any remote changes will be overwritten.</p>' +
                '<div style="display:flex;gap:var(--space-3);justify-content:flex-end">' +
                    '<button class="btn btn-secondary" data-confirm-cancel>Cancel</button>' +
                    '<button class="btn btn-primary" data-confirm-publish>Publish</button>' +
                '</div>' +
            '</div>';
            document.body.appendChild(overlay);
            overlay.addEventListener('click', function(ev) {
                if (ev.target === overlay || ev.target.closest('[data-confirm-cancel]')) {
                    overlay.remove();
                    return;
                }
                var pubBtn = ev.target.closest('[data-confirm-publish]');
                if (pubBtn) {
                    pubBtn.disabled = true;
                    pubBtn.textContent = 'Publishing...';
                    fetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id) + '/publish', {
                        method: 'POST',
                        credentials: 'include'
                    })
                    .then(function(resp) { return resp.json().then(function(data) { return { ok: resp.ok, data: data }; }); })
                    .then(function(result) {
                        overlay.remove();
                        if (result.ok) {
                            var msg = 'Published: ' + (result.data.plugins_synced || 0) + ' plugins';
                            if (!result.data.changed) msg = 'No changes to publish';
                            app.Toast.show(msg, 'success');
                            if (result.data.changed) setTimeout(function() { window.location.reload(); }, 1000);
                        } else {
                            app.Toast.show(result.data.error || 'Publish failed', 'error');
                        }
                    })
                    .catch(function() {
                        overlay.remove();
                        app.Toast.show('Network error during publish', 'error');
                    });
                }
            });
        });

        initManagePluginsPanel();
        initEditPanel();
    };

    function showDeleteConfirm(marketplaceId) {
        const overlay = document.createElement('div');
        overlay.className = 'confirm-overlay';
        overlay.innerHTML = '<div class="confirm-dialog">' +
            '<h3 style="margin:0 0 var(--space-3)">Delete Marketplace?</h3>' +
            '<p style="margin:0 0 var(--space-2);color:var(--text-secondary);font-size:var(--text-sm)">You are about to delete <strong>' + app.escapeHtml(marketplaceId) + '</strong>.</p>' +
            '<p style="margin:0 0 var(--space-5);color:var(--text-secondary);font-size:var(--text-sm)">This will remove the marketplace and all plugin associations. This action cannot be undone.</p>' +
            '<div style="display:flex;gap:var(--space-3);justify-content:flex-end">' +
                '<button class="btn btn-secondary" data-confirm-cancel>Cancel</button>' +
                '<button class="btn btn-danger" data-confirm-delete="' + app.escapeHtml(marketplaceId) + '">Delete Marketplace</button>' +
            '</div>' +
        '</div>';
        document.body.appendChild(overlay);

        overlay.addEventListener('click', async function(e) {
            if (e.target === overlay || e.target.closest('[data-confirm-cancel]')) {
                overlay.remove();
                return;
            }
            const confirmBtn = e.target.closest('[data-confirm-delete]');
            if (confirmBtn) {
                const id = confirmBtn.getAttribute('data-confirm-delete');
                confirmBtn.disabled = true;
                confirmBtn.textContent = 'Deleting...';
                try {
                    const resp = await fetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id), {
                        method: 'DELETE',
                        credentials: 'include'
                    });
                    if (resp.ok) {
                        app.Toast.show('Marketplace deleted', 'success');
                        setTimeout(function() { window.location.reload(); }, 500);
                    } else {
                        const data = await resp.json().catch(function() { return {}; });
                        app.Toast.show(data.error || 'Failed to delete', 'error');
                    }
                } catch (err) {
                    app.Toast.show('Network error', 'error');
                }
                overlay.remove();
            }
        });
    }

    function initManagePluginsPanel() {
        if (!window.AdminApp.OrgCommon) return;
        const panelApi = AdminApp.OrgCommon.initSidePanel('mkt-panel');
        if (!panelApi) return;

        app.events.on('click', '[data-manage-plugins]', function(e, btn) {
            const id = btn.getAttribute('data-manage-plugins');

            const dataEl = document.querySelector('script[data-marketplace-detail="' + id + '"]');
            if (!dataEl) return;
            let mktData;
            try { mktData = JSON.parse(dataEl.textContent); } catch (err) { return; }

            panelApi.setTitle('Manage Plugins - ' + (mktData.name || id));

            fetch(app.API_BASE + '/plugins', { credentials: 'include' })
                .then(function(r) { return r.json(); })
                .then(function(allPlugins) {
                    const currentIds = {};
                    (mktData.plugin_ids || []).forEach(function(pid) { currentIds[pid] = true; });

                    let html = '<div class="assign-panel-checklist">';
                    if (!allPlugins.length) {
                        html += '<p style="color:var(--text-tertiary);font-size:var(--text-sm)">No plugins available.</p>';
                    } else {
                        allPlugins.forEach(function(p) {
                            const pid = p.id || p.plugin_id;
                            const pname = p.name || pid;
                            const checked = currentIds[pid] ? ' checked' : '';
                            html += '<label class="acl-checkbox-row" style="display:flex;align-items:center;gap:var(--space-2);padding:var(--space-1) 0;cursor:pointer">' +
                                '<input type="checkbox" name="plugin_id" value="' + app.escapeHtml(pid) + '"' + checked + '>' +
                                '<span>' + app.escapeHtml(pname) + '</span>' +
                                '</label>';
                        });
                    }
                    html += '</div>';
                    panelApi.setBody(html);
                    panelApi.setFooter(
                        '<button class="btn btn-secondary" data-panel-close>Cancel</button> ' +
                        '<button class="btn btn-primary" id="mkt-save-plugins">Save</button>'
                    );

                    const footer = panelApi.panel.querySelector('[data-panel-footer]');
                    if (footer) {
                        const cancelBtn = footer.querySelector('[data-panel-close]');
                        if (cancelBtn) cancelBtn.addEventListener('click', panelApi.close);
                    }

                    const saveBtn = document.getElementById('mkt-save-plugins');
                    if (saveBtn) {
                        saveBtn.addEventListener('click', async function() {
                            const checked = panelApi.panel.querySelectorAll('input[name="plugin_id"]:checked');
                            const ids = [];
                            checked.forEach(function(cb) { ids.push(cb.value); });
                            saveBtn.disabled = true;
                            saveBtn.textContent = 'Saving...';
                            try {
                                const resp = await fetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id) + '/plugins', {
                                    method: 'PUT',
                                    credentials: 'include',
                                    headers: { 'Content-Type': 'application/json' },
                                    body: JSON.stringify({ plugin_ids: ids })
                                });
                                if (resp.ok) {
                                    app.Toast.show('Plugins updated', 'success');
                                    panelApi.close();
                                    setTimeout(function() { window.location.reload(); }, 500);
                                } else {
                                    const data = await resp.json().catch(function() { return {}; });
                                    app.Toast.show(data.error || 'Failed to update', 'error');
                                    saveBtn.disabled = false;
                                    saveBtn.textContent = 'Save';
                                }
                            } catch (err) {
                                app.Toast.show('Network error', 'error');
                                saveBtn.disabled = false;
                                saveBtn.textContent = 'Save';
                            }
                        });
                    }

                    panelApi.open();
                })
                .catch(function() {
                    app.Toast.show('Failed to load plugins', 'error');
                });
        });
    }

    function initEditPanel() {
        if (!window.AdminApp.OrgCommon) return;
        const panelApi = AdminApp.OrgCommon.initSidePanel('mkt-edit-panel');
        if (!panelApi) return;

        function readJsonEl(id) {
            const el = document.getElementById(id);
            if (!el) return [];
            try { return JSON.parse(el.textContent); } catch (e) { return []; }
        }

        const allRoles = readJsonEl('mkt-all-roles');
        const allDepts = readJsonEl('mkt-all-departments');

        function openEdit(marketplaceId) {
            const isEdit = !!marketplaceId;
            let mktData = {};

            if (isEdit) {
                const dataEl = document.querySelector('script[data-marketplace-detail="' + marketplaceId + '"]');
                if (dataEl) {
                    try { mktData = JSON.parse(dataEl.textContent); } catch (e) { /* skip */ }
                }
            }

            panelApi.setTitle(isEdit ? 'Edit Marketplace' : 'Create Marketplace');

            fetch(app.API_BASE + '/plugins', { credentials: 'include' })
                .then(function(r) { return r.json(); })
                .then(function(allPlugins) {
                    const currentPluginIds = {};
                    (mktData.plugin_ids || []).forEach(function(pid) { currentPluginIds[pid] = true; });

                    const currentRoles = {};
                    (mktData.roles || []).forEach(function(r) {
                        if (r.assigned) currentRoles[r.name] = true;
                    });

                    const currentDepts = {};
                    const deptDefaults = {};
                    (mktData.departments || []).forEach(function(d) {
                        if (d.assigned) {
                            currentDepts[d.name] = true;
                            deptDefaults[d.name] = d.default_included;
                        }
                    });

                    let html = '<form id="panel-edit-form">';

                    if (!isEdit) {
                        html += '<div class="form-group">' +
                            '<label class="field-label">Marketplace ID</label>' +
                            '<input type="text" class="field-input" name="marketplace_id" required placeholder="e.g. my-marketplace">' +
                            '</div>';
                    }

                    html += '<div class="form-group">' +
                        '<label class="field-label">Name</label>' +
                        '<input type="text" class="field-input" name="name" required value="' + app.escapeHtml(mktData.name || '') + '">' +
                        '</div>';

                    html += '<div class="form-group">' +
                        '<label class="field-label">Description</label>' +
                        '<textarea class="field-input" name="description" rows="3">' + app.escapeHtml(mktData.description || '') + '</textarea>' +
                        '</div>';

                    html += '<div class="form-group">' +
                        '<label class="field-label">GitHub Repository URL</label>' +
                        '<input type="url" class="field-input" name="github_repo_url" placeholder="https://github.com/org/repo" value="' + app.escapeHtml(mktData.github_repo_url || '') + '">' +
                        '<span class="field-hint">Link to a GitHub repository to enable sync/publish</span>' +
                        '</div>';

                    // Roles
                    html += '<div class="form-group">' +
                        '<label class="field-label">Roles</label>' +
                        '<div style="display:flex;flex-wrap:wrap;gap:var(--space-1);padding:var(--space-2) 0">';
                    allRoles.forEach(function(r) {
                        const val = r.value || r;
                        const checked = currentRoles[val] ? ' checked' : '';
                        html += '<label style="display:inline-flex;align-items:center;gap:var(--space-2);margin-right:var(--space-3);font-size:var(--text-sm);cursor:pointer">' +
                            '<input type="checkbox" name="roles" value="' + app.escapeHtml(val) + '"' + checked + '> ' +
                            app.escapeHtml(val) + '</label>';
                    });
                    html += '</div></div>';

                    // Departments
                    html += '<div class="form-group">' +
                        '<label class="field-label">Departments</label>' +
                        '<div class="checklist-container" style="max-height:300px;overflow-y:auto;border:1px solid var(--border-subtle);border-radius:var(--radius-md);padding:var(--space-2)">' +
                        '<div class="checklist-item" style="display:flex;align-items:center;gap:var(--space-2);padding:var(--space-2);border-bottom:1px solid var(--border-subtle)">' +
                        '<input type="checkbox" id="panel-dept-check-all">' +
                        '<label for="panel-dept-check-all" style="flex:1;font-size:var(--text-sm);cursor:pointer;color:var(--text-primary);font-weight:600">Check all</label>' +
                        '</div>';
                    allDepts.forEach(function(d, i) {
                        const val = d.value || d.name || d;
                        const checked = currentDepts[val] ? ' checked' : '';
                        const defaultChecked = deptDefaults[val] ? ' checked' : '';
                        html += '<div class="checklist-item" style="display:flex;align-items:center;gap:var(--space-2);padding:var(--space-2)">' +
                            '<input type="checkbox" name="departments" value="' + app.escapeHtml(val) + '"' + checked + ' id="panel-dept-' + i + '">' +
                            '<label for="panel-dept-' + i + '" style="flex:1;font-size:var(--text-sm);cursor:pointer;color:var(--text-primary)">' + app.escapeHtml(val) + '</label>' +
                            '<span class="badge badge-gray" style="font-size:var(--text-xs)">' + (d.user_count || 0) + ' users</span>' +
                            '<label style="display:inline-flex;align-items:center;gap:4px;font-size:var(--text-xs);color:var(--text-secondary);cursor:pointer;white-space:nowrap">' +
                            '<input type="checkbox" name="dept_default_' + val + '"' + defaultChecked + '> Default</label>' +
                            '</div>';
                    });
                    html += '</div>' +
                        '<span class="field-hint" style="margin-top:var(--space-2);display:block">At least one department is required.</span>' +
                        '</div>';

                    // Plugins
                    html += '<div class="form-group">' +
                        '<label class="field-label">Plugins</label>' +
                        '<input type="text" class="field-input" placeholder="Filter plugins..." id="panel-plugin-filter" style="margin-bottom:var(--space-2)">' +
                        '<div class="checklist-container" style="max-height:200px;overflow-y:auto;border:1px solid var(--border-subtle);border-radius:var(--radius-md);padding:var(--space-2)">';
                    allPlugins.forEach(function(p, i) {
                        const pid = p.id || p.plugin_id;
                        const pname = p.name || pid;
                        const checked = currentPluginIds[pid] ? ' checked' : '';
                        html += '<div class="checklist-item" data-item-name="' + app.escapeHtml((pname).toLowerCase()) + '">' +
                            '<input type="checkbox" name="plugin_ids" value="' + app.escapeHtml(pid) + '"' + checked + ' id="panel-plugin-' + i + '">' +
                            '<label for="panel-plugin-' + i + '">' + app.escapeHtml(pname) + '</label>' +
                            '</div>';
                    });
                    html += '</div></div>';

                    html += '</form>';

                    panelApi.setBody(html);

                    let footerHtml = '<button class="btn btn-secondary" data-panel-close>Cancel</button> ' +
                        '<button class="btn btn-primary" id="mkt-edit-save">' + (isEdit ? 'Save Changes' : 'Create Marketplace') + '</button>';
                    if (isEdit) {
                        footerHtml = '<button class="btn btn-danger" id="mkt-edit-delete" style="margin-right:auto">Delete</button> ' + footerHtml;
                    }
                    panelApi.setFooter(footerHtml);

                    // Wire up cancel
                    const footer = panelApi.panel.querySelector('[data-panel-footer]');
                    if (footer) {
                        const cancelBtn = footer.querySelector('[data-panel-close]');
                        if (cancelBtn) cancelBtn.addEventListener('click', panelApi.close);
                    }

                    // Wire up check-all
                    const checkAll = document.getElementById('panel-dept-check-all');
                    if (checkAll) {
                        checkAll.addEventListener('change', function() {
                            const boxes = panelApi.panel.querySelectorAll('input[name="departments"]');
                            boxes.forEach(function(cb) { cb.checked = checkAll.checked; });
                        });
                        const boxes = panelApi.panel.querySelectorAll('input[name="departments"]');
                        let allChecked = boxes.length > 0;
                        boxes.forEach(function(cb) { if (!cb.checked) allChecked = false; });
                        checkAll.checked = allChecked;
                        panelApi.panel.addEventListener('change', function(e) {
                            if (e.target.name === 'departments') {
                                const boxes = panelApi.panel.querySelectorAll('input[name="departments"]');
                                let all = boxes.length > 0;
                                boxes.forEach(function(cb) { if (!cb.checked) all = false; });
                                checkAll.checked = all;
                            }
                        });
                    }

                    // Wire up plugin filter
                    const pluginFilter = document.getElementById('panel-plugin-filter');
                    if (pluginFilter) {
                        pluginFilter.addEventListener('input', function() {
                            const q = pluginFilter.value.toLowerCase();
                            panelApi.panel.querySelectorAll('.checklist-item[data-item-name]').forEach(function(item) {
                                const name = item.getAttribute('data-item-name') || '';
                                item.style.display = (!q || name.indexOf(q) !== -1) ? '' : 'none';
                            });
                        });
                    }

                    // Wire up save
                    const saveBtn = document.getElementById('mkt-edit-save');
                    if (saveBtn) {
                        saveBtn.addEventListener('click', function() {
                            handlePanelSave(isEdit, marketplaceId, saveBtn, panelApi);
                        });
                    }

                    // Wire up delete
                    const deleteBtn = document.getElementById('mkt-edit-delete');
                    if (deleteBtn) {
                        deleteBtn.addEventListener('click', function() {
                            panelApi.close();
                            showDeleteConfirm(marketplaceId);
                        });
                    }

                    panelApi.open();
                })
                .catch(function() {
                    app.Toast.show('Failed to load plugins', 'error');
                });
        }

        app.events.on('click', '[data-edit-marketplace]', function(e, btn) {
            openEdit(btn.getAttribute('data-edit-marketplace'));
        });

        app.events.on('click', '[data-create-marketplace]', function(e, btn) {
            e.preventDefault();
            openEdit(null);
        });
    }

    async function handlePanelSave(isEdit, marketplaceId, saveBtn, panelApi) {
        const form = document.getElementById('panel-edit-form');
        if (!form) return;

        const deptChecked = form.querySelectorAll('input[name="departments"]:checked');
        if (deptChecked.length === 0) {
            app.Toast.show('Please select at least one department', 'error');
            return;
        }

        saveBtn.disabled = true;
        saveBtn.textContent = 'Saving...';

        const pluginIds = [];
        form.querySelectorAll('input[name="plugin_ids"]:checked').forEach(function(cb) { pluginIds.push(cb.value); });

        const selectedRoles = [];
        form.querySelectorAll('input[name="roles"]:checked').forEach(function(cb) { selectedRoles.push(cb.value); });

        const deptRules = [];
        form.querySelectorAll('input[name="departments"]').forEach(function(cb) {
            if (cb.checked) {
                const defaultToggle = form.querySelector('input[name="dept_default_' + cb.value + '"]');
                deptRules.push({
                    rule_type: 'department',
                    rule_value: cb.value,
                    access: 'allow',
                    default_included: defaultToggle ? defaultToggle.checked : false
                });
            }
        });

        let aclRules = [];
        selectedRoles.forEach(function(role) {
            aclRules.push({ rule_type: 'role', rule_value: role, access: 'allow', default_included: false });
        });
        aclRules = aclRules.concat(deptRules);

        var githubUrlInput = form.querySelector('input[name="github_repo_url"]');
        var githubUrl = githubUrlInput ? githubUrlInput.value.trim() : '';

        try {
            if (isEdit) {
                const body = {
                    name: form.querySelector('input[name="name"]').value,
                    description: form.querySelector('textarea[name="description"]').value,
                    github_repo_url: githubUrl || null,
                    plugin_ids: pluginIds
                };
                const resp = await fetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(marketplaceId), {
                    method: 'PUT',
                    credentials: 'include',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(body)
                });
                if (resp.ok) {
                    await fetch(app.API_BASE + '/access-control/entity/marketplace/' + encodeURIComponent(marketplaceId), {
                        method: 'PUT',
                        credentials: 'include',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ rules: aclRules, sync_yaml: false })
                    });
                    app.Toast.show('Marketplace updated', 'success');
                    panelApi.close();
                    setTimeout(function() { window.location.reload(); }, 500);
                } else {
                    const data = await resp.json().catch(function() { return {}; });
                    app.Toast.show(data.error || 'Failed to update', 'error');
                }
            } else {
                const body = {
                    id: form.querySelector('input[name="marketplace_id"]').value,
                    name: form.querySelector('input[name="name"]').value,
                    description: form.querySelector('textarea[name="description"]').value,
                    github_repo_url: githubUrl || null,
                    plugin_ids: pluginIds
                };
                const resp = await fetch(app.API_BASE + '/org/marketplaces', {
                    method: 'POST',
                    credentials: 'include',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(body)
                });
                if (resp.ok || resp.status === 201) {
                    const created = await resp.json().catch(function() { return {}; });
                    const createdId = created.id || body.id;
                    if (aclRules.length > 0 && createdId) {
                        await fetch(app.API_BASE + '/access-control/entity/marketplace/' + encodeURIComponent(createdId), {
                            method: 'PUT',
                            credentials: 'include',
                            headers: { 'Content-Type': 'application/json' },
                            body: JSON.stringify({ rules: aclRules, sync_yaml: false })
                        });
                    }
                    app.Toast.show('Marketplace created', 'success');
                    panelApi.close();
                    setTimeout(function() { window.location.reload(); }, 500);
                } else {
                    const data = await resp.json().catch(function() { return {}; });
                    app.Toast.show(data.error || 'Failed to create', 'error');
                }
            }
        } catch (err) {
            app.Toast.show('Network error', 'error');
        }

        saveBtn.disabled = false;
        saveBtn.textContent = isEdit ? 'Save Changes' : 'Create Marketplace';
    }

    app.initMarketplaceEditForm = function() {
        const form = document.getElementById('marketplace-edit-form');
        if (!form) return;

        const isEdit = !!form.querySelector('input[name="marketplace_id"][readonly]');

        form.addEventListener('submit', async function(e) {
            e.preventDefault();
            const submitBtn = form.querySelector('button[type="submit"]');
            if (submitBtn) {
                submitBtn.disabled = true;
                submitBtn.textContent = 'Saving...';
            }

            const deptChecked = form.querySelectorAll('input[name="departments"]:checked');
            if (deptChecked.length === 0) {
                app.Toast.show('Please select at least one department', 'error');
                if (submitBtn) {
                    submitBtn.disabled = false;
                    submitBtn.textContent = isEdit ? 'Save Changes' : 'Create Marketplace';
                }
                return;
            }

            const pluginCheckboxes = form.querySelectorAll('input[name="plugin_ids"]:checked');
            const pluginIds = [];
            pluginCheckboxes.forEach(function(cb) { pluginIds.push(cb.value); });

            const roleCheckboxes = form.querySelectorAll('input[name="roles"]:checked');
            const selectedRoles = [];
            roleCheckboxes.forEach(function(cb) { selectedRoles.push(cb.value); });

            const deptCheckboxes = form.querySelectorAll('input[name="departments"]');
            const deptRules = [];
            deptCheckboxes.forEach(function(cb) {
                if (cb.checked) {
                    const defaultToggle = form.querySelector('input[name="dept_default_' + cb.value + '"]');
                    deptRules.push({
                        rule_type: 'department',
                        rule_value: cb.value,
                        access: 'allow',
                        default_included: defaultToggle ? defaultToggle.checked : false
                    });
                }
            });

            let aclRules = [];
            selectedRoles.forEach(function(role) {
                aclRules.push({ rule_type: 'role', rule_value: role, access: 'allow', default_included: false });
            });
            aclRules = aclRules.concat(deptRules);

            var formGithubInput = form.querySelector('input[name="github_repo_url"]');
            var formGithubUrl = formGithubInput ? formGithubInput.value.trim() : '';

            if (isEdit) {
                const id = form.querySelector('input[name="marketplace_id"]').value;
                const body = {
                    name: form.querySelector('input[name="name"]').value,
                    description: form.querySelector('textarea[name="description"]').value,
                    github_repo_url: formGithubUrl || null,
                    plugin_ids: pluginIds
                };

                try {
                    const resp = await fetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id), {
                        method: 'PUT',
                        credentials: 'include',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(body)
                    });
                    if (resp.ok) {
                        await fetch(app.API_BASE + '/access-control/entity/marketplace/' + encodeURIComponent(id), {
                            method: 'PUT',
                            credentials: 'include',
                            headers: { 'Content-Type': 'application/json' },
                            body: JSON.stringify({ rules: aclRules, sync_yaml: false })
                        });
                        app.Toast.show('Marketplace updated', 'success');
                        setTimeout(function() { window.location.href = '/admin/org/marketplaces/'; }, 500);
                    } else {
                        const data = await resp.json().catch(function() { return {}; });
                        app.Toast.show(data.error || 'Failed to update', 'error');
                    }
                } catch (err) {
                    app.Toast.show('Network error', 'error');
                }
            } else {
                const body = {
                    id: form.querySelector('input[name="marketplace_id"]').value,
                    name: form.querySelector('input[name="name"]').value,
                    description: form.querySelector('textarea[name="description"]').value,
                    github_repo_url: formGithubUrl || null,
                    plugin_ids: pluginIds
                };

                try {
                    const resp = await fetch(app.API_BASE + '/org/marketplaces', {
                        method: 'POST',
                        credentials: 'include',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(body)
                    });
                    if (resp.ok || resp.status === 201) {
                        const created = await resp.json().catch(function() { return {}; });
                        const createdId = created.id || body.id;
                        if (aclRules.length > 0 && createdId) {
                            await fetch(app.API_BASE + '/access-control/entity/marketplace/' + encodeURIComponent(createdId), {
                                method: 'PUT',
                                credentials: 'include',
                                headers: { 'Content-Type': 'application/json' },
                                body: JSON.stringify({ rules: aclRules, sync_yaml: false })
                            });
                        }
                        app.Toast.show('Marketplace created', 'success');
                        setTimeout(function() { window.location.href = '/admin/org/marketplaces/'; }, 500);
                    } else {
                        const data = await resp.json().catch(function() { return {}; });
                        app.Toast.show(data.error || 'Failed to create', 'error');
                    }
                } catch (err) {
                    app.Toast.show('Network error', 'error');
                }
            }

            if (submitBtn) {
                submitBtn.disabled = false;
                submitBtn.textContent = isEdit ? 'Save Changes' : 'Create Marketplace';
            }
        });

        const deleteBtn = document.getElementById('btn-delete-marketplace');
        if (deleteBtn) {
            deleteBtn.addEventListener('click', function() {
                const idInput = form.querySelector('input[name="marketplace_id"]');
                if (idInput) showDeleteConfirm(idInput.value);
            });
        }

        const checkAllDept = form.querySelector('#dept-check-all');
        if (checkAllDept) {
            checkAllDept.addEventListener('change', function() {
                const boxes = form.querySelectorAll('input[name="departments"]');
                boxes.forEach(function(cb) { cb.checked = checkAllDept.checked; });
            });
            form.addEventListener('change', function(e) {
                if (e.target.name === 'departments') {
                    const boxes = form.querySelectorAll('input[name="departments"]');
                    let allChecked = boxes.length > 0;
                    boxes.forEach(function(cb) { if (!cb.checked) allChecked = false; });
                    checkAllDept.checked = allChecked;
                }
            });
            // Sync check-all initial state to match current department selections
            const boxes = form.querySelectorAll('input[name="departments"]');
            let allChecked = boxes.length > 0;
            boxes.forEach(function(cb) { if (!cb.checked) allChecked = false; });
            checkAllDept.checked = allChecked;
        }
    };

})(window.AdminApp = window.AdminApp || {});

(function(app) {
    'use strict';

    var MyCommon = {

        initExpandRows: function(tableSelector, renderCallback) {
            var table = document.querySelector(tableSelector);
            if (!table) return;

            table.addEventListener('click', function(e) {
                if (e.target.closest('[data-no-row-click]') ||
                    e.target.closest('.actions-menu') ||
                    e.target.closest('.btn') ||
                    e.target.closest('a') ||
                    e.target.closest('input') ||
                    e.target.closest('.toggle-switch')) {
                    return;
                }

                var row = e.target.closest('tr.clickable-row');
                if (!row) return;

                var detailRow = row.nextElementSibling;
                if (!detailRow || !detailRow.classList.contains('detail-row')) return;

                MyCommon.handleRowClick(row, detailRow);

                if (renderCallback && detailRow.classList.contains('visible')) {
                    renderCallback(row, detailRow);
                }
            });
        },

        handleRowClick: function(row, detailRow) {
            var isVisible = detailRow.classList.contains('visible');

            var table = row.closest('table');
            if (table) {
                table.querySelectorAll('tr.detail-row.visible').forEach(function(r) {
                    if (r !== detailRow) {
                        r.classList.remove('visible');
                        var prevRow = r.previousElementSibling;
                        if (prevRow) {
                            var indicator = prevRow.querySelector('.expand-indicator');
                            if (indicator) indicator.classList.remove('expanded');
                        }
                    }
                });
            }

            if (!isVisible) {
                detailRow.classList.add('visible');
                var expandIndicator = row.querySelector('.expand-indicator');
                if (expandIndicator) expandIndicator.classList.add('expanded');
            } else {
                detailRow.classList.remove('visible');
                var collapseIndicator = row.querySelector('.expand-indicator');
                if (collapseIndicator) collapseIndicator.classList.remove('expanded');
            }
        },

        initSidePanel: function(panelId) {
            var panel = document.getElementById(panelId);
            if (!panel) return null;

            var overlayId = panel.getAttribute('data-overlay') || (panelId + '-overlay');
            var overlay = document.getElementById(overlayId);
            var closeBtn = panel.querySelector('[data-panel-close]');

            var api = {
                open: function() {
                    panel.classList.add('open');
                    if (overlay) overlay.classList.add('active');
                },
                close: function() {
                    panel.classList.remove('open');
                    if (overlay) overlay.classList.remove('active');
                },
                setTitle: function(text) {
                    var title = panel.querySelector('[data-panel-title]');
                    if (title) title.textContent = text;
                },
                setBody: function(html) {
                    var body = panel.querySelector('[data-panel-body]');
                    if (body) body.innerHTML = html;
                },
                setFooter: function(html) {
                    var footer = panel.querySelector('[data-panel-footer]');
                    if (footer) footer.innerHTML = html;
                },
                panel: panel
            };

            if (closeBtn) closeBtn.addEventListener('click', api.close);
            if (overlay) overlay.addEventListener('click', api.close);

            return api;
        },

        initBulkActions: function(tableSelector, barId) {
            var table = document.querySelector(tableSelector);
            if (!table) return null;

            var selected = {};

            function updateCount() {
                var count = Object.keys(selected).length;
                var countEl = document.querySelector('[data-bulk-count]');
                if (countEl) countEl.textContent = count;
                var bar = document.getElementById(barId);
                if (bar) bar.style.display = count > 0 ? 'flex' : 'none';
            }

            table.addEventListener('change', function(e) {
                if (e.target.classList.contains('bulk-select-all')) {
                    var checked = e.target.checked;
                    table.querySelectorAll('.bulk-checkbox').forEach(function(cb) {
                        cb.checked = checked;
                        var id = cb.getAttribute('data-entity-id');
                        if (checked) {
                            selected[id] = true;
                        } else {
                            delete selected[id];
                        }
                    });
                    updateCount();
                    return;
                }

                if (e.target.classList.contains('bulk-checkbox')) {
                    var id = e.target.getAttribute('data-entity-id');
                    if (e.target.checked) {
                        selected[id] = true;
                    } else {
                        delete selected[id];
                    }
                    updateCount();
                }
            });

            return {
                getSelected: function() { return Object.keys(selected); },
                clear: function() {
                    selected = {};
                    table.querySelectorAll('.bulk-checkbox, .bulk-select-all').forEach(function(cb) {
                        cb.checked = false;
                    });
                    updateCount();
                }
            };
        },

        initSearch: function(inputId, tableSelector) {
            var input = document.getElementById(inputId);
            var table = document.querySelector(tableSelector);
            if (!input || !table) return;

            var timer = null;
            input.addEventListener('input', function() {
                clearTimeout(timer);
                timer = setTimeout(function() {
                    var query = input.value.toLowerCase().trim();
                    var rows = table.querySelectorAll('tbody tr.clickable-row');
                    rows.forEach(function(row) {
                        var text = row.textContent.toLowerCase();
                        var matches = !query || text.indexOf(query) !== -1;
                        row.style.display = matches ? '' : 'none';
                        var detail = row.nextElementSibling;
                        if (detail && detail.classList.contains('detail-row')) {
                            detail.style.display = matches ? '' : 'none';
                        }
                    });
                }, 200);
            });
        },

        initFilterSelect: function(selectId, tableSelector, dataAttr) {
            var select = document.getElementById(selectId);
            var table = document.querySelector(tableSelector);
            if (!select || !table) return;

            select.addEventListener('change', function() {
                var value = select.value;
                var rows = table.querySelectorAll('tbody tr.clickable-row');
                rows.forEach(function(row) {
                    var attrVal = row.getAttribute(dataAttr) || '';
                    var matches = !value || attrVal === value;
                    row.style.display = matches ? '' : 'none';
                    var detail = row.nextElementSibling;
                    if (detail && detail.classList.contains('detail-row')) {
                        detail.style.display = matches ? '' : 'none';
                    }
                });
            });
        },

        initForkPanel: function(config) {
            // config: { panelId, entityType, entityLabel, onForked }
            var panelApi = MyCommon.initSidePanel(config.panelId);
            if (!panelApi) return null;

            return {
                open: function() {
                    panelApi.setTitle('Fork from Org: ' + (config.entityLabel || config.entityType));
                    panelApi.setBody('<p style="color:var(--text-tertiary);text-align:center;padding:var(--space-4)">Loading...</p>');
                    panelApi.setFooter('');
                    panelApi.open();

                    fetch(app.API_BASE + '/user/forkable/' + config.entityType)
                        .then(function(res) { return res.json(); })
                        .then(function(data) {
                            var items = data[config.entityType] || data.plugins || data.skills || data.agents || data.mcp_servers || data.hooks || [];
                            if (items.length === 0) {
                                panelApi.setBody('<p style="color:var(--text-tertiary);text-align:center;padding:var(--space-4)">No org entities available to fork.</p>');
                                return;
                            }

                            var html = '<div class="add-checklist">';
                            items.forEach(function(item) {
                                var disabled = item.already_forked ? ' disabled' : '';
                                var label = item.already_forked ? ' (already forked)' : '';
                                html += '<label class="acl-checkbox-row">' +
                                    '<input type="checkbox" name="fork_id" value="' + app.escapeHtml(item.id) + '"' + disabled + '>' +
                                    '<span class="acl-checkbox-label">' + app.escapeHtml(item.name || item.id) + label + '</span>' +
                                    '</label>';
                            });
                            html += '</div>';
                            panelApi.setBody(html);

                            panelApi.setFooter(
                                '<button class="btn btn-secondary" data-panel-close>Cancel</button> ' +
                                '<button class="btn btn-primary" data-fork-save>Fork Selected</button>'
                            );

                            var footer = panelApi.panel.querySelector('[data-panel-footer]');
                            if (footer) {
                                var cancelBtn = footer.querySelector('[data-panel-close]');
                                if (cancelBtn) cancelBtn.addEventListener('click', panelApi.close);

                                var saveBtn = footer.querySelector('[data-fork-save]');
                                if (saveBtn) {
                                    saveBtn.addEventListener('click', function() {
                                        var checked = panelApi.panel.querySelectorAll('input[name="fork_id"]:checked');
                                        if (checked.length === 0) {
                                            app.Toast.show('Select at least one entity to fork', 'warning');
                                            return;
                                        }
                                        saveBtn.disabled = true;
                                        saveBtn.textContent = 'Forking...';

                                        var promises = [];
                                        var typeKey = config.entityType.replace(/s$/, '');
                                        checked.forEach(function(cb) {
                                            var body = {};
                                            body['org_' + typeKey + '_id'] = cb.value;
                                            promises.push(
                                                fetch(app.API_BASE + '/user/fork/' + typeKey.replace('_', '-'), {
                                                    method: 'POST',
                                                    headers: { 'Content-Type': 'application/json' },
                                                    body: JSON.stringify(body)
                                                })
                                            );
                                        });

                                        Promise.all(promises).then(function(results) {
                                            var ok = results.filter(function(r) { return r.ok; }).length;
                                            app.Toast.show('Forked ' + ok + ' ' + config.entityLabel + '(s)', 'success');
                                            panelApi.close();
                                            if (config.onForked) config.onForked();
                                            else setTimeout(function() { window.location.reload(); }, 500);
                                        }).catch(function() {
                                            app.Toast.show('Fork failed', 'error');
                                            saveBtn.disabled = false;
                                            saveBtn.textContent = 'Fork Selected';
                                        });
                                    });
                                }
                            }
                        })
                        .catch(function() {
                            panelApi.setBody('<p style="color:var(--danger);text-align:center;padding:var(--space-4)">Failed to load forkable entities.</p>');
                        });
                },
                close: panelApi.close,
                panel: panelApi
            };
        },

        formatJson: function(data) {
            if (typeof data === 'string') {
                try { data = JSON.parse(data); } catch (e) { return app.escapeHtml(data); }
            }
            return '<pre class="json-view">' + app.escapeHtml(JSON.stringify(data, null, 2)) + '</pre>';
        },

        renderSourceBadge: function(baseId) {
            if (baseId) {
                return '<span class="fork-indicator forked">' +
                    '<svg class="fork-icon" viewBox="0 0 16 16" fill="currentColor"><path d="M5 3.25a.75.75 0 11-1.5 0 .75.75 0 011.5 0zm0 2.122a2.25 2.25 0 10-1.5 0v.878A2.25 2.25 0 005.75 8.5h1.5v2.128a2.251 2.251 0 101.5 0V8.5h1.5a2.25 2.25 0 002.25-2.25v-.878a2.25 2.25 0 10-1.5 0v.878a.75.75 0 01-.75.75h-4.5A.75.75 0 015 6.25v-.878z"/></svg>' +
                    'forked</span>';
            }
            return '<span class="fork-indicator custom">' +
                '<svg class="fork-icon" viewBox="0 0 16 16" fill="currentColor"><path d="M8 2a6 6 0 100 12A6 6 0 008 2zm.75 3.75v2.5h2.5v1.5h-2.5v2.5h-1.5v-2.5h-2.5v-1.5h2.5v-2.5h1.5z"/></svg>' +
                'custom</span>';
        }
    };

    // Expose MyCommon on app
    app.MyCommon = MyCommon;

    // ---- Page initializers ----

    app.initMyPlugins = function() {
        MyCommon.initExpandRows('#my-plugins-table');
        MyCommon.initSearch('my-plugins-search', '#my-plugins-table');
        MyCommon.initFilterSelect('my-plugins-category-filter', '#my-plugins-table', 'data-category');
        MyCommon.initBulkActions('#my-plugins-table', 'my-plugins-bulk-bar');

        var forkBtn = document.getElementById('my-plugins-fork-btn');
        if (forkBtn) {
            var forkPanel = MyCommon.initForkPanel({
                panelId: 'fork-panel',
                entityType: 'plugins',
                entityLabel: 'plugin'
            });
            if (forkPanel) {
                forkBtn.addEventListener('click', forkPanel.open);
            }
        }
    };

    app.initMySkills = function() {
        MyCommon.initExpandRows('#my-skills-table');
        MyCommon.initSearch('my-skills-search', '#my-skills-table');
        MyCommon.initFilterSelect('my-skills-tag-filter', '#my-skills-table', 'data-tags');
        MyCommon.initBulkActions('#my-skills-table', 'my-skills-bulk-bar');

        var forkBtn = document.getElementById('my-skills-fork-btn');
        if (forkBtn) {
            var forkPanel = MyCommon.initForkPanel({
                panelId: 'fork-panel',
                entityType: 'skills',
                entityLabel: 'skill'
            });
            if (forkPanel) {
                forkBtn.addEventListener('click', forkPanel.open);
            }
        }
    };

    app.initMyAgents = function() {
        MyCommon.initExpandRows('#my-agents-table');
        MyCommon.initSearch('my-agents-search', '#my-agents-table');
        MyCommon.initBulkActions('#my-agents-table', 'my-agents-bulk-bar');

        var forkBtn = document.getElementById('my-agents-fork-btn');
        if (forkBtn) {
            var forkPanel = MyCommon.initForkPanel({
                panelId: 'fork-panel',
                entityType: 'agents',
                entityLabel: 'agent'
            });
            if (forkPanel) {
                forkBtn.addEventListener('click', forkPanel.open);
            }
        }
    };

    app.initMyMcpServers = function() {
        MyCommon.initExpandRows('#my-mcp-table');
        MyCommon.initSearch('my-mcp-search', '#my-mcp-table');
        MyCommon.initBulkActions('#my-mcp-table', 'my-mcp-bulk-bar');

        var forkBtn = document.getElementById('my-mcp-fork-btn');
        if (forkBtn) {
            var forkPanel = MyCommon.initForkPanel({
                panelId: 'fork-panel',
                entityType: 'mcp-servers',
                entityLabel: 'MCP server'
            });
            if (forkPanel) {
                forkBtn.addEventListener('click', forkPanel.open);
            }
        }
    };

    app.initMyHooks = function() {
        MyCommon.initExpandRows('#my-hooks-table');
        MyCommon.initSearch('my-hooks-search', '#my-hooks-table');
        MyCommon.initBulkActions('#my-hooks-table', 'my-hooks-bulk-bar');

        var forkBtn = document.getElementById('my-hooks-fork-btn');
        if (forkBtn) {
            var forkPanel = MyCommon.initForkPanel({
                panelId: 'fork-panel',
                entityType: 'hooks',
                entityLabel: 'hook'
            });
            if (forkPanel) {
                forkBtn.addEventListener('click', forkPanel.open);
            }
        }
    };

    app.initMyMarketplace = function() {
        MyCommon.initExpandRows('#my-marketplace-table');
        MyCommon.initSearch('my-marketplace-search', '#my-marketplace-table');
        MyCommon.initFilterSelect('my-marketplace-source-filter', '#my-marketplace-table', 'data-source');
        MyCommon.initFilterSelect('my-marketplace-category-filter', '#my-marketplace-table', 'data-category');

        // Customize button handler
        document.addEventListener('click', function(e) {
            var btn = e.target.closest('[data-customize-plugin]');
            if (!btn) return;
            var pluginId = btn.getAttribute('data-customize-plugin');
            btn.disabled = true;
            btn.textContent = 'Customizing...';

            fetch(app.API_BASE + '/user/fork/plugin', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ org_plugin_id: pluginId })
            }).then(function(res) {
                if (res.ok) {
                    return res.json().then(function(data) {
                        app.Toast.show('Plugin customized successfully', 'success');
                        if (data.plugin && data.plugin.plugin_id) {
                            setTimeout(function() {
                                window.location.href = '/admin/my/plugins/edit?id=' + encodeURIComponent(data.plugin.plugin_id);
                            }, 500);
                        } else {
                            setTimeout(function() { window.location.reload(); }, 500);
                        }
                    });
                } else {
                    app.Toast.show('Failed to customize plugin', 'error');
                    btn.disabled = false;
                    btn.textContent = 'Customize';
                }
            }).catch(function() {
                app.Toast.show('Failed to customize plugin', 'error');
                btn.disabled = false;
                btn.textContent = 'Customize';
            });
        });
    };

})(window.AdminApp || (window.AdminApp = {}));
