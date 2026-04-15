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
    // Display-only: decoded for UI hints. All API auth is server-side.
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

    const DropdownManager = {
        portal: null,
        activeMenu: null,
        activeDropdown: null,
        activeTrigger: null,

        init: function() {
            let portal = document.getElementById('dropdown-portal');
            if (!portal) {
                portal = document.createElement('div');
                portal.id = 'dropdown-portal';
                portal.style.cssText = 'position:fixed;top:0;left:0;width:0;height:0;z-index:1000;pointer-events:none;';
                document.body.append(portal);
            }
            DropdownManager.portal = portal;

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
        }).join('') : '<div class="empty-state" style="padding:var(--sp-space-4)"><p>None available.</p></div>';
        let filterRow = '<input type="text" class="field-input" placeholder="Filter..." data-filter-list="' + escapeHtml(id) + '" style="margin-bottom:var(--sp-space-2)">';
        if (options.hasSelectAll) {
            filterRow = '<div style="display:flex;gap:var(--sp-space-2);margin-bottom:var(--sp-space-2)">' +
                '<input type="text" class="field-input" placeholder="Search..." data-filter-list="' + escapeHtml(id) + '" style="flex:1">' +
                '<button type="button" class="btn btn-secondary btn-sm" data-select-all="' + escapeHtml(id) + '">Select All</button>' +
                '<button type="button" class="btn btn-secondary btn-sm" data-deselect-all="' + escapeHtml(id) + '">Deselect All</button>' +
            '</div>';
        }
        const maxHeight = options.hasSelectAll ? '300px' : '200px';
        return '<div class="form-group">' +
            '<label class="field-label">' + escapeHtml(label) + '</label>' +
            filterRow +
            '<div class="checklist-container" data-checklist="' + escapeHtml(id) + '" style="max-height:' + maxHeight + ';overflow-y:auto;border:1px solid var(--sp-border-subtle);border-radius:var(--sp-radius-md);padding:var(--sp-space-2)">' +
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
                item.style.display = (q && !name.includes(q)) ? 'none' : '';
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
                th.append(icon);
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
            tbody.append(sortableRows[j]);
            const eventId = sortableRows[j].getAttribute('data-event-id');
            if (eventId && detailMap[eventId]) {
                tbody.append(detailMap[eventId]);
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
                    rows[i].style.display = (!q || searchVal.includes(q)) ? '' : 'none';
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
        var text = copyBtn.getAttribute('data-copy');
        navigator.clipboard.writeText(text).then(function() {
            copyBtn.classList.add('copy-success');
            setTimeout(function() { copyBtn.classList.remove('copy-success'); }, 2000);
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

    document.addEventListener('DOMContentLoaded', function() {
        const tokenEl = document.querySelector('.install-token-value[data-masked="true"]');
        if (tokenEl) tokenEl.style.filter = 'blur(4px)';
    });
})(window.AdminApp);
