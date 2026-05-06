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
            const iconSpan = document.createElement('span');
            iconSpan.className = 'toast-icon';
            iconSpan.textContent = icon;
            const msgSpan = document.createElement('span');
            msgSpan.className = 'toast-message';
            msgSpan.textContent = message;
            el.append(iconSpan, msgSpan);
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

    const on = (eventType, selector, handler, options) => {
        const entry = { selector, handler, exclusive: (options && options.exclusive) || false };
        if (handlers[eventType]) {
            handlers[eventType].push(entry);
        }
    };

    const dispatch = (entries, e) => {
        for (let i = 0; i < entries.length; i++) {
            const entry = entries[i];
            const match = e.target.closest(entry.selector);
            if (match) {
                entry.handler(e, match);
                if (entry.exclusive) return true;
            }
        }
        return false;
    };

    const init = () => {
        document.addEventListener('click', (e) => {
            const handled = dispatch(handlers.click, e);
            if (!handled && app.shared) {
                app.shared.closeAllMenus();
            }
        });

        document.addEventListener('change', (e) => {
            dispatch(handlers.change, e);
        });

        document.addEventListener('input', (e) => {
            dispatch(handlers.input, e);
        });

        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && app.shared) {
                app.shared.closeAllMenus();
            }
            dispatch(handlers.keydown, e);
        });
    };

    app.events = { on, init };
})(window.AdminApp);

(function(app) {
    'use strict';

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
        if (app._closeSidebar) {
            app._closeSidebar();
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
        const dialog = document.createElement('div');
        dialog.className = 'confirm-dialog';
        const h3 = document.createElement('h3');
        h3.textContent = title;
        const p = document.createElement('p');
        p.textContent = message;
        const btnRow = document.createElement('div');
        btnRow.style.cssText = 'display:flex;gap:var(--sp-space-3);justify-content:flex-end;margin-top:var(--sp-space-5)';
        const cancelBtn = document.createElement('button');
        cancelBtn.className = 'btn btn-secondary';
        cancelBtn.setAttribute('data-action', 'cancel');
        cancelBtn.textContent = 'Cancel';
        const confirmBtn = document.createElement('button');
        confirmBtn.className = 'btn ' + btnClass;
        confirmBtn.setAttribute('data-action', 'confirm');
        confirmBtn.textContent = confirmLabel;
        btnRow.append(cancelBtn, confirmBtn);
        dialog.append(h3, p, btnRow);
        overlay.append(dialog);
        cancelBtn.addEventListener('click', () => overlay.remove());
        confirmBtn.addEventListener('click', () => {
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
        const dialog = document.createElement('div');
        dialog.className = 'confirm-dialog';
        const h3 = document.createElement('h3');
        h3.textContent = title;
        const p = document.createElement('p');
        p.textContent = 'This action cannot be undone.';
        const btnRow = document.createElement('div');
        btnRow.style.cssText = 'display:flex;gap:var(--sp-space-3);justify-content:flex-end;margin-top:var(--sp-space-5)';
        const cancelBtn = document.createElement('button');
        cancelBtn.className = 'btn btn-secondary';
        cancelBtn.setAttribute('data-confirm-cancel', '');
        cancelBtn.textContent = 'Cancel';
        const deleteBtn = document.createElement('button');
        deleteBtn.className = 'btn btn-danger';
        deleteBtn.setAttribute('data-confirm-delete', itemId);
        deleteBtn.textContent = 'Delete';
        btnRow.append(cancelBtn, deleteBtn);
        dialog.append(h3, p, btnRow);
        overlay.append(dialog);
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
        el.replaceChildren();
        const wrapper = document.createElement('div');
        wrapper.className = 'loading-spinner';
        const spinner = document.createElement('div');
        spinner.className = 'spinner';
        const p = document.createElement('p');
        p.textContent = msg || 'Loading...';
        wrapper.append(spinner, p);
        el.append(wrapper);
    }

    function showEmpty(el, msg) {
        el.replaceChildren();
        const wrapper = document.createElement('div');
        wrapper.className = 'empty-state';
        const p = document.createElement('p');
        p.textContent = msg;
        wrapper.append(p);
        el.append(wrapper);
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

        const group = document.createElement('div');
        group.className = 'form-group';

        const labelEl = document.createElement('label');
        labelEl.className = 'field-label';
        labelEl.textContent = label;
        group.append(labelEl);

        const filterInput = document.createElement('input');
        filterInput.type = 'text';
        filterInput.className = 'field-input';
        filterInput.setAttribute('data-filter-list', id);

        if (options.hasSelectAll) {
            filterInput.placeholder = 'Search...';
            filterInput.style.flex = '1';
            const filterRow = document.createElement('div');
            filterRow.style.cssText = 'display:flex;gap:var(--sp-space-2);margin-bottom:var(--sp-space-2)';
            const selectAllBtn = document.createElement('button');
            selectAllBtn.type = 'button';
            selectAllBtn.className = 'btn btn-secondary btn-sm';
            selectAllBtn.setAttribute('data-select-all', id);
            selectAllBtn.textContent = 'Select All';
            const deselectAllBtn = document.createElement('button');
            deselectAllBtn.type = 'button';
            deselectAllBtn.className = 'btn btn-secondary btn-sm';
            deselectAllBtn.setAttribute('data-deselect-all', id);
            deselectAllBtn.textContent = 'Deselect All';
            filterRow.append(filterInput, selectAllBtn, deselectAllBtn);
            group.append(filterRow);
        } else {
            filterInput.placeholder = 'Filter...';
            filterInput.style.marginBottom = 'var(--sp-space-2)';
            group.append(filterInput);
        }

        const maxHeight = options.hasSelectAll ? '300px' : '200px';
        const container = document.createElement('div');
        container.className = 'checklist-container';
        container.setAttribute('data-checklist', id);
        container.style.cssText = 'max-height:' + maxHeight + ';overflow-y:auto;border:1px solid var(--sp-border-subtle);border-radius:var(--sp-radius-md);padding:var(--sp-space-2)';

        const hasItems = items && items.length > 0;
        if (hasItems) {
            items.forEach(function(item) {
                const val = typeof item === 'string' ? item : (item.name || item.id || item);
                const displayName = typeof item === 'string' ? item : (item.name || item.id || String(item));
                const itemId = id + '-chk-' + val.replace(/[^a-zA-Z0-9_-]/g, '_');
                const itemDiv = document.createElement('div');
                itemDiv.className = 'checklist-item';
                itemDiv.setAttribute('data-item-name', val.toLowerCase());
                const checkbox = document.createElement('input');
                checkbox.type = 'checkbox';
                checkbox.name = id;
                checkbox.value = val;
                checkbox.id = itemId;
                if (selectedSet[val]) checkbox.checked = true;
                const itemLabel = document.createElement('label');
                itemLabel.setAttribute('for', itemId);
                itemLabel.textContent = displayName;
                itemDiv.append(checkbox, itemLabel);
                container.append(itemDiv);
            });
        } else {
            const emptyDiv = document.createElement('div');
            emptyDiv.className = 'empty-state';
            emptyDiv.style.padding = 'var(--sp-space-4)';
            const emptyP = document.createElement('p');
            emptyP.textContent = 'None available.';
            emptyDiv.append(emptyP);
            container.append(emptyDiv);
        }

        group.append(container);
        return group;
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

        sortableRows.sort((a, b) => {
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

        app.events.on('input', '#' + searchInputId, (e, el) => {
            let debounceTimer = el._debounceTimer || null;
            clearTimeout(debounceTimer);
            el._debounceTimer = setTimeout(() => {
                const q = el.value.toLowerCase().trim();
                const rows = document.querySelectorAll('.data-table tbody tr');
                for (let i = 0; i < rows.length; i++) {
                    const searchVal = rows[i].getAttribute(searchAttr) || rows[i].textContent.toLowerCase();
                    rows[i].style.display = (!q || searchVal.includes(q)) ? '' : 'none';
                }
            }, 200);
        });

        app.events.on('click', '[data-action="delete"]', (e, deleteBtn) => {
            shared.closeAllMenus();
            const entityId = deleteBtn.getAttribute('data-entity-id');
            const deleteEntityType = deleteBtn.getAttribute('data-entity-type') || entityType;
            showDeleteDialog(deleteEntityType, entityId, opts);
        }, { exclusive: true });

        app.events.on('click', '[data-confirm-delete]', (e, confirmBtn) => {
            const deleteId = confirmBtn.getAttribute('data-confirm-delete');
            performDelete(entityType, deleteId, confirmBtn, opts);
        }, { exclusive: true });

        app.events.on('change', '[data-action="toggle"]', (e, toggle) => {
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
            }).then(() => {
                app.Toast.show(toggleType + ' ' + (enabled ? 'enabled' : 'disabled'), 'success');
            }).catch((err) => {
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
        app.api(apiPath, { method: 'DELETE' }).then(() => {
            app.Toast.show(entityType + ' deleted', 'success');
            shared.closeDeleteConfirm();
            window.location.reload();
        }).catch((err) => {
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

        form.addEventListener('submit', (e) => {
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
                .then(() => {
                    app.Toast.show(entity + ' saved!', 'success');
                    window.location.href = app.BASE + listPath;
                })
                .catch((err) => {
                    app.Toast.show(err.message || 'Failed to save ' + entity, 'error');
                    if (submitBtn) { submitBtn.disabled = false; submitBtn.textContent = isEdit ? 'Save Changes' : 'Create'; }
                });
        });
    }

    function formDataToObject(formData) {
        const obj = {};
        formData.forEach((value, key) => {
            if (key === 'tags') {
                obj[key] = value.split(',').map((t) => t.trim()).filter(Boolean);
            } else {
                obj[key] = value;
            }
        });
        return obj;
    }

    document.addEventListener('DOMContentLoaded', () => {
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
            const savedNodes = Array.from(copyBtn.childNodes).map((n) => n.cloneNode(true));
            copyBtn.replaceChildren();
            const checkSpan = document.createElement('span');
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

((app) => {
    'use strict';

    const OrgCommon = {

        initExpandRows: (tableSelector, renderCallback) => {
            const table = document.querySelector(tableSelector);
            if (!table) return;

            table.addEventListener('click', (e) => {
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

        handleRowClick: (row, detailRow) => {
            const isVisible = detailRow.classList.contains('visible');

            const table = row.closest('table');
            if (table) {
                table.querySelectorAll('tr.detail-row.visible').forEach((r) => {
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

        initSidePanel: (panelId) => {
            const panel = document.getElementById(panelId);
            if (!panel) return null;

            const overlayId = panel.getAttribute('data-overlay') || (panelId + '-overlay');
            const overlay = document.getElementById(overlayId);
            const closeBtn = panel.querySelector('[data-panel-close]');

            const api = {
                open: () => {
                    panel.classList.add('open');
                    if (overlay) overlay.classList.add('active');
                },
                close: () => {
                    panel.classList.remove('open');
                    if (overlay) overlay.classList.remove('active');
                },
                setTitle: (text) => {
                    const title = panel.querySelector('[data-panel-title]');
                    if (title) title.textContent = text;
                },
                setBody: (content) => {
                    const body = panel.querySelector('[data-panel-body]');
                    if (!body) return;
                    body.replaceChildren();
                    if (typeof content === 'string') {
                        body.textContent = content;
                    } else if (content instanceof Node) {
                        body.append(content);
                    }
                },
                setBodyDom: (el) => {
                    const body = panel.querySelector('[data-panel-body]');
                    if (!body) return;
                    body.replaceChildren();
                    body.append(el);
                },
                setFooterDom: (el) => {
                    const footer = panel.querySelector('[data-panel-footer]');
                    if (!footer) return;
                    footer.replaceChildren();
                    footer.append(el);
                },
                panel: panel
            };

            if (closeBtn) closeBtn.addEventListener('click', api.close);
            if (overlay) overlay.addEventListener('click', api.close);

            return api;
        },

        initAssignPanel: (config) => {
            const panelApi = OrgCommon.initSidePanel(config.panelId);
            if (!panelApi) return null;

            return {
                open: (entityId, entityName, currentPluginIds) => {
                    panelApi.setTitle('Assign ' + (entityName || entityId));

                    const allPlugins = config.allPlugins || [];
                    const currentSet = {};
                    (currentPluginIds || []).forEach((id) => { currentSet[id] = true; });

                    const checklist = document.createElement('div');
                    checklist.className = 'assign-panel-checklist';

                    if (allPlugins.length === 0) {
                        const p = document.createElement('p');
                        p.style.cssText = 'color:var(--sp-text-tertiary);font-size:var(--sp-text-sm)';
                        p.textContent = 'No plugins available.';
                        checklist.append(p);
                    } else {
                        allPlugins.forEach((p) => {
                            const label = document.createElement('label');
                            label.className = 'acl-checkbox-row';
                            const input = document.createElement('input');
                            input.type = 'checkbox';
                            input.name = 'plugin_id';
                            input.value = p.id;
                            if (currentSet[p.id]) input.checked = true;
                            const span = document.createElement('span');
                            span.className = 'acl-checkbox-label';
                            span.textContent = p.name || p.id;
                            label.append(input, span);
                            checklist.append(label);
                        });
                    }

                    panelApi.setBodyDom(checklist);

                    const footerFrag = document.createDocumentFragment();
                    const cancelBtn = document.createElement('button');
                    cancelBtn.className = 'btn btn-secondary';
                    cancelBtn.setAttribute('data-panel-close', '');
                    cancelBtn.textContent = 'Cancel';
                    const saveBtn = document.createElement('button');
                    saveBtn.className = 'btn btn-primary';
                    saveBtn.setAttribute('data-assign-save', '');
                    saveBtn.setAttribute('data-entity-id', entityId);
                    saveBtn.textContent = 'Save';
                    footerFrag.append(cancelBtn, document.createTextNode(' '), saveBtn);
                    panelApi.setFooterDom(footerFrag);

                    cancelBtn.addEventListener('click', panelApi.close);

                    panelApi.open();
                },
                close: panelApi.close,
                panel: panelApi
            };
        },

        initEditPanel: (config) => {
            const panelApi = OrgCommon.initSidePanel(config.panelId);
            if (!panelApi) return null;
            let currentEntityId = null;

            const buildForm = (entityData) => {
                const form = document.createElement('form');
                form.className = 'edit-panel-form';
                (config.fields || []).forEach((f) => {
                    let val = entityData[f.name] || '';
                    if (Array.isArray(val)) val = val.join(', ');
                    const group = document.createElement('div');
                    group.className = 'form-group';
                    const label = document.createElement('label');
                    label.className = 'form-label';
                    label.textContent = f.label;
                    group.append(label);
                    if (f.type === 'textarea') {
                        const textarea = document.createElement('textarea');
                        textarea.className = 'form-control';
                        textarea.name = f.name;
                        textarea.rows = f.rows || 10;
                        textarea.textContent = val;
                        group.append(textarea);
                    } else {
                        const input = document.createElement('input');
                        input.type = 'text';
                        input.className = 'form-control';
                        input.name = f.name;
                        input.value = val;
                        if (f.required) input.required = true;
                        group.append(input);
                    }
                    form.append(group);
                });
                return form;
            };

            const collectFormData = () => {
                const form = panelApi.panel.querySelector('.edit-panel-form');
                if (!form) return {};
                const body = {};
                (config.fields || []).forEach((f) => {
                    const el = form.querySelector('[name="' + f.name + '"]');
                    if (!el) return;
                    const val = el.value;
                    if (f.name === 'tags') {
                        body[f.name] = val.split(',').map((t) => t.trim()).filter(Boolean);
                    } else {
                        body[f.name] = val;
                    }
                });
                return body;
            };

            app.events.on('click', '[data-edit-save]', (e, btn) => {
                btn.disabled = true;
                btn.textContent = 'Saving...';
                const body = collectFormData();
                const url = config.apiBasePath + encodeURIComponent(currentEntityId);
                fetch(url, {
                    method: 'PUT',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(body)
                }).then((res) => {
                    if (res.ok) {
                        app.Toast.show((config.entityLabel || 'Item') + ' updated', 'success');
                        panelApi.close();
                        setTimeout(() => { window.location.reload(); }, 500);
                    } else {
                        res.text().then((t) => {
                            app.Toast.show('Failed to save: ' + t, 'error');
                        });
                        btn.disabled = false;
                        btn.textContent = 'Save';
                    }
                }).catch(() => {
                    app.Toast.show('Failed to save', 'error');
                    btn.disabled = false;
                    btn.textContent = 'Save';
                });
            });

            return {
                open: (entityId, entityData) => {
                    currentEntityId = entityId;
                    panelApi.setTitle('Edit ' + (entityData.name || entityId));
                    panelApi.setBodyDom(buildForm(entityData));

                    const footerFrag = document.createDocumentFragment();
                    const cancelBtn = document.createElement('button');
                    cancelBtn.className = 'btn btn-secondary';
                    cancelBtn.setAttribute('data-panel-close', '');
                    cancelBtn.textContent = 'Cancel';
                    const saveBtn = document.createElement('button');
                    saveBtn.className = 'btn btn-primary';
                    saveBtn.setAttribute('data-edit-save', '');
                    saveBtn.textContent = 'Save';
                    footerFrag.append(cancelBtn, document.createTextNode(' '), saveBtn);
                    panelApi.setFooterDom(footerFrag);

                    cancelBtn.addEventListener('click', panelApi.close);
                    panelApi.open();
                },
                close: panelApi.close
            };
        },

        initBulkActions: (tableSelector, barId) => {
            const table = document.querySelector(tableSelector);
            if (!table) return null;

            let selected = {};

            const updateCount = () => {
                const count = Object.keys(selected).length;
                const countEl = document.querySelector('[data-bulk-count]');
                if (countEl) countEl.textContent = count;
                const bar = document.getElementById(barId);
                if (bar) bar.style.display = count > 0 ? 'flex' : 'none';
            };

            table.addEventListener('change', (e) => {
                if (e.target.classList.contains('bulk-select-all')) {
                    const checked = e.target.checked;
                    table.querySelectorAll('.bulk-checkbox').forEach((cb) => {
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
                getSelected: () => Object.keys(selected),
                clear: () => {
                    selected = {};
                    table.querySelectorAll('.bulk-checkbox, .bulk-select-all').forEach((cb) => {
                        cb.checked = false;
                    });
                    updateCount();
                }
            };
        },

        formatJson: (data) => {
            if (typeof data === 'string') {
                try { data = JSON.parse(data); } catch (e) {
                    const span = document.createElement('span');
                    span.textContent = data;
                    return span;
                }
            }
            const pre = document.createElement('pre');
            pre.className = 'json-view';
            pre.textContent = JSON.stringify(data, null, 2);
            return pre;
        },

        renderRoleBadges: (roles) => {
            const frag = document.createDocumentFragment();
            if (!roles || !roles.length) {
                const badge = document.createElement('span');
                badge.className = 'badge badge-gray';
                badge.textContent = 'All';
                frag.append(badge);
                return frag;
            }
            const assigned = roles.filter((r) => r.assigned);
            if (!assigned.length) {
                const badge = document.createElement('span');
                badge.className = 'badge badge-gray';
                badge.textContent = 'All';
                frag.append(badge);
                return frag;
            }
            assigned.forEach((r, i) => {
                if (i > 0) frag.append(document.createTextNode(' '));
                const badge = document.createElement('span');
                badge.className = 'badge badge-blue';
                badge.textContent = r.name;
                frag.append(badge);
            });
            return frag;
        },

        renderDeptBadges: (departments) => {
            const frag = document.createDocumentFragment();
            if (!departments || !departments.length) {
                const badge = document.createElement('span');
                badge.className = 'badge badge-gray';
                badge.textContent = 'None';
                frag.append(badge);
                return frag;
            }
            const assigned = departments.filter((d) => d.assigned);
            if (!assigned.length) {
                const badge = document.createElement('span');
                badge.className = 'badge badge-gray';
                badge.textContent = 'None';
                frag.append(badge);
                return frag;
            }
            assigned.forEach((d, i) => {
                if (i > 0) frag.append(document.createTextNode(' '));
                const cls = d.default_included ? 'badge-yellow' : 'badge-green';
                const badge = document.createElement('span');
                badge.className = 'badge ' + cls;
                badge.textContent = d.name;
                frag.append(badge);
            });
            return frag;
        },

        renderPluginBadges: (plugins) => {
            const frag = document.createDocumentFragment();
            if (!plugins || !plugins.length) {
                const badge = document.createElement('span');
                badge.className = 'badge badge-gray';
                badge.textContent = 'None';
                frag.append(badge);
                return frag;
            }
            plugins.forEach((p, i) => {
                if (i > 0) frag.append(document.createTextNode(' '));
                const name = typeof p === 'string' ? p : (p.name || p.id || p);
                const badge = document.createElement('span');
                badge.className = 'badge badge-purple';
                badge.textContent = name;
                frag.append(badge);
            });
            return frag;
        },

        initFilters: (searchInputId, tableSelector, filters) => {
            const table = document.querySelector(tableSelector);
            if (!table) return;

            const applyFilters = () => {
                const searchInput = document.getElementById(searchInputId);
                const q = (searchInput ? searchInput.value : '').toLowerCase().trim();
                const filterValues = filters.map((f) => {
                    const sel = document.getElementById(f.selectId);
                    return { attr: f.dataAttr, value: sel ? sel.value : '' };
                });

                table.querySelectorAll('tbody tr.clickable-row').forEach((row) => {
                    const matchSearch = !q ||
                        (row.getAttribute('data-name') || '').includes(q) ||
                        (row.getAttribute('data-skill-id') || row.getAttribute('data-agent-id') || '').toLowerCase().includes(q) ||
                        (row.getAttribute('data-description') || '').includes(q);

                    const matchFilters = filterValues.every((fv) => {
                        if (!fv.value) return true;
                        const rowVal = row.getAttribute(fv.attr) || '';
                        return rowVal.includes(fv.value);
                    });

                    const match = matchSearch && matchFilters;
                    row.style.display = match ? '' : 'none';
                    const detail = row.nextElementSibling;
                    if (detail && detail.classList.contains('detail-row')) {
                        if (!match) { detail.style.display = 'none'; detail.classList.remove('visible'); }
                        else { detail.style.display = ''; }
                    }
                });
            };

            filters.forEach((f) => {
                const sel = document.getElementById(f.selectId);
                if (sel) sel.addEventListener('change', applyFilters);
            });

            let searchTimer = null;
            const searchInput = document.getElementById(searchInputId);
            if (searchInput) {
                searchInput.addEventListener('input', () => {
                    clearTimeout(searchTimer);
                    searchTimer = setTimeout(applyFilters, 200);
                });
            }

            return { apply: applyFilters };
        },

        formatTimeAgo: (isoString) => {
            if (!isoString) return '--';
            const date = new Date(isoString);
            if (isNaN(date.getTime())) return '--';
            const now = new Date();
            const diff = Math.floor((now - date) / 1000);
            if (diff < 60) return 'just now';
            if (diff < 3600) return Math.floor(diff / 60) + 'm ago';
            if (diff < 86400) return Math.floor(diff / 3600) + 'h ago';
            if (diff < 2592000) return Math.floor(diff / 86400) + 'd ago';
            return date.toLocaleDateString();
        },

        initTimeAgo: () => {
            document.querySelectorAll('.metadata-timestamp').forEach((el) => {
                const iso = el.getAttribute('title') || el.textContent.trim();
                if (iso && iso !== '--') {
                    el.textContent = OrgCommon.formatTimeAgo(iso);
                    el.setAttribute('title', new Date(iso).toLocaleString());
                }
            });
        }
    };

    app.OrgCommon = OrgCommon;
})(window.AdminApp = window.AdminApp || {});

(function (global) {
    'use strict';

    var DEFAULT_ARRAY_PAGE = 50;

    function typeOf(value) {
        if (value === null) return 'null';
        if (Array.isArray(value)) return 'array';
        return typeof value;
    }

    function escapeText(str) {
        return String(str)
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;');
    }

    function formatPathSegment(key, isArrayIndex) {
        if (isArrayIndex) return '[' + key + ']';
        if (/^[A-Za-z_$][A-Za-z0-9_$]*$/.test(key)) return '.' + key;
        return '[' + JSON.stringify(key) + ']';
    }

    function joinPath(parentPath, key, isArrayIndex) {
        if (parentPath === '' && !isArrayIndex) {
            return /^[A-Za-z_$][A-Za-z0-9_$]*$/.test(key) ? key : '[' + JSON.stringify(key) + ']';
        }
        return parentPath + formatPathSegment(key, isArrayIndex);
    }

    function makeBtn(cls, label, title) {
        var b = document.createElement('button');
        b.type = 'button';
        b.className = cls;
        if (label !== undefined) b.textContent = label;
        if (title) b.title = title;
        return b;
    }

    function copyToClipboard(text) {
        if (navigator.clipboard && navigator.clipboard.writeText) {
            return navigator.clipboard.writeText(text);
        }
        return new Promise(function (resolve, reject) {
            try {
                var ta = document.createElement('textarea');
                ta.value = text;
                ta.setAttribute('readonly', '');
                ta.style.position = 'absolute';
                ta.style.left = '-9999px';
                document.body.appendChild(ta);
                ta.select();
                document.execCommand('copy');
                document.body.removeChild(ta);
                resolve();
            } catch (e) {
                reject(e);
            }
        });
    }

    function flashCopied(btn) {
        btn.classList.add('is-copied');
        var prev = btn.textContent;
        btn.textContent = '✓';
        setTimeout(function () {
            btn.classList.remove('is-copied');
            btn.textContent = prev;
        }, 900);
    }

    function buildPathCopy(path) {
        if (!path) return null;
        var b = makeBtn('json-tree__path-copy', '⧉', 'Copy path: ' + path);
        b.setAttribute('aria-label', 'Copy path ' + path);
        b.dataset.path = path;
        b.addEventListener('click', function (e) {
            e.stopPropagation();
            copyToClipboard(path).then(function () {
                flashCopied(b);
            }).catch(function () {});
        });
        return b;
    }

    function valueClass(t) {
        if (t === 'boolean') return 'bool';
        if (t === 'undefined') return 'null';
        return t;
    }

    function renderPrimitive(value) {
        var span = document.createElement('span');
        var t = typeOf(value);
        span.classList.add('json-tree__value', 'json-tree__value--' + valueClass(t));
        if (t === 'string') {
            span.textContent = '"' + value + '"';
        } else if (t === 'null') {
            span.textContent = 'null';
        } else if (t === 'undefined') {
            span.textContent = 'undefined';
        } else {
            span.textContent = String(value);
        }
        return span;
    }

    function summaryText(value) {
        var t = typeOf(value);
        if (t === 'array') return 'Array(' + value.length + ')';
        if (t === 'object') {
            var keys = Object.keys(value);
            return '{' + keys.length + (keys.length === 1 ? ' key' : ' keys') + '}';
        }
        return '';
    }

    function renderNode(opts) {
        var value = opts.value;
        var keyLabel = opts.keyLabel; // string or null
        var isArrayIndex = !!opts.isArrayIndex;
        var path = opts.path;
        var depth = opts.depth;
        var seen = opts.seen;
        var startCollapsed = !!opts.startCollapsed;

        var node = document.createElement('div');
        node.className = 'json-tree__node';
        node.style.setProperty('--depth', String(depth));

        var row = document.createElement('span');
        row.className = 'json-tree__row';
        node.appendChild(row);

        var t = typeOf(value);
        var isContainer = (t === 'object' || t === 'array');

        var toggle = makeBtn('json-tree__toggle', '', isContainer ? 'Toggle' : '');
        if (!isContainer) toggle.classList.add('json-tree__toggle--leaf');
        row.appendChild(toggle);

        if (keyLabel !== null && keyLabel !== undefined) {
            var keyEl = document.createElement('span');
            keyEl.className = isArrayIndex ? 'json-tree__index' : 'json-tree__key';
            keyEl.textContent = keyLabel;
            row.appendChild(keyEl);
        }

        if (isContainer) {
            // Circular reference defense
            if (seen.indexOf(value) !== -1) {
                var circ = document.createElement('span');
                circ.className = 'json-tree__circular';
                circ.textContent = '[circular]';
                row.appendChild(circ);
                return node;
            }

            var openCh = (t === 'array') ? '[' : '{';
            var closeCh = (t === 'array') ? ']' : '}';

            var open = document.createElement('span');
            open.className = 'json-tree__punct';
            open.textContent = openCh;
            row.appendChild(open);

            var summary = document.createElement('span');
            summary.className = 'json-tree__summary';
            summary.textContent = summaryText(value);
            row.appendChild(summary);

            var close = document.createElement('span');
            close.className = 'json-tree__punct';
            close.textContent = closeCh;
            close.style.marginLeft = '0.25rem';
            row.appendChild(close);

            if (path) {
                var pc = buildPathCopy(path);
                if (pc) row.appendChild(pc);
            }

            var children = document.createElement('div');
            children.className = 'json-tree__children';
            node.appendChild(children);

            var childrenBuilt = false;
            var renderedCount = 0;

            function buildChildren(limit) {
                var nextSeen = seen.concat([value]);
                var keys = (t === 'array') ? null : Object.keys(value);
                var total = (t === 'array') ? value.length : keys.length;
                var stop = Math.min(total, limit);

                for (var i = renderedCount; i < stop; i++) {
                    var k, v, label, isIdx;
                    if (t === 'array') {
                        k = i;
                        v = value[i];
                        label = String(i);
                        isIdx = true;
                    } else {
                        k = keys[i];
                        v = value[k];
                        label = k;
                        isIdx = false;
                    }
                    var childPath = joinPath(path, isIdx ? k : k, isIdx);
                    var childNode = renderNode({
                        value: v,
                        keyLabel: label,
                        isArrayIndex: isIdx,
                        path: childPath,
                        depth: depth + 1,
                        seen: nextSeen,
                        startCollapsed: false
                    });
                    children.appendChild(childNode);
                }
                renderedCount = stop;

                // existing "expand more" button
                var existing = children.querySelector(':scope > .json-tree__expand-more');
                if (existing) existing.remove();

                if (renderedCount < total) {
                    var more = makeBtn('json-tree__expand-more', 'Show ' + Math.min(DEFAULT_ARRAY_PAGE, total - renderedCount) + ' more (' + (total - renderedCount) + ' remaining)');
                    more.style.setProperty('--depth', String(depth));
                    more.addEventListener('click', function (e) {
                        e.stopPropagation();
                        buildChildren(renderedCount + DEFAULT_ARRAY_PAGE);
                    });
                    children.appendChild(more);
                }
            }

            function ensureBuilt() {
                if (childrenBuilt) return;
                childrenBuilt = true;
                var total = (t === 'array') ? value.length : Object.keys(value).length;
                var initial = (t === 'array' && total > DEFAULT_ARRAY_PAGE) ? DEFAULT_ARRAY_PAGE : total;
                buildChildren(initial);
            }

            // Decide initial collapsed state. Large arrays start collapsed by default.
            var totalChildren = (t === 'array') ? value.length : Object.keys(value).length;
            var collapsed = startCollapsed || (t === 'array' && totalChildren > DEFAULT_ARRAY_PAGE) || depth >= 6;

            function setCollapsed(c) {
                if (c) {
                    node.classList.add('is-collapsed');
                    children.style.display = 'none';
                } else {
                    node.classList.remove('is-collapsed');
                    children.style.display = '';
                    ensureBuilt();
                }
            }

            setCollapsed(collapsed);

            toggle.addEventListener('click', function (e) {
                e.stopPropagation();
                setCollapsed(!node.classList.contains('is-collapsed'));
            });

            // also let clicking the punctuation toggle
            open.addEventListener('click', function (e) {
                e.stopPropagation();
                setCollapsed(!node.classList.contains('is-collapsed'));
            });
            close.addEventListener('click', function (e) {
                e.stopPropagation();
                setCollapsed(!node.classList.contains('is-collapsed'));
            });
        } else {
            row.appendChild(renderPrimitive(value));
            if (path) {
                var pc2 = buildPathCopy(path);
                if (pc2) row.appendChild(pc2);
            }
        }

        return node;
    }

    function mountJsonTree(rootEl, value, options) {
        if (!rootEl) return;
        options = options || {};
        rootEl.innerHTML = '';
        rootEl.classList.add('json-tree');

        var rootLabel = options.rootLabel || rootEl.dataset.rootLabel || null;
        var startCollapsed = options.startCollapsed === true || rootEl.dataset.collapsed === '1';
        var rootPath = rootLabel || '';

        try {
            var node = renderNode({
                value: value,
                keyLabel: rootLabel,
                isArrayIndex: false,
                path: rootPath,
                depth: 0,
                seen: [],
                startCollapsed: startCollapsed
            });
            rootEl.appendChild(node);
        } catch (e) {
            var err = document.createElement('div');
            err.className = 'json-tree__error';
            err.textContent = 'Failed to render JSON: ' + (e && e.message ? e.message : String(e));
            rootEl.appendChild(err);
        }
    }

    function parseDataAttr(raw) {
        if (raw === undefined || raw === null) return undefined;
        var trimmed = String(raw).trim();
        if (trimmed === '') return undefined;
        try {
            return JSON.parse(trimmed);
        } catch (e) {
            return { __parseError: e.message, raw: trimmed };
        }
    }

    function autoMount(root) {
        var scope = root || document;
        var els = scope.querySelectorAll('.json-tree[data-json]');
        for (var i = 0; i < els.length; i++) {
            var el = els[i];
            if (el.dataset.jsonTreeMounted === '1') continue;
            el.dataset.jsonTreeMounted = '1';
            var value = parseDataAttr(el.getAttribute('data-json'));
            if (value && typeof value === 'object' && value.__parseError) {
                el.innerHTML = '';
                var err = document.createElement('div');
                err.className = 'json-tree__error';
                err.textContent = 'Invalid JSON: ' + value.__parseError;
                el.appendChild(err);
                continue;
            }
            mountJsonTree(el, value);
        }
    }

    var ns = global.SystempromptAdmin = global.SystempromptAdmin || {};
    ns.jsonTree = {
        mount: mountJsonTree,
        mountJsonTree: mountJsonTree,
        autoMount: autoMount
    };

    // Back-compat: also expose via AdminApp namespace used elsewhere.
    if (global.AdminApp) {
        global.AdminApp.JsonTree = ns.jsonTree;
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', function () { autoMount(); });
    } else {
        autoMount();
    }
})(window);

(function (global) {
    'use strict';

    var DRAWER_ID = 'chain-drawer';
    var QS_KEY = 'chain';
    var FOUR_STAGES = ['scope', 'secret_scan', 'blocklist', 'rate_limit'];

    var lastFocused = null;

    function getDrawer() {
        return document.getElementById(DRAWER_ID);
    }

    function escapeText(s) {
        return String(s == null ? '' : s)
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;');
    }

    function formatCost(microdollars) {
        if (microdollars == null || isNaN(microdollars)) return '—';
        var dollars = Number(microdollars) / 1000000;
        if (dollars === 0) return '$0';
        if (dollars < 0.01) return '$' + dollars.toFixed(6);
        return '$' + dollars.toFixed(4);
    }

    function formatTokens(input, output) {
        if (input == null && output == null) return '—';
        return (input || 0).toLocaleString() + ' / ' + (output || 0).toLocaleString();
    }

    function formatTime(isoString) {
        if (!isoString) return '—';
        try {
            var d = new Date(isoString);
            if (isNaN(d.getTime())) return isoString;
            return d.toISOString().replace('T', ' ').replace(/\..+/, '');
        } catch (e) {
            return isoString;
        }
    }

    function setText(el, value, fallback) {
        if (!el) return;
        var t = (value == null || value === '') ? (fallback || '—') : value;
        el.textContent = String(t);
    }

    function clearChildren(el) {
        if (!el) return;
        while (el.firstChild) el.removeChild(el.firstChild);
    }

    function copyToClipboard(text) {
        if (!text) return;
        if (navigator.clipboard && navigator.clipboard.writeText) {
            navigator.clipboard.writeText(text).catch(function () {});
            return;
        }
        var ta = document.createElement('textarea');
        ta.value = text;
        ta.setAttribute('readonly', '');
        ta.style.position = 'absolute';
        ta.style.left = '-9999px';
        document.body.appendChild(ta);
        ta.select();
        try { document.execCommand('copy'); } catch (e) {}
        document.body.removeChild(ta);
    }

    function renderHeader(drawer, env) {
        var traceEl = drawer.querySelector('[data-chain-trace-id]');
        if (traceEl) {
            traceEl.textContent = env.trace_id || env.session_id || '—';
            traceEl.dataset.value = env.trace_id || env.session_id || '';
        }

        var statusEl = drawer.querySelector('[data-chain-status]');
        if (statusEl) {
            statusEl.classList.remove('chain-drawer__pill--allow', 'chain-drawer__pill--deny');
            var hasDeny = (env.totals && env.totals.deny_count > 0);
            statusEl.textContent = hasDeny ? 'denied' : 'allowed';
            statusEl.classList.add(hasDeny ? 'chain-drawer__pill--deny' : 'chain-drawer__pill--allow');
        }

        var idEl = drawer.querySelector('[data-chain-identity]');
        if (idEl && env.identity) {
            var parts = [];
            if (env.identity.user_id) parts.push('user=' + env.identity.user_id);
            if (env.identity.tenant_id) parts.push('tenant=' + env.identity.tenant_id);
            if (env.identity.agent_id) parts.push('agent=' + env.identity.agent_id);
            idEl.textContent = parts.join(' · ');
        }

        var totals = env.totals || {};
        setText(drawer.querySelector('[data-chain-total="decisions"]'),
                totals.decision_count != null ? totals.decision_count : '—');
        setText(drawer.querySelector('[data-chain-total="denies"]'),
                totals.deny_count != null ? totals.deny_count : '—');
        setText(drawer.querySelector('[data-chain-total="cost"]'),
                formatCost(totals.total_cost_microdollars));
        setText(drawer.querySelector('[data-chain-total="tokens"]'),
                formatTokens(totals.total_input_tokens, totals.total_output_tokens));
    }

    function renderStepper(drawer, env) {
        var byPolicy = {};
        var decisions = (env.decisions || []);
        for (var i = 0; i < decisions.length; i++) {
            var d = decisions[i];
            // First match wins; we want the earliest decision per stage.
            if (!byPolicy[d.policy]) byPolicy[d.policy] = d;
        }

        for (var s = 0; s < FOUR_STAGES.length; s++) {
            var stage = FOUR_STAGES[s];
            var li = drawer.querySelector('.chain-drawer__stage[data-stage="' + stage + '"]');
            if (!li) continue;
            li.classList.remove('chain-drawer__stage--pass', 'chain-drawer__stage--fail',
                                'chain-drawer__stage--skipped');
            // remove any previous detail row
            var oldDetail = li.querySelector('.chain-drawer__stage-detail');
            if (oldDetail) oldDetail.remove();

            var stateEl = li.querySelector('.chain-drawer__stage-state');
            var hit = byPolicy[stage];
            if (!hit) {
                li.classList.add('chain-drawer__stage--skipped');
                if (stateEl) stateEl.textContent = 'skipped';
                continue;
            }

            var failed = (hit.decision === 'deny');
            li.classList.add(failed ? 'chain-drawer__stage--fail' : 'chain-drawer__stage--pass');
            if (stateEl) stateEl.textContent = failed ? 'fail' : 'pass';

            if (hit.reason) {
                var detail = document.createElement('span');
                detail.className = 'chain-drawer__stage-detail';
                detail.textContent = hit.reason;
                li.appendChild(detail);
            }
        }
    }

    function renderEvents(drawer, env) {
        var ul = drawer.querySelector('[data-chain-events]');
        if (!ul) return;
        clearChildren(ul);
        var events = env.events || [];
        if (!events.length) {
            var empty = document.createElement('li');
            empty.className = 'chain-drawer__empty';
            empty.textContent = 'No tool calls.';
            ul.appendChild(empty);
            return;
        }
        for (var i = 0; i < events.length; i++) {
            var ev = events[i];
            var li = document.createElement('li');
            li.className = 'chain-drawer__event';
            var type = document.createElement('span');
            type.className = 'chain-drawer__event-type';
            type.textContent = ev.event_type || '—';
            var tool = document.createElement('span');
            tool.className = 'chain-drawer__event-tool';
            tool.textContent = (ev.tool_name || ev.description || '').slice(0, 200);
            var time = document.createElement('span');
            time.className = 'chain-drawer__event-time';
            time.textContent = formatTime(ev.created_at);
            li.appendChild(type);
            li.appendChild(tool);
            li.appendChild(time);
            ul.appendChild(li);
        }
    }

    function renderRequests(drawer, env) {
        var table = drawer.querySelector('[data-chain-requests]');
        if (!table) return;
        var tbody = table.querySelector('tbody');
        if (!tbody) return;
        clearChildren(tbody);

        var rows = env.requests || [];
        if (!rows.length) {
            var tr = document.createElement('tr');
            var td = document.createElement('td');
            td.colSpan = 6;
            td.className = 'chain-drawer__empty';
            td.textContent = 'No AI requests.';
            tr.appendChild(td);
            tbody.appendChild(tr);
            return;
        }

        for (var i = 0; i < rows.length; i++) {
            var r = rows[i];
            var tr2 = document.createElement('tr');
            tr2.appendChild(td(formatTime(r.created_at)));
            tr2.appendChild(td(r.model || '—'));
            tr2.appendChild(td(r.status || '—'));
            tr2.appendChild(td(formatTokens(r.input_tokens, r.output_tokens)));
            tr2.appendChild(td(r.latency_ms != null ? r.latency_ms + ' ms' : '—'));
            tr2.appendChild(td(formatCost(r.cost_microdollars)));
            tbody.appendChild(tr2);
        }

        function td(text) {
            var c = document.createElement('td');
            c.textContent = text;
            return c;
        }
    }

    function renderTranscript(drawer, env) {
        var holder = drawer.querySelector('[data-chain-transcript]');
        if (!holder) return;
        clearChildren(holder);

        var summary = env.summary;
        if (summary && (summary.ai_title || summary.ai_summary)) {
            if (summary.ai_title) {
                var h = document.createElement('p');
                h.style.fontWeight = '600';
                h.textContent = summary.ai_title;
                holder.appendChild(h);
            }
            if (summary.ai_summary) {
                var p = document.createElement('p');
                p.textContent = summary.ai_summary;
                holder.appendChild(p);
            }
        }

        if (!env.transcript) {
            if (!holder.firstChild) {
                var empty = document.createElement('p');
                empty.className = 'chain-drawer__empty';
                empty.textContent = 'No transcript captured.';
                holder.appendChild(empty);
            }
            return;
        }

        var tEl = document.createElement('div');
        tEl.className = 'json-tree';
        holder.appendChild(tEl);
        mountJson(tEl, env.transcript.transcript, { rootLabel: 'transcript', startCollapsed: true });
    }

    function renderRaw(drawer, env) {
        var holder = drawer.querySelector('[data-chain-raw]');
        if (!holder) return;
        clearChildren(holder);
        var el = document.createElement('div');
        el.className = 'json-tree';
        holder.appendChild(el);
        mountJson(el, env, { rootLabel: 'envelope', startCollapsed: true });
    }

    function mountJson(rootEl, value, options) {
        var ns = global.SystempromptAdmin && global.SystempromptAdmin.jsonTree;
        if (ns && typeof ns.mountJsonTree === 'function') {
            ns.mountJsonTree(rootEl, value, options);
            return;
        }
        // fallback: stringified pre
        var pre = document.createElement('pre');
        try {
            pre.textContent = JSON.stringify(value, null, 2);
        } catch (e) {
            pre.textContent = String(value);
        }
        rootEl.appendChild(pre);
    }

    function renderError(drawer, message) {
        var panel = drawer.querySelector('.chain-drawer__panel');
        if (!panel) return;
        var existing = panel.querySelector('.chain-drawer__error');
        if (existing) existing.remove();
        var div = document.createElement('div');
        div.className = 'chain-drawer__error';
        div.textContent = message;
        panel.insertBefore(div, panel.firstChild.nextSibling);
    }

    function clearError(drawer) {
        var existing = drawer.querySelector('.chain-drawer__error');
        if (existing) existing.remove();
    }

    function showDrawer(drawer) {
        if (!drawer || drawer.hidden === false) return;
        lastFocused = document.activeElement;
        drawer.hidden = false;
        document.body.style.overflow = 'hidden';
        // Defer focus to next frame so animation can begin first.
        requestAnimationFrame(function () {
            var closeBtn = drawer.querySelector('.chain-drawer__close');
            if (closeBtn) closeBtn.focus();
        });
    }

    function hideDrawer() {
        var drawer = getDrawer();
        if (!drawer || drawer.hidden) return;
        drawer.hidden = true;
        document.body.style.overflow = '';
        clearError(drawer);
        if (lastFocused && typeof lastFocused.focus === 'function') {
            lastFocused.focus();
        }
        // Strip ?chain= from URL without reload.
        try {
            var url = new URL(window.location.href);
            if (url.searchParams.has(QS_KEY)) {
                url.searchParams.delete(QS_KEY);
                window.history.replaceState({}, '', url.toString());
            }
        } catch (e) {}
    }

    function trapFocus(drawer, event) {
        if (event.key !== 'Tab') return;
        var focusable = drawer.querySelectorAll(
            'button:not([disabled]), [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
        );
        if (!focusable.length) return;
        var first = focusable[0];
        var last = focusable[focusable.length - 1];
        if (event.shiftKey) {
            if (document.activeElement === first || !drawer.contains(document.activeElement)) {
                event.preventDefault();
                last.focus();
            }
        } else if (document.activeElement === last) {
            event.preventDefault();
            first.focus();
        }
    }

    function openChainDrawer(id) {
        var drawer = getDrawer();
        if (!drawer || !id) return;
        clearError(drawer);
        showDrawer(drawer);

        // Update URL deep-link
        try {
            var url = new URL(window.location.href);
            url.searchParams.set(QS_KEY, id);
            window.history.replaceState({}, '', url.toString());
        } catch (e) {}

        fetch('/admin/api/chain/' + encodeURIComponent(id), {
            credentials: 'same-origin',
            headers: { 'Accept': 'application/json' }
        })
            .then(function (resp) {
                if (resp.status === 404) throw new Error('No chain found for ' + id);
                if (!resp.ok) throw new Error('Failed (' + resp.status + ')');
                return resp.json();
            })
            .then(function (env) {
                renderHeader(drawer, env);
                renderStepper(drawer, env);
                renderEvents(drawer, env);
                renderRequests(drawer, env);
                renderTranscript(drawer, env);
                renderRaw(drawer, env);
            })
            .catch(function (err) {
                renderError(drawer, err.message || String(err));
            });
    }

    function bindGlobalHandlers() {
        document.addEventListener('click', function (e) {
            // Close handlers
            var closeEl = e.target.closest && e.target.closest('[data-chain-close]');
            if (closeEl) {
                e.preventDefault();
                hideDrawer();
                return;
            }

            // Copy handlers
            var copyEl = e.target.closest && e.target.closest('[data-chain-copy]');
            if (copyEl) {
                e.preventDefault();
                var key = copyEl.getAttribute('data-chain-copy');
                var drawer = getDrawer();
                if (drawer) {
                    var src = drawer.querySelector('[data-chain-' + key + ']');
                    if (src) copyToClipboard(src.dataset.value || src.textContent || '');
                }
                return;
            }

            // Open trigger
            var trigger = e.target.closest && e.target.closest('[data-chain-id]');
            if (trigger) {
                var id = trigger.getAttribute('data-chain-id');
                if (!id) return;
                e.preventDefault();
                openChainDrawer(id);
            }
        });

        document.addEventListener('keydown', function (e) {
            var drawer = getDrawer();
            if (!drawer || drawer.hidden) return;
            if (e.key === 'Escape') {
                e.preventDefault();
                hideDrawer();
                return;
            }
            trapFocus(drawer, e);
        });
    }

    function checkDeepLink() {
        try {
            var url = new URL(window.location.href);
            var id = url.searchParams.get(QS_KEY);
            if (id) openChainDrawer(id);
        } catch (e) {}
    }

    function init() {
        bindGlobalHandlers();
        checkDeepLink();
    }

    var ns = global.SystempromptAdmin = global.SystempromptAdmin || {};
    ns.chainDrawer = {
        open: openChainDrawer,
        close: hideDrawer
    };

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})(window);

(function () {
    'use strict';

    function applyIntensity(root) {
        var max = parseInt(root.getAttribute('data-max-cell') || '0', 10);
        if (!Number.isFinite(max) || max <= 0) return;
        var cells = root.querySelectorAll('.heatmap__cell[data-count]');
        for (var i = 0; i < cells.length; i++) {
            var cell = cells[i];
            var n = parseInt(cell.getAttribute('data-count') || '0', 10);
            if (!Number.isFinite(n) || n <= 0) continue;
            // Map count→intensity using a sqrt curve so a single hit is still
            // visible while extreme cells don't saturate everything else.
            var intensity = Math.round(Math.sqrt(n / max) * 70);
            cell.style.setProperty('--intensity', String(intensity));
        }
    }

    function init() {
        var roots = document.querySelectorAll('.heatmap[data-max-cell]');
        for (var i = 0; i < roots.length; i++) {
            applyIntensity(roots[i]);
        }
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();

(function () {
    'use strict';

    function applyHistogram(root) {
        var max = parseInt(root.getAttribute('data-histogram-max') || '0', 10);
        if (!Number.isFinite(max) || max <= 0) max = 1;
        var bars = root.querySelectorAll('.latency-histogram__bar');
        for (var i = 0; i < bars.length; i++) {
            var bar = bars[i];
            var c = parseInt(bar.getAttribute('data-count') || '0', 10);
            var ratio = (Number.isFinite(c) && c > 0) ? c / max : 0;
            bar.style.setProperty('--ratio', ratio.toFixed(4));
        }
    }

    function applyCostSpark(root) {
        var max = parseInt(root.getAttribute('data-cost-max') || '0', 10);
        if (!Number.isFinite(max) || max <= 0) max = 1;
        var bars = root.querySelectorAll('.cost-spark__bar');
        for (var i = 0; i < bars.length; i++) {
            var bar = bars[i];
            var c = parseInt(bar.getAttribute('data-cost') || '0', 10);
            var pct = (Number.isFinite(c) && c > 0) ? (c / max) * 100 : 0;
            bar.style.setProperty('--cost', pct.toFixed(2));
        }
    }

    function init() {
        var hists = document.querySelectorAll('.latency-histogram[data-histogram-max]');
        for (var i = 0; i < hists.length; i++) applyHistogram(hists[i]);
        var sparks = document.querySelectorAll('.cost-spark[data-cost-max]');
        for (var j = 0; j < sparks.length; j++) applyCostSpark(sparks[j]);
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();

(function(app) {
    'use strict';

    const openCreatePanel = () => {
        const overlay = document.getElementById('create-user-overlay');
        const panel = document.getElementById('create-user-panel');
        if (!overlay || !panel) return;
        overlay.classList.add('open');
        panel.classList.add('open');
        const first = panel.querySelector('input');
        if (first) setTimeout(() => { first.focus(); }, 350);
    };
    const closeCreatePanel = () => {
        const overlay = document.getElementById('create-user-overlay');
        const panel = document.getElementById('create-user-panel');
        if (panel) panel.classList.remove('open');
        if (overlay) overlay.classList.remove('open');
    };
    const resetForm = () => {
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
    };
    const bindCreatePanelEvents = (refreshFn) => {
        app.events.on('click', '#create-user-overlay', () => {
            closeCreatePanel();
        });

        app.events.on('click', '#create-user-panel .panel-close', () => {
            closeCreatePanel();
        });

        app.events.on('click', '#create-user-panel [data-action="cancel"]', () => {
            closeCreatePanel();
        });

        app.events.on('click', '#create-user-panel [data-action="save"]', async () => {
            const userId = document.getElementById('new-user-id').value.trim();
            const displayName = document.getElementById('new-user-name').value.trim();
            const email = document.getElementById('new-user-email').value.trim();
            const deptVal = document.getElementById('new-user-dept').value;
            const roleBoxes = document.querySelectorAll('#create-user-panel input[name="roles"]:checked');
            const roles = Array.from(roleBoxes).map((cb) => cb.value);
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
        });
    };
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

    const closeAllPopups = () => {
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
    };

    const getOrCreatePortal = () => {
        let portal = document.getElementById('user-actions-popup');
        if (!portal) {
            portal = document.createElement('div');
            portal.id = 'user-actions-popup';
            portal.className = 'actions-popup';
            portal.setAttribute('role', 'menu');
            document.body.append(portal);
        }
        return portal;
    };

    const positionPopup = (portal, trigger) => {
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
    };

    app.usersInteractions = () => {
        app.events.on('click', '.btn-actions-trigger', (e, trigger) => {
            const userId = trigger.dataset.userId;
            const portal = getOrCreatePortal();
            const isOpen = portal.classList.contains('open') && activePopupId === userId;
            closeAllPopups();
            if (isOpen) return;

            const row = trigger.closest('tr');
            const isActive = row && row.querySelector('.badge-green') !== null;
            const toggleLabel = isActive ? 'Deactivate' : 'Activate';
            const toggleClass = isActive ? ' actions-popup-item--danger' : '';

            portal.replaceChildren();

            const editBtn = document.createElement('button');
            editBtn.className = 'actions-popup-item';
            editBtn.setAttribute('data-action', 'edit');
            editBtn.setAttribute('data-user-id', userId);
            const editIcon = document.createElement('span');
            editIcon.className = 'popup-icon';
            editIcon.textContent = '\u270E';
            editBtn.append(editIcon, document.createTextNode(' Edit User'));

            const separator = document.createElement('div');
            separator.className = 'actions-popup-separator';

            const toggleBtn = document.createElement('button');
            toggleBtn.className = 'actions-popup-item' + toggleClass;
            toggleBtn.setAttribute('data-action', 'toggle');
            toggleBtn.setAttribute('data-user-id', userId);
            toggleBtn.setAttribute('data-is-active', String(isActive));
            const toggleIcon = document.createElement('span');
            toggleIcon.className = 'popup-icon';
            toggleIcon.textContent = isActive ? '\u2716' : '\u2714';
            toggleBtn.append(toggleIcon, document.createTextNode(' ' + toggleLabel));

            portal.append(editBtn, separator, toggleBtn);

            activePopupId = userId;
            portal.classList.add('open');
            trigger.classList.add('active');
            trigger.setAttribute('aria-expanded', 'true');
            positionPopup(portal, trigger);

            portal.querySelectorAll('.actions-popup-item').forEach((item) => {
                item.addEventListener('click', (ev) => {
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
                                async () => {
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
                            }).then(() => {
                                app.Toast.show('User activated', 'success');
                                window.location.reload();
                            }).catch((err) => {
                                app.Toast.show(err.message || 'Failed to activate user', 'error');
                            });
                        }
                    }
                });
            });
        });

        app.events.on('click', '*', (e) => {
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

(function(app) {
    'use strict';

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

    const groupByCategory = (fileList) => {
        const groups = {};
        fileList.forEach((f) => {
            const cat = f.category || 'config';
            if (!groups[cat]) groups[cat] = [];
            groups[cat].push(f);
        });
        return groups;
    };

    const buildFileList = () => {
        const frag = document.createDocumentFragment();
        if (!files.length) {
            const empty = document.createElement('div');
            empty.className = 'empty-state';
            empty.style.cssText = 'padding:var(--sp-space-6)';
            const p1 = document.createElement('p');
            p1.textContent = 'No files found for this skill.';
            const p2 = document.createElement('p');
            p2.style.cssText = 'font-size:var(--sp-text-sm);color:var(--sp-text-tertiary);margin-top:var(--sp-space-2)';
            p2.textContent = 'Click "Sync Files" to scan the filesystem.';
            empty.append(p1, p2);
            frag.append(empty);
            return frag;
        }
        const groups = groupByCategory(files);
        categoryOrder.forEach((cat) => {
            const group = groups[cat];
            if (!group || !group.length) return;
            const wrapper = document.createElement('div');
            wrapper.style.cssText = 'margin-bottom:var(--sp-space-3)';
            const catDiv = document.createElement('div');
            catDiv.className = 'skill-file-category';
            catDiv.textContent = (categoryLabels[cat] || cat) + ' (' + group.length + ')';
            wrapper.append(catDiv);
            group.forEach((f) => {
                const isSelected = selectedFile && selectedFile.id === f.id;
                const item = document.createElement('div');
                item.className = 'skill-file-item' + (isSelected ? ' selected' : '');
                item.setAttribute('data-file-id', f.id);
                const nameSpan = document.createElement('span');
                nameSpan.className = 'skill-file-name';
                nameSpan.textContent = f.file_path;
                item.append(nameSpan);
                if (f.language) {
                    const langSpan = document.createElement('span');
                    langSpan.className = 'skill-file-lang';
                    langSpan.textContent = f.language;
                    item.append(langSpan);
                }
                wrapper.append(item);
            });
            frag.append(wrapper);
        });
        return frag;
    };

    const validateContent = (content, lang) => {
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
    };

    const checkBrackets = (content, pairs) => {
        const stack = [];
        const closeMap = {};
        const openSet = {};
        pairs.forEach((p) => { closeMap[p[1]] = p[0]; openSet[p[0]] = p[1]; });
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
    };

    const buildEditor = () => {
        if (!selectedFile) {
            const placeholder = document.createElement('div');
            placeholder.style.cssText = 'display:flex;align-items:center;justify-content:center;height:100%;color:var(--sp-text-tertiary);font-size:var(--sp-text-sm)';
            placeholder.textContent = 'Select a file to view its contents';
            return placeholder;
        }
        const wrapper = document.createElement('div');
        wrapper.style.cssText = 'display:flex;flex-direction:column;height:100%';

        const header = document.createElement('div');
        header.style.cssText = 'display:flex;align-items:center;gap:var(--sp-space-2);padding:var(--sp-space-2) var(--sp-space-3);border-bottom:1px solid var(--sp-border-subtle);flex-shrink:0';

        const pathSpan = document.createElement('span');
        pathSpan.style.cssText = 'font-family:monospace;font-size:var(--sp-text-sm);font-weight:600';
        pathSpan.textContent = selectedFile.file_path;

        const langBadge = document.createElement('span');
        langBadge.className = 'badge badge-blue';
        langBadge.style.cssText = 'font-size:var(--sp-text-xs)';
        langBadge.textContent = selectedFile.language || 'text';

        header.append(pathSpan, langBadge);

        if (selectedFile.executable) {
            const execBadge = document.createElement('span');
            execBadge.className = 'badge badge-green';
            execBadge.style.cssText = 'font-size:var(--sp-text-xs)';
            execBadge.textContent = 'executable';
            header.append(execBadge);
        }

        const sizeSpan = document.createElement('span');
        sizeSpan.style.cssText = 'margin-left:auto;font-size:var(--sp-text-xs);color:var(--sp-text-tertiary)';
        sizeSpan.textContent = selectedFile.size_bytes + ' bytes';
        header.append(sizeSpan);

        const textarea = document.createElement('textarea');
        textarea.id = 'skill-file-editor';
        textarea.style.cssText = 'flex:1;width:100%;border:none;padding:var(--sp-space-3);font-family:monospace;font-size:var(--sp-text-sm);line-height:1.5;resize:none;background:var(--sp-bg-surface);color:var(--sp-text-primary);outline:none;box-sizing:border-box';
        textarea.value = selectedFile.content || '';

        const footer = document.createElement('div');
        footer.style.cssText = 'display:flex;align-items:center;padding:var(--sp-space-2) var(--sp-space-3);border-top:1px solid var(--sp-border-subtle);flex-shrink:0';

        const validationSpan = document.createElement('span');
        validationSpan.id = 'skill-file-validation';
        validationSpan.style.cssText = 'font-size:var(--sp-text-xs);flex:1';

        const saveBtn = document.createElement('button');
        saveBtn.className = 'btn btn-primary btn-sm';
        saveBtn.id = 'skill-file-save';
        saveBtn.style.cssText = 'font-size:var(--sp-text-xs)';
        saveBtn.textContent = 'Save';

        footer.append(validationSpan, saveBtn);
        wrapper.append(header, textarea, footer);
        return wrapper;
    };

    const buildModal = () => {
        const outer = document.createElement('div');
        outer.style.cssText = 'display:flex;flex-direction:column;height:100%';

        const topBar = document.createElement('div');
        topBar.style.cssText = 'display:flex;align-items:center;padding:var(--sp-space-4);border-bottom:1px solid var(--sp-border-subtle);flex-shrink:0';

        const heading = document.createElement('h2');
        heading.style.cssText = 'margin:0;font-size:var(--sp-text-lg);font-weight:600;color:var(--sp-text-primary)';
        heading.textContent = currentSkillName + ' - Files';

        const btnGroup = document.createElement('div');
        btnGroup.style.cssText = 'margin-left:auto;display:flex;gap:var(--sp-space-2)';

        const syncBtn = document.createElement('button');
        syncBtn.className = 'btn btn-secondary btn-sm';
        syncBtn.id = 'skill-files-sync';
        syncBtn.style.cssText = 'font-size:var(--sp-text-xs)';
        syncBtn.textContent = 'Sync Files';

        const closeBtn = document.createElement('button');
        closeBtn.className = 'btn btn-secondary btn-sm';
        closeBtn.id = 'skill-files-close';
        closeBtn.style.cssText = 'font-size:var(--sp-text-xs)';
        closeBtn.textContent = 'Close';

        btnGroup.append(syncBtn, closeBtn);
        topBar.append(heading, btnGroup);

        const body = document.createElement('div');
        body.style.cssText = 'display:flex;flex:1;min-height:0';

        const listPane = document.createElement('div');
        listPane.id = 'skill-files-list';
        listPane.style.cssText = 'width:280px;overflow-y:auto;border-right:1px solid var(--sp-border-subtle);padding:var(--sp-space-2) 0';
        listPane.append(buildFileList());

        const editorPane = document.createElement('div');
        editorPane.id = 'skill-files-editor';
        editorPane.style.cssText = 'flex:1;min-width:0;overflow:hidden';
        editorPane.append(buildEditor());

        body.append(listPane, editorPane);
        outer.append(topBar, body);
        return outer;
    };

    const updatePanel = () => {
        const panel = overlay && overlay.querySelector('.skill-files-panel');
        if (panel) panel.replaceChildren(buildModal());
        bindEvents();
    };

    const runValidation = () => {
        if (!overlay || !selectedFile) return;
        const editor = overlay.querySelector('#skill-file-editor');
        const badge = overlay.querySelector('#skill-file-validation');
        if (!editor || !badge) return;
        const err = validateContent(editor.value, selectedFile.language);
        if (err) {
            badge.textContent = err;
            badge.style.color = 'var(--sp-danger)';
        } else {
            badge.textContent = '';
        }
    };

    const bindEditorValidation = () => {
        if (!overlay) return;
        const editor = overlay.querySelector('#skill-file-editor');
        if (editor) {
            editor.addEventListener('input', runValidation);
            runValidation();
        }
    };

    const handleFileClick = (e) => {
        const item = e.currentTarget;
        const fileId = item.getAttribute('data-file-id');
        selectedFile = files.find((f) => f.id === fileId) || null;
        const listEl = overlay.querySelector('#skill-files-list');
        const editorEl = overlay.querySelector('#skill-files-editor');
        if (listEl) listEl.replaceChildren(buildFileList());
        if (editorEl) editorEl.replaceChildren(buildEditor());
        bindFileItems();
        const newSaveBtn = overlay.querySelector('#skill-file-save');
        if (newSaveBtn) newSaveBtn.addEventListener('click', handleSave);
        bindEditorValidation();
    };

    const bindFileItems = () => {
        if (!overlay) return;
        const fileItems = overlay.querySelectorAll('.skill-file-item');
        fileItems.forEach((item) => {
            item.addEventListener('click', handleFileClick);
        });
    };

    const bindEvents = () => {
        if (!overlay) return;

        const closeBtn = overlay.querySelector('#skill-files-close');
        if (closeBtn) closeBtn.addEventListener('click', close);

        const syncBtn = overlay.querySelector('#skill-files-sync');
        if (syncBtn) syncBtn.addEventListener('click', handleSync);

        const saveBtn = overlay.querySelector('#skill-file-save');
        if (saveBtn) saveBtn.addEventListener('click', handleSave);

        bindFileItems();
        bindEditorValidation();
    };

    const handleSync = async () => {
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
    };

    const handleSave = async () => {
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
    };

    const loadFiles = async () => {
        try {
            files = await app.api('/skills/' + encodeURIComponent(currentSkillId) + '/files');
            if (!Array.isArray(files)) files = [];
        } catch (err) {
            files = [];
            app.Toast.show(err.message || 'Failed to load files', 'error');
        }
    };

    const close = () => {
        if (overlay) {
            overlay.remove();
            overlay = null;
        }
        currentSkillId = null;
        currentSkillName = '';
        files = [];
        selectedFile = null;
    };

    const open = async (skillId, skillName) => {
        close();
        currentSkillId = skillId;
        currentSkillName = skillName || skillId;

        overlay = document.createElement('div');
        overlay.className = 'confirm-overlay';
        overlay.style.cssText = 'display:flex;align-items:center;justify-content:center;z-index:1000';

        const panel = document.createElement('div');
        panel.className = 'skill-files-panel';
        panel.style.cssText = 'background:var(--sp-bg-surface);border-radius:var(--sp-radius-lg);width:90vw;max-width:1100px;height:80vh;overflow:hidden;box-shadow:var(--sp-shadow-lg);display:flex;flex-direction:column';

        const loadingDiv = document.createElement('div');
        loadingDiv.style.cssText = 'display:flex;align-items:center;justify-content:center;height:100%;color:var(--sp-text-tertiary)';
        loadingDiv.textContent = 'Loading files...';

        panel.append(loadingDiv);
        overlay.append(panel);
        document.body.append(overlay);

        overlay.addEventListener('click', (e) => {
            if (e.target === overlay) close();
        });

        await loadFiles();
        updatePanel();
    };

    app.skillFiles = {
        open: open,
        close: close
    };
})(window.AdminApp);

(function(app) {
    'use strict';

    let overlay = null;
    let currentPluginId = null;
    let currentPluginName = '';
    let envVars = [];
    let varDefs = [];

    function mergeDefsWithValues(defs, stored) {
        const merged = [];
        const storedMap = {};
        stored.forEach((v) => { storedMap[v.var_name] = v; });

        defs.forEach((def) => {
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

        Object.keys(storedMap).forEach((key) => {
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

    function buildVarList(vars) {
        const frag = document.createDocumentFragment();
        if (!vars.length) {
            const empty = document.createElement('div');
            empty.className = 'empty-state';
            empty.style.cssText = 'padding:var(--sp-space-6)';
            const p = document.createElement('p');
            p.textContent = 'No environment variables defined for this plugin.';
            empty.append(p);
            frag.append(empty);
            return frag;
        }
        vars.forEach((v, i) => {
            const group = document.createElement('div');
            group.className = 'form-group';

            const label = document.createElement('label');
            label.textContent = v.name;
            if (v.required) {
                const reqBadge = document.createElement('span');
                reqBadge.className = 'badge badge-red';
                reqBadge.textContent = 'required';
                label.append(document.createTextNode(' '), reqBadge);
            }
            if (v.secret) {
                const secBadge = document.createElement('span');
                secBadge.className = 'badge badge-gray';
                secBadge.textContent = 'secret';
                label.append(document.createTextNode(' '), secBadge);
            }
            group.append(label);

            if (v.description) {
                const desc = document.createElement('p');
                desc.style.cssText = 'margin:0 0 var(--sp-space-1);font-size:var(--sp-text-xs);color:var(--sp-text-tertiary)';
                desc.textContent = v.description;
                group.append(desc);
            }

            const input = document.createElement('input');
            input.type = v.secret ? 'password' : 'text';
            input.className = 'plugin-env-input';
            input.setAttribute('data-var-index', i);
            input.setAttribute('data-var-name', v.name);
            input.setAttribute('data-is-secret', v.secret ? '1' : '0');
            input.value = v.value;
            if (v.example) input.placeholder = v.example;
            group.append(input);

            frag.append(group);
        });
        return frag;
    }

    function buildModal(vars) {
        const frag = document.createDocumentFragment();

        const heading = document.createElement('h3');
        heading.style.cssText = 'margin:0 0 var(--sp-space-4)';
        heading.textContent = currentPluginName + ' \u2014 Environment Variables';

        const scrollArea = document.createElement('div');
        scrollArea.style.cssText = 'max-height:60vh;overflow-y:auto';
        scrollArea.append(buildVarList(vars));

        const actions = document.createElement('div');
        actions.className = 'form-actions';
        actions.style.cssText = 'display:flex;gap:var(--sp-space-3);justify-content:flex-end;margin-top:var(--sp-space-4)';

        const closeBtn = document.createElement('button');
        closeBtn.className = 'btn btn-secondary';
        closeBtn.id = 'plugin-env-close';
        closeBtn.textContent = 'Close';

        const saveBtn = document.createElement('button');
        saveBtn.className = 'btn btn-primary';
        saveBtn.id = 'plugin-env-save';
        saveBtn.textContent = 'Save';

        actions.append(closeBtn, saveBtn);
        frag.append(heading, scrollArea, actions);
        return frag;
    }

    function updatePanel(vars) {
        const panel = overlay && overlay.querySelector('.confirm-dialog');
        if (panel) {
            panel.replaceChildren(buildModal(vars));
        }
        bindEvents(vars);
    }

    function bindEvents(vars) {
        if (!overlay) return;

        const closeBtn = overlay.querySelector('#plugin-env-close');
        if (closeBtn) closeBtn.addEventListener('click', close);

        const saveBtn = overlay.querySelector('#plugin-env-save');
        if (saveBtn) saveBtn.addEventListener('click', () => { handleSave(vars); });
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
            inputs.forEach((input) => {
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
                saveBtn.style.background = 'var(--sp-success)';
                saveBtn.style.borderColor = 'var(--sp-success)';
            }
            app.Toast.show('Environment variables saved', 'success');
            setTimeout(() => { close(); }, 600);
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

        const dialog = document.createElement('div');
        dialog.className = 'confirm-dialog';
        dialog.style.cssText = 'width:560px;max-width:90vw';

        const loadingDiv = document.createElement('div');
        loadingDiv.style.cssText = 'display:flex;align-items:center;justify-content:center;padding:var(--sp-space-6);color:var(--sp-text-tertiary)';
        loadingDiv.textContent = 'Loading...';

        dialog.append(loadingDiv);
        overlay.append(dialog);
        document.body.append(overlay);

        overlay.addEventListener('click', (e) => {
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
        btns.forEach((btn) => {
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

        const dialog = document.createElement('div');
        dialog.className = 'confirm-dialog';

        const heading = document.createElement('h3');
        heading.style.cssText = 'margin:0 0 var(--sp-space-3)';
        heading.textContent = 'Delete Plugin?';

        const p1 = document.createElement('p');
        p1.style.cssText = 'margin:0 0 var(--sp-space-2);color:var(--sp-text-secondary);font-size:var(--sp-text-sm)';
        const p1Text1 = document.createTextNode('You are about to delete ');
        const p1Strong = document.createElement('strong');
        p1Strong.textContent = pluginId;
        const p1Text2 = document.createTextNode('.');
        p1.append(p1Text1, p1Strong, p1Text2);

        const p2 = document.createElement('p');
        p2.style.cssText = 'margin:0 0 var(--sp-space-5);color:var(--sp-text-secondary);font-size:var(--sp-text-sm)';
        p2.textContent = 'This will remove the plugin directory and all its configuration. This action cannot be undone.';

        const btnRow = document.createElement('div');
        btnRow.style.cssText = 'display:flex;gap:var(--sp-space-3);justify-content:flex-end';

        const cancelBtn = document.createElement('button');
        cancelBtn.className = 'btn btn-secondary';
        cancelBtn.setAttribute('data-confirm-cancel', '');
        cancelBtn.textContent = 'Cancel';

        const deleteBtn = document.createElement('button');
        deleteBtn.className = 'btn btn-danger';
        deleteBtn.setAttribute('data-confirm-delete', pluginId);
        deleteBtn.textContent = 'Delete Plugin';

        btnRow.append(cancelBtn, deleteBtn);
        dialog.append(heading, p1, p2, btnRow);
        overlay.append(dialog);
        document.body.append(overlay);
        overlay.addEventListener('click', async (e) => {
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
            const bundle = items.find((b) => b.id === pluginId || b.plugin_id === pluginId);
            if (!bundle || !bundle.files) throw new Error('No files found in export');
            bundle.files.forEach((f) => {
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

        function createOverviewRow(label, valueContent) {
            const labelSpan = document.createElement('span');
            labelSpan.className = 'config-overview-label';
            labelSpan.textContent = label;
            const valueSpan = document.createElement('span');
            valueSpan.className = 'config-overview-value';
            if (typeof valueContent === 'string') {
                valueSpan.textContent = valueContent;
            } else {
                valueSpan.append(valueContent);
            }
            return [labelSpan, valueSpan];
        }

        const overviewSection = document.createElement('div');
        overviewSection.className = 'config-panel-section';
        const overviewH4 = document.createElement('h4');
        overviewH4.textContent = 'Overview';
        const overviewGrid = document.createElement('div');
        overviewGrid.className = 'config-overview-grid';

        const idCode = document.createElement('code');
        idCode.textContent = data.id;
        overviewGrid.append.apply(overviewGrid, createOverviewRow('ID', idCode));

        const statusBadge = document.createElement('span');
        statusBadge.className = data.enabled ? 'badge badge-green' : 'badge badge-gray';
        statusBadge.textContent = data.enabled ? 'Enabled' : 'Disabled';
        overviewGrid.append.apply(overviewGrid, createOverviewRow('Status', statusBadge));

        overviewGrid.append.apply(overviewGrid, createOverviewRow('Version', data.version || '\u2014'));
        overviewGrid.append.apply(overviewGrid, createOverviewRow('Category', data.category || '\u2014'));
        overviewGrid.append.apply(overviewGrid, createOverviewRow('Author', data.author_name || '\u2014'));
        overviewGrid.append.apply(overviewGrid, createOverviewRow('Description', data.description || '\u2014'));

        overviewSection.append(overviewH4, overviewGrid);

        const envSection = document.createElement('div');
        envSection.className = 'config-panel-section';
        const envH4 = document.createElement('h4');
        envH4.textContent = 'Environment';
        const envStatus = document.createElement('div');
        envStatus.id = 'panel-env-status';
        envStatus.textContent = 'Loading...';
        envSection.append(envH4, envStatus);

        const panelBody = document.getElementById('panel-body');
        panelBody.replaceChildren(overviewSection, envSection);

        const panelFooter = document.getElementById('panel-footer');
        panelFooter.replaceChildren();
        if (data.id !== 'custom') {
            const editLink = document.createElement('a');
            editLink.href = '/admin/org/plugins/edit/?id=' + encodeURIComponent(data.id);
            editLink.className = 'btn btn-primary';
            editLink.textContent = 'Edit Plugin';

            const envBtn = document.createElement('button');
            envBtn.className = 'btn btn-secondary';
            envBtn.setAttribute('data-open-env', data.id);
            envBtn.setAttribute('data-plugin-name', data.name);
            envBtn.textContent = 'Configure Env';

            panelFooter.append(editLink, document.createTextNode(' '), envBtn);
        }

        openPanel();

        if (data.id !== 'custom') {
            loadEnvStatus(data.id, document.getElementById('panel-env-status'));
        } else {
            const naDiv = document.createElement('div');
            naDiv.className = 'empty-state';
            const naP = document.createElement('p');
            naP.textContent = 'N/A';
            naDiv.append(naP);
            document.getElementById('panel-env-status').replaceChildren(naDiv);
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
                    const found = data.skills.find((s) => s.id === skillId);
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

        document.querySelectorAll('tr.detail-row.visible').forEach((r) => {
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
                detailRow.querySelectorAll('.detail-section').forEach((s) => {
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
        rows.forEach((row) => {
            const name = row.getAttribute('data-name') || '';
            const category = (row.getAttribute('data-category') || '').toLowerCase();
            const matchSearch = !searchVal || name.includes(searchVal);
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

    app.initPluginsConfig = () => {
        const bulkActions = app.OrgCommon ? app.OrgCommon.initBulkActions('#plugins-table', 'bulk-actions-btn') : null;

        const pluginRows = document.querySelectorAll('#plugins-table tr[data-entity-type="plugin"]');
        pluginRows.forEach((row) => {
            const pid = row.getAttribute('data-entity-id');
            if (!pid || pid === 'custom') return;
            updateGenerateButtons(pid);
            app.api('/plugins/' + encodeURIComponent(pid) + '/env').then((envData) => {
                pluginEnvValid[pid] = envData.valid !== false;
                updateGenerateButtons(pid);
            }).catch((err) => {
                pluginEnvValid[pid] = false;
                updateGenerateButtons(pid);
            });
        });

        app.shared.createDebouncedSearch(document, 'plugin-search', () => {
            applyFilters();
        });

        document.getElementById('category-filter').addEventListener('change', () => {
            applyFilters();
        });

        app.events.on('click', '[data-remove-from-plugin]', (e, btn) => {
            const itemId = btn.getAttribute('data-remove-from-plugin');
            const resourceType = btn.getAttribute('data-resource-type');
            const pluginId = btn.getAttribute('data-plugin-id');
            if (!pluginId || pluginId === 'custom') return;

            const detailEl = document.querySelector('[data-plugin-detail="' + pluginId + '"]');
            if (!detailEl) return;
            let data;
            try { data = JSON.parse(detailEl.textContent); } catch (ex) { return; }

            const apiField = resourceType === 'mcp_servers' ? 'mcp_servers' : resourceType;
            let currentIds;
            if (resourceType === 'skills') {
                currentIds = (data.skills || []).map((s) => s.id);
            } else if (resourceType === 'agents') {
                currentIds = (data.agents || []).map((a) => a.id);
            } else if (resourceType === 'mcp_servers') {
                currentIds = data.mcp_servers || [];
            } else if (resourceType === 'hooks') {
                currentIds = (data.hooks || []).map((h) => h.id);
            } else {
                return;
            }

            const updatedIds = currentIds.filter((id) => id !== itemId);
            const body = {};
            body[apiField] = updatedIds;

            btn.disabled = true;
            app.api('/plugins/' + encodeURIComponent(pluginId), {
                method: 'PUT',
                body: JSON.stringify(body)
            }).then(() => {
                const row = btn.closest('tr');
                if (row) row.remove();
                const countEl = document.querySelector('[data-count="' + resourceType + '"][data-for-plugin="' + pluginId + '"]');
                if (countEl) countEl.textContent = updatedIds.length;
                if (resourceType === 'skills') {
                    data.skills = data.skills.filter((s) => s.id !== itemId);
                } else if (resourceType === 'agents') {
                    data.agents = data.agents.filter((a) => a.id !== itemId);
                } else if (resourceType === 'mcp_servers') {
                    data.mcp_servers = updatedIds;
                } else if (resourceType === 'hooks') {
                    data.hooks = data.hooks.filter((h) => h.id !== itemId);
                }
                detailEl.textContent = JSON.stringify(data);
                app.Toast.show('Removed from plugin', 'success');
            }).catch((err) => {
                btn.disabled = false;
                app.Toast.show(err.message || 'Failed to remove', 'error');
            });
        });

        app.events.on('click', '[data-add-to-plugin]', (e, btn) => {
            const resourceType = btn.getAttribute('data-add-to-plugin');
            const pluginId = btn.getAttribute('data-plugin-id');
            if (!pluginId || pluginId === 'custom') return;

            const detailEl = document.querySelector('[data-plugin-detail="' + pluginId + '"]');
            if (!detailEl) return;
            let data;
            try { data = JSON.parse(detailEl.textContent); } catch (ex) { return; }

            const apiMap = { skills: '/skills', agents: '/agents', mcp_servers: '/mcp-servers', hooks: '/hooks' };
            const apiPath = apiMap[resourceType];
            if (!apiPath) return;

            let currentIds;
            if (resourceType === 'skills') {
                currentIds = (data.skills || []).map((s) => s.id);
            } else if (resourceType === 'agents') {
                currentIds = (data.agents || []).map((a) => a.id);
            } else if (resourceType === 'mcp_servers') {
                currentIds = data.mcp_servers || [];
            } else if (resourceType === 'hooks') {
                currentIds = (data.hooks || []).map((h) => h.id);
            }
            const currentSet = {};
            currentIds.forEach((id) => { currentSet[id] = true; });

            btn.disabled = true;
            btn.textContent = 'Loading...';
            app.api(apiPath).then((allItems) => {
                btn.disabled = false;
                btn.textContent = '+ Add ' + resourceType.charAt(0).toUpperCase() + resourceType.slice(1).replace('_', ' ');
                const items = Array.isArray(allItems) ? allItems : (allItems.items || allItems.data || []);
                const available = items.filter((item) => {
                    const id = typeof item === 'string' ? item : (item.id || item.skill_id || item.agent_id);
                    return id && !currentSet[id];
                });

                if (available.length === 0) {
                    app.Toast.show('No additional ' + resourceType.replace('_', ' ') + ' available', 'info');
                    return;
                }

                const overlay = document.createElement('div');
                overlay.className = 'confirm-overlay';

                const dialog = document.createElement('div');
                dialog.className = 'confirm-dialog';

                const heading = document.createElement('h3');
                heading.style.cssText = 'margin:0 0 var(--sp-space-3)';
                heading.textContent = 'Add ' + resourceType.replace('_', ' ');

                const checklist = document.createElement('div');
                checklist.className = 'add-checklist';
                available.forEach((item) => {
                    const id = typeof item === 'string' ? item : (item.id || item.skill_id || item.agent_id);
                    const name = typeof item === 'string' ? item : (item.name || item.id || item.skill_id);
                    const label = document.createElement('label');
                    const cb = document.createElement('input');
                    cb.type = 'checkbox';
                    cb.value = id;
                    label.append(cb, document.createTextNode(' ' + name));
                    checklist.append(label);
                });

                const btnRow = document.createElement('div');
                btnRow.style.cssText = 'display:flex;gap:var(--sp-space-3);justify-content:flex-end;margin-top:var(--sp-space-3)';
                const addCancelBtn = document.createElement('button');
                addCancelBtn.className = 'btn btn-secondary';
                addCancelBtn.setAttribute('data-add-cancel', '');
                addCancelBtn.textContent = 'Cancel';
                const addConfirmBtn = document.createElement('button');
                addConfirmBtn.className = 'btn btn-primary';
                addConfirmBtn.setAttribute('data-add-confirm', '');
                addConfirmBtn.textContent = 'Add Selected';
                btnRow.append(addCancelBtn, addConfirmBtn);

                dialog.append(heading, checklist, btnRow);
                overlay.append(dialog);
                document.body.append(overlay);

                overlay.addEventListener('click', (ev) => {
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
                    checked.forEach((cb) => { newIds.push(cb.value); });
                    const mergedIds = currentIds.concat(newIds);

                    const body = {};
                    const apiField = resourceType === 'mcp_servers' ? 'mcp_servers' : resourceType;
                    body[apiField] = mergedIds;

                    confirmBtn.disabled = true;
                    confirmBtn.textContent = 'Saving...';
                    app.api('/plugins/' + encodeURIComponent(pluginId), {
                        method: 'PUT',
                        body: JSON.stringify(body)
                    }).then(() => {
                        overlay.remove();
                        app.Toast.show('Added to plugin', 'success');
                        window.location.reload();
                    }).catch((err) => {
                        confirmBtn.disabled = false;
                        confirmBtn.textContent = 'Add Selected';
                        app.Toast.show(err.message || 'Failed to add', 'error');
                    });
                });
            }).catch((err) => {
                btn.disabled = false;
                btn.textContent = '+ Add ' + resourceType.charAt(0).toUpperCase() + resourceType.slice(1).replace('_', ' ');
                app.Toast.show(err.message || 'Failed to load available items', 'error');
            });
        });

        app.events.on('click', '[data-expand-section]', (e, expandBadge) => {
            const section = expandBadge.getAttribute('data-expand-section');
            const pluginId = expandBadge.getAttribute('data-plugin-id');
            toggleDetailRow(pluginId, section);
        });

        app.events.on('click', '[data-browse-skill]', (e, el) => {
            e.preventDefault();
            const skillId = el.getAttribute('data-browse-skill');
            const skillName = el.getAttribute('data-skill-name') || skillId;
            if (app.skillFiles) app.skillFiles.open(skillId, skillName);
        });

        app.events.on('click', '[data-toggle-json]', (e, jsonToggle) => {
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

        app.events.on('click', 'tr.clickable-row', (e, row) => {
            if (e.target.closest('[data-no-row-click]') || e.target.closest('[data-action="toggle"]') || e.target.closest('.actions-menu') || e.target.closest('.btn') || e.target.closest('a') || e.target.closest('input')) return;
            const entityId = row.getAttribute('data-entity-id');
            toggleDetailRow(entityId);
        });

        app.events.on('click', '[data-open-env]', (e, envBtn) => {
            const envPluginId = envBtn.getAttribute('data-open-env');
            const pluginName = envBtn.getAttribute('data-plugin-name') || envPluginId;
            if (app.pluginEnv) app.pluginEnv.open(envPluginId, pluginName);
        });

        app.events.on('click', '[data-generate-plugin]', (e, generateBtn) => {
            const platform = generateBtn.getAttribute('data-platform') || 'unix';
            handleExport(generateBtn.getAttribute('data-generate-plugin'), generateBtn, platform);
        });

        app.events.on('click', '[data-delete-plugin]', (e, deletePluginBtn) => {
            app.shared.closeAllMenus();
            showDeleteConfirm(deletePluginBtn.getAttribute('data-delete-plugin'));
        });

        document.getElementById('panel-close').addEventListener('click', closePanel);
        document.getElementById('config-overlay').addEventListener('click', closePanel);

        app.events.on('click', '#export-marketplace-btn', async (e, btn) => {
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
                a.href = url; a.download = 'marketplace.zip'; a.click();
                URL.revokeObjectURL(url);
                app.Toast.show('Marketplace zip generated', 'success');
            } catch (err) {
                app.Toast.show(err.message || 'Export failed', 'error');
            } finally {
                btn.disabled = false;
                btn.textContent = 'Export';
            }
        });

        window.addEventListener('env-saved', (e) => {
            const pid = e.detail && e.detail.pluginId;
            if (!pid) return;
            app.api('/plugins/' + encodeURIComponent(pid) + '/env').then((envData) => {
                pluginEnvValid[pid] = envData.valid !== false;
                updateGenerateButtons(pid);
            }).catch((err) => {
                pluginEnvValid[pid] = false;
                updateGenerateButtons(pid);
            });
        });
    };

    app.initPluginsList = app.initPluginsConfig;

    function buildEnvDefItem(def, storedMap) {
        const s = storedMap[def.name];
        const hasValue = s && s.var_value && s.var_value !== '';

        const item = document.createElement('div');
        item.className = 'detail-item';

        const info = document.createElement('div');
        info.className = 'detail-item-info';

        const nameRow = document.createElement('div');
        nameRow.className = 'detail-item-name';

        const code = document.createElement('code');
        code.style.cssText = 'background:var(--sp-bg-surface-raised);padding:1px 6px;border-radius:var(--sp-radius-xs);font-size:var(--sp-text-sm)';
        code.textContent = def.name;
        nameRow.append(code, document.createTextNode(' '));

        const valBadge = document.createElement('span');
        valBadge.className = hasValue ? 'badge badge-green' : 'badge badge-red';
        valBadge.textContent = hasValue ? 'configured' : 'not set';
        nameRow.append(valBadge);

        if (def.required !== false && !hasValue) {
            const reqBadge = document.createElement('span');
            reqBadge.className = 'badge badge-yellow';
            reqBadge.textContent = 'required';
            nameRow.append(document.createTextNode(' '), reqBadge);
        }
        if (def.secret) {
            const secBadge = document.createElement('span');
            secBadge.className = 'badge badge-gray';
            secBadge.textContent = 'secret';
            nameRow.append(document.createTextNode(' '), secBadge);
        }

        const descRow = document.createElement('div');
        descRow.className = 'detail-item-desc';
        descRow.style.cssText = 'font-size:var(--sp-text-sm);color:var(--sp-text-secondary);margin-top:var(--sp-space-1)';
        if (def.description) {
            descRow.textContent = def.description;
        }
        if (hasValue) {
            const maskedSpan = document.createElement('span');
            maskedSpan.style.cssText = 'font-family:monospace;color:var(--sp-text-tertiary)';
            maskedSpan.textContent = s.is_secret ? '--------' : s.var_value;
            descRow.append(document.createTextNode(' '), maskedSpan);
        }

        info.append(nameRow, descRow);
        item.append(info);
        return item;
    }

    function loadEnvStatus(pluginId, container) {
        const loadingDiv = document.createElement('div');
        loadingDiv.style.cssText = 'padding:var(--sp-space-4);color:var(--sp-text-tertiary);font-size:var(--sp-text-sm)';
        loadingDiv.textContent = 'Loading variables...';
        container.replaceChildren(loadingDiv);
        app.api('/plugins/' + encodeURIComponent(pluginId) + '/env').then((data) => {
            const defs = data.definitions || [];
            const stored = data.stored || [];
            if (!defs.length && !stored.length) {
                const emptyDiv = document.createElement('div');
                emptyDiv.className = 'empty-state';
                const emptyP = document.createElement('p');
                emptyP.textContent = 'No environment variables defined for this plugin.';
                emptyDiv.append(emptyP);
                container.replaceChildren(emptyDiv);
                return;
            }
            const storedMap = {};
            stored.forEach((v) => { storedMap[v.var_name] = v; });
            const frag = document.createDocumentFragment();
            defs.forEach((def) => {
                frag.append(buildEnvDefItem(def, storedMap));
            });
            const btnWrap = document.createElement('div');
            btnWrap.style.cssText = 'padding:var(--sp-space-3) 0';
            const configBtn = document.createElement('button');
            configBtn.className = 'btn btn-primary btn-sm';
            configBtn.setAttribute('data-open-env', pluginId);
            configBtn.setAttribute('data-plugin-name', pluginId);
            configBtn.textContent = 'Configure';
            btnWrap.append(configBtn);
            frag.append(btnWrap);
            container.replaceChildren(frag);
        }).catch(() => {
            const errDiv = document.createElement('div');
            errDiv.className = 'empty-state';
            const errP = document.createElement('p');
            errP.textContent = 'Failed to load environment variables.';
            errDiv.append(errP);
            container.replaceChildren(errDiv);
        });
    }
})(window.AdminApp);

(function(app) {
    'use strict';

    let plugins = [];

    function showVisibilityModal(pluginId) {
        const plugin = plugins.find((p) => p.id === pluginId);
        if (!plugin) return;
        const rules = plugin.visibility_rules || [];

        const overlay = document.createElement('div');
        overlay.className = 'confirm-overlay';
        overlay.id = 'visibility-modal';

        const dialog = document.createElement('div');
        dialog.className = 'confirm-dialog';
        dialog.style.maxWidth = '500px';

        const heading = document.createElement('h3');
        heading.style.margin = '0 0 var(--sp-space-3)';
        heading.textContent = 'Edit Visibility - ' + plugin.name;

        const rulesList = document.createElement('div');
        rulesList.id = 'visibility-rules-list';
        renderRulesListDOM(rulesList, rules);

        const addSection = document.createElement('div');
        addSection.style.cssText = 'margin-top:var(--sp-space-4);padding-top:var(--sp-space-3);border-top:1px solid var(--sp-border-primary)';

        const addLabel = document.createElement('strong');
        addLabel.style.fontSize = 'var(--sp-text-sm)';
        addLabel.textContent = 'Add Rule';

        const addRow = document.createElement('div');
        addRow.style.cssText = 'display:flex;gap:var(--sp-space-2);margin-top:var(--sp-space-2);flex-wrap:wrap';

        const ruleTypeSelect = document.createElement('select');
        ruleTypeSelect.id = 'vis-rule-type';
        ruleTypeSelect.className = 'btn btn-secondary';
        ruleTypeSelect.style.cssText = 'cursor:pointer;font-size:var(--sp-text-sm)';
        const opt1 = document.createElement('option');
        opt1.value = 'department';
        opt1.textContent = 'Department';
        const opt2 = document.createElement('option');
        opt2.value = 'user';
        opt2.textContent = 'User';
        ruleTypeSelect.append(opt1, opt2);

        const ruleValueInput = document.createElement('input');
        ruleValueInput.type = 'text';
        ruleValueInput.id = 'vis-rule-value';
        ruleValueInput.className = 'search-input';
        ruleValueInput.placeholder = 'Value...';
        ruleValueInput.style.cssText = 'flex:1;min-width:120px;font-size:var(--sp-text-sm)';

        const ruleAccessSelect = document.createElement('select');
        ruleAccessSelect.id = 'vis-rule-access';
        ruleAccessSelect.className = 'btn btn-secondary';
        ruleAccessSelect.style.cssText = 'cursor:pointer;font-size:var(--sp-text-sm)';
        const optAllow = document.createElement('option');
        optAllow.value = 'allow';
        optAllow.textContent = 'Allow';
        const optDeny = document.createElement('option');
        optDeny.value = 'deny';
        optDeny.textContent = 'Deny';
        ruleAccessSelect.append(optAllow, optDeny);

        const addRuleBtn = document.createElement('button');
        addRuleBtn.className = 'btn btn-secondary';
        addRuleBtn.id = 'vis-add-rule';
        addRuleBtn.style.fontSize = 'var(--sp-text-sm)';
        addRuleBtn.textContent = 'Add';

        addRow.append(ruleTypeSelect, ruleValueInput, ruleAccessSelect, addRuleBtn);
        addSection.append(addLabel, addRow);

        const btnRow = document.createElement('div');
        btnRow.style.cssText = 'display:flex;gap:var(--sp-space-3);justify-content:flex-end;margin-top:var(--sp-space-4)';

        const cancelBtn = document.createElement('button');
        cancelBtn.className = 'btn btn-secondary';
        cancelBtn.setAttribute('data-confirm-cancel', '');
        cancelBtn.textContent = 'Cancel';

        const saveBtn = document.createElement('button');
        saveBtn.className = 'btn btn-primary';
        saveBtn.id = 'vis-save';
        saveBtn.textContent = 'Save';

        btnRow.append(cancelBtn, saveBtn);
        dialog.append(heading, rulesList, addSection, btnRow);
        overlay.append(dialog);

        document.body.append(overlay);

        const modalRules = rules.slice();

        function refreshRulesList() {
            const container = overlay.querySelector('#visibility-rules-list');
            if (container) renderRulesListDOM(container, modalRules);
        }

        overlay.addEventListener('click', async (e) => {
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

    function renderRulesListDOM(container, rules) {
        container.replaceChildren();
        if (!rules.length) {
            const emptyP = document.createElement('p');
            emptyP.style.cssText = 'font-size:var(--sp-text-sm);color:var(--sp-text-tertiary)';
            emptyP.textContent = 'No rules configured';
            container.append(emptyP);
            return;
        }
        rules.forEach(function(rule, idx) {
            const row = document.createElement('div');
            row.style.cssText = 'display:flex;align-items:center;gap:var(--sp-space-2);padding:var(--sp-space-1) 0;font-size:var(--sp-text-sm)';
            const badge = document.createElement('span');
            badge.className = 'badge ' + (rule.access === 'allow' ? 'badge-yellow' : 'badge-red');
            badge.textContent = rule.rule_type + ': ' + rule.rule_value + ' (' + rule.access + ')';
            const removeBtn = document.createElement('button');
            removeBtn.className = 'btn btn-danger';
            removeBtn.style.cssText = 'font-size:var(--sp-text-xs);padding:2px 6px';
            removeBtn.setAttribute('data-remove-rule', idx);
            removeBtn.textContent = 'Remove';
            row.append(badge, removeBtn);
            container.append(row);
        });
    }

    app.initMarketplace = (selector, pluginsData) => {
        const root = document.querySelector(selector);
        if (!root) return;
        plugins = pluginsData || [];

        const searchInput = document.getElementById('mkt-search');
        if (searchInput) {
            let debounceTimer = null;
            searchInput.addEventListener('input', () => {
                clearTimeout(debounceTimer);
                debounceTimer = setTimeout(() => {
                    const q = searchInput.value.toLowerCase().trim();
                    const cards = root.querySelectorAll('.plugin-card[data-plugin-id]');
                    for (let i = 0; i < cards.length; i++) {
                        const name = cards[i].getAttribute('data-search-name') || '';
                        const desc = cards[i].getAttribute('data-search-desc') || '';
                        const cat = cards[i].getAttribute('data-search-cat') || '';
                        cards[i].style.display = (!q || name.includes(q) || desc.includes(q) || cat.includes(q)) ? '' : 'none';
                    }
                }, 200);
            });
        }

        const sortSelect = document.getElementById('mkt-sort');
        if (sortSelect) {
            sortSelect.addEventListener('change', () => {
                const url = new URL(window.location.href);
                url.searchParams.set('sort', sortSelect.value);
                window.location.href = url.toString();
            });
        }

        root.addEventListener('click', async (e) => {
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
                showVisibilityModal(visBtn.getAttribute('data-edit-visibility'));
                return;
            }

            const loadUsersBtn = e.target.closest('[data-load-users]');
            if (loadUsersBtn) {
                const pluginId = loadUsersBtn.getAttribute('data-load-users');
                loadUsersBtn.disabled = true;
                loadUsersBtn.textContent = 'Loading...';
                try {
                    const usersData = await app.api('/marketplace-plugins/' + encodeURIComponent(pluginId) + '/users');
                    const users = usersData.users || usersData || [];
                    const container = root.querySelector('[data-users-for="' + pluginId + '"]');
                    if (container) {
                        container.replaceChildren();
                        if (users.length === 0) {
                            const noUsers = document.createElement('div');
                            noUsers.style.cssText = 'margin-top:var(--sp-space-2);font-size:var(--sp-text-xs);color:var(--sp-text-tertiary)';
                            noUsers.textContent = 'No users found';
                            container.append(noUsers);
                        } else {
                            const userList = document.createElement('div');
                            userList.style.cssText = 'margin-top:var(--sp-space-2);display:flex;flex-direction:column;gap:var(--sp-space-1)';
                            users.forEach(function(u) {
                                const userRow = document.createElement('div');
                                userRow.style.cssText = 'display:flex;align-items:center;gap:var(--sp-space-2);font-size:var(--sp-text-xs);padding:var(--sp-space-1) 0;border-bottom:1px solid var(--sp-border-primary)';
                                const nameSpan = document.createElement('span');
                                nameSpan.style.cssText = 'font-weight:600;color:var(--sp-text-primary)';
                                nameSpan.textContent = u.display_name || 'Unknown';
                                userRow.append(nameSpan);
                                if (u.department) {
                                    const deptBadge = document.createElement('span');
                                    deptBadge.className = 'badge badge-blue';
                                    deptBadge.textContent = u.department;
                                    userRow.append(deptBadge);
                                }
                                const eventBadge = document.createElement('span');
                                eventBadge.className = 'badge badge-gray';
                                eventBadge.textContent = (u.event_count || 0) + ' events';
                                userRow.append(eventBadge);
                                if (u.last_used) {
                                    const dateSpan = document.createElement('span');
                                    dateSpan.style.color = 'var(--sp-text-tertiary)';
                                    dateSpan.textContent = new Date(u.last_used).toLocaleDateString();
                                    userRow.append(dateSpan);
                                }
                                userList.append(userRow);
                            });
                            container.append(userList);
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

    const versionDetails = {};
    const diffCache = {};
    let activeDiff = null;

    function marketplaceApi(userId, path) {
        const url = '/api/public/marketplace/' + encodeURIComponent(userId) + path;
        return fetch(url, { headers: { 'Content-Type': 'application/json' } })
            .then((resp) => {
                if (!resp.ok) return resp.text().then((t) => { throw new Error(t || resp.statusText); });
                return resp.json();
            });
    }

    function renderSkillRow(skill, versionId) {
        const hasBase = skill.base_skill_id && skill.base_skill_id !== 'null';

        const row = document.createElement('div');
        row.className = 'detail-item';

        const info = document.createElement('div');
        info.className = 'detail-item-info';

        const nameDiv = document.createElement('div');
        nameDiv.className = 'detail-item-name';
        nameDiv.append(document.createTextNode((skill.name || skill.skill_id) + ' '));

        const baseBadge = document.createElement('span');
        baseBadge.className = hasBase ? 'badge badge-yellow' : 'badge badge-gray';
        baseBadge.textContent = hasBase ? 'customized' : 'custom';
        nameDiv.append(baseBadge, document.createTextNode(' '));

        const enabledBadge = document.createElement('span');
        enabledBadge.className = skill.enabled === false ? 'badge badge-red' : 'badge badge-green';
        enabledBadge.textContent = skill.enabled === false ? 'disabled' : 'enabled';
        nameDiv.append(enabledBadge);

        const metaDiv = document.createElement('div');
        metaDiv.style.cssText = 'font-size:var(--sp-text-xs);color:var(--sp-text-tertiary);margin-top:2px';

        const codeEl = document.createElement('code');
        codeEl.style.cssText = 'background:var(--sp-bg-surface-raised);padding:1px 6px;border-radius:var(--sp-radius-xs)';
        codeEl.textContent = skill.skill_id;
        metaDiv.append(codeEl);

        if (skill.version) {
            const vSpan = document.createElement('span');
            vSpan.textContent = ' v' + skill.version;
            metaDiv.append(vSpan);
        }
        if (skill.description) {
            metaDiv.append(document.createTextNode(' \u2014 ' + app.shared.truncate(skill.description, 80)));
        }

        info.append(nameDiv, metaDiv);
        row.append(info);

        if (hasBase) {
            const isActive = activeDiff && activeDiff.versionId === versionId && activeDiff.skillId === skill.skill_id;
            const cmpBtn = document.createElement('button');
            cmpBtn.className = 'btn btn-secondary btn-sm';
            cmpBtn.setAttribute('data-compare-skill', skill.skill_id);
            cmpBtn.setAttribute('data-compare-version', versionId);
            cmpBtn.setAttribute('data-base-skill', skill.base_skill_id);
            cmpBtn.style.cssText = 'font-size:var(--sp-text-xs);padding:2px 8px;white-space:nowrap';
            if (isActive) cmpBtn.disabled = true;
            cmpBtn.textContent = isActive ? 'Viewing Diff' : 'Compare to Core';
            row.append(cmpBtn);
        }

        return row;
    }

    function renderDiffPanel(userSkill, coreSkill) {
        const userLines = (userSkill.content || '').split('\n');
        const coreLines = (coreSkill.content || '').split('\n');
        const maxLen = Math.max(userLines.length, coreLines.length);

        const panel = document.createElement('div');
        panel.className = 'diff-panel';

        const header = document.createElement('div');
        header.className = 'diff-panel-header';

        const h4 = document.createElement('h4');
        h4.style.cssText = 'margin:0;font-size:var(--sp-text-sm);font-weight:600';
        h4.textContent = 'Diff: ' + userSkill.skill_id;

        const legendDiv = document.createElement('div');
        legendDiv.style.cssText = 'display:flex;gap:var(--sp-space-3);font-size:var(--sp-text-xs)';

        const coreLabel = document.createElement('span');
        const coreBadge = document.createElement('span');
        coreBadge.className = 'badge badge-blue';
        coreBadge.textContent = 'core';
        coreLabel.append(coreBadge, document.createTextNode(' Base skill'));

        const userLabel = document.createElement('span');
        const userBadge = document.createElement('span');
        userBadge.className = 'badge badge-green';
        userBadge.textContent = 'user';
        userLabel.append(userBadge, document.createTextNode(' User version'));

        legendDiv.append(coreLabel, userLabel);

        const closeBtn = document.createElement('button');
        closeBtn.className = 'btn btn-secondary btn-sm';
        closeBtn.setAttribute('data-close-diff', '');
        closeBtn.style.cssText = 'margin-left:auto;font-size:var(--sp-text-xs);padding:2px 8px';
        closeBtn.textContent = 'Close';

        header.append(h4, legendDiv, closeBtn);
        panel.append(header);

        let hasMetaDiff = false;
        if ((userSkill.name || '') !== (coreSkill.name || '') || (userSkill.description || '') !== (coreSkill.description || '')) {
            hasMetaDiff = true;
            const metaSection = document.createElement('div');
            metaSection.style.cssText = 'padding:var(--sp-space-3) var(--sp-space-4);border-bottom:1px solid var(--sp-border-subtle);font-size:var(--sp-text-sm)';

            if ((userSkill.name || '') !== (coreSkill.name || '')) {
                const nameRow = document.createElement('div');
                nameRow.style.marginBottom = 'var(--sp-space-2)';
                const nameLabel = document.createElement('strong');
                nameLabel.textContent = 'Name:';
                const nameOld = document.createElement('span');
                nameOld.className = 'diff-removed';
                nameOld.style.padding = '1px 4px';
                nameOld.textContent = coreSkill.name || '';
                const nameNew = document.createElement('span');
                nameNew.className = 'diff-added';
                nameNew.style.padding = '1px 4px';
                nameNew.textContent = userSkill.name || '';
                nameRow.append(nameLabel, document.createTextNode(' '), nameOld, document.createTextNode(' \u2192 '), nameNew);
                metaSection.append(nameRow);
            }

            if ((userSkill.description || '') !== (coreSkill.description || '')) {
                const descRow = document.createElement('div');
                descRow.style.marginBottom = 'var(--sp-space-2)';
                const descLabel = document.createElement('strong');
                descLabel.textContent = 'Description:';
                const descOld = document.createElement('span');
                descOld.className = 'diff-removed';
                descOld.style.padding = '1px 4px';
                descOld.textContent = coreSkill.description || '';
                const descNew = document.createElement('span');
                descNew.className = 'diff-added';
                descNew.style.padding = '1px 4px';
                descNew.textContent = userSkill.description || '';
                descRow.append(descLabel, document.createTextNode(' '), descOld, document.createTextNode(' \u2192 '), descNew);
                metaSection.append(descRow);
            }

            panel.append(metaSection);
        }

        const diffContent = document.createElement('div');
        diffContent.className = 'diff-content';

        let hasDiff = false;
        for (let i = 0; i < maxLen; i++) {
            const coreLine = i < coreLines.length ? coreLines[i] : '';
            const userLine = i < userLines.length ? userLines[i] : '';
            const lineNum = i + 1;
            if (coreLine === userLine) {
                const unchangedLine = document.createElement('div');
                unchangedLine.className = 'diff-line diff-unchanged';
                const numSpan = document.createElement('span');
                numSpan.className = 'diff-linenum';
                numSpan.textContent = lineNum;
                const textSpan = document.createElement('span');
                textSpan.className = 'diff-text';
                textSpan.textContent = coreLine;
                unchangedLine.append(numSpan, textSpan);
                diffContent.append(unchangedLine);
                hasDiff = true;
            } else {
                if (coreLine) {
                    const removedLine = document.createElement('div');
                    removedLine.className = 'diff-line diff-removed';
                    const rNumSpan = document.createElement('span');
                    rNumSpan.className = 'diff-linenum';
                    rNumSpan.textContent = lineNum;
                    const rTextSpan = document.createElement('span');
                    rTextSpan.className = 'diff-text';
                    rTextSpan.textContent = '- ' + coreLine;
                    removedLine.append(rNumSpan, rTextSpan);
                    diffContent.append(removedLine);
                    hasDiff = true;
                }
                if (userLine) {
                    const addedLine = document.createElement('div');
                    addedLine.className = 'diff-line diff-added';
                    const aNumSpan = document.createElement('span');
                    aNumSpan.className = 'diff-linenum';
                    aNumSpan.textContent = lineNum;
                    const aTextSpan = document.createElement('span');
                    aTextSpan.className = 'diff-text';
                    aTextSpan.textContent = '+ ' + userLine;
                    addedLine.append(aNumSpan, aTextSpan);
                    diffContent.append(addedLine);
                    hasDiff = true;
                }
            }
        }

        if (!hasDiff) {
            const identicalMsg = document.createElement('div');
            identicalMsg.style.cssText = 'padding:var(--sp-space-4);color:var(--sp-text-tertiary);text-align:center';
            identicalMsg.textContent = 'Content is identical';
            diffContent.append(identicalMsg);
        }

        panel.append(diffContent);
        return panel;
    }

    function renderVersionDetails(detailsContainer, versionId) {
        const detail = versionDetails[versionId];
        if (!detail || detail === 'loading') return;
        detailsContainer.replaceChildren();
        if (detail === 'error') {
            const errWrap = document.createElement('div');
            errWrap.style.padding = 'var(--sp-space-4)';
            const errState = document.createElement('div');
            errState.className = 'empty-state';
            const errP = document.createElement('p');
            errP.textContent = 'Failed to load version details.';
            errState.append(errP);
            errWrap.append(errState);
            detailsContainer.append(errWrap);
            return;
        }
        let skills = [];
        if (Array.isArray(detail.skills_snapshot)) {
            skills = detail.skills_snapshot;
        } else if (typeof detail.skills_snapshot === 'string') {
            try { skills = JSON.parse(detail.skills_snapshot); } catch(e) { skills = []; }
        }

        const skillsWrap = document.createElement('div');
        skillsWrap.style.padding = 'var(--sp-space-4)';

        const skillsLabel = document.createElement('div');
        skillsLabel.style.cssText = 'font-size:var(--sp-text-sm);font-weight:600;margin-bottom:var(--sp-space-2);color:var(--sp-text-secondary)';
        skillsLabel.textContent = 'Skills Snapshot (' + skills.length + ')';
        skillsWrap.append(skillsLabel);

        if (skills.length) {
            skills.forEach(function(s) {
                skillsWrap.append(renderSkillRow(s, versionId));
            });
        } else {
            const emptyState = document.createElement('div');
            emptyState.className = 'empty-state';
            emptyState.style.padding = 'var(--sp-space-4)';
            const emptyP = document.createElement('p');
            emptyP.textContent = 'No skills in this snapshot.';
            emptyState.append(emptyP);
            skillsWrap.append(emptyState);
        }

        detailsContainer.append(skillsWrap);

        if (activeDiff && activeDiff.versionId === versionId && diffCache[activeDiff.cacheKey]) {
            const userSkill = skills.find(function(s) { return s.skill_id === activeDiff.skillId; });
            if (userSkill) detailsContainer.append(renderDiffPanel(userSkill, diffCache[activeDiff.cacheKey]));
        }
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

    app.initMarketplaceVersions = (selector) => {
        const root = document.querySelector(selector);
        if (!root) return;

        let activeTab = 'versions';
        const changelogLoaded = {};

        root.addEventListener('click', async (e) => {
            const tabBtn = e.target.closest('[data-tab]');
            if (tabBtn) {
                const newTab = tabBtn.getAttribute('data-tab');
                if (activeTab === newTab) return;
                activeTab = newTab;
                root.querySelectorAll('[data-tab]').forEach((btn) => {
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
                const skillId = compareBtn.getAttribute('data-compare-skill');
                const baseSkillId = compareBtn.getAttribute('data-base-skill');
                const compareVersionId = compareBtn.getAttribute('data-compare-version');
                const versionCard = compareBtn.closest('.version-card');
                const detailsEl = versionCard && versionCard.querySelector('.plugin-details');
                if (detailsEl) await loadCoreDiff(skillId, baseSkillId, compareVersionId, detailsEl);
                return;
            }

            if (e.target.closest('[data-close-diff]')) {
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
                await handleRestore(
                    restoreBtn.getAttribute('data-restore-version'),
                    restoreBtn.getAttribute('data-restore-num'),
                    restoreBtn.getAttribute('data-restore-user')
                );
                return;
            }
        });

        root.addEventListener('change', (e) => {
            if (e.target.id === 'mv-user-select') {
                const userId = e.target.value;
                const groups = root.querySelectorAll('.version-user-group');
                groups.forEach((group) => {
                    const versions = group.querySelectorAll('[data-version-user]');
                    let hasMatch = !userId;
                    versions.forEach((v) => {
                        if (v.getAttribute('data-version-user') === userId) hasMatch = true;
                    });
                    group.style.display = hasMatch ? '' : 'none';
                });
                if (activeTab === 'changelog' && userId) {
                    loadChangelog(userId);
                }
            }
        });

        function setEmptyState(container, message) {
            container.replaceChildren();
            const state = document.createElement('div');
            state.className = 'empty-state';
            const p = document.createElement('p');
            p.textContent = message;
            state.append(p);
            container.append(state);
        }

        async function loadChangelog(userId) {
            const container = document.getElementById('mv-changelog-tab');
            if (!container) return;
            if (!userId) {
                setEmptyState(container, 'Select a user to view changelog.');
                return;
            }
            container.replaceChildren();
            const loadingCenter = document.createElement('div');
            loadingCenter.className = 'loading-center';
            const spinner = document.createElement('div');
            spinner.className = 'loading-spinner';
            spinner.setAttribute('role', 'status');
            const srOnly = document.createElement('span');
            srOnly.className = 'sr-only';
            srOnly.textContent = 'Loading...';
            spinner.append(srOnly);
            loadingCenter.append(spinner);
            container.append(loadingCenter);
            try {
                const changelog = await marketplaceApi(userId, '/changelog');
                changelogLoaded[userId] = true;
                if (!changelog || !changelog.length) {
                    setEmptyState(container, 'No changelog entries found for this user.');
                    return;
                }
                container.replaceChildren();
                const tableContainer = document.createElement('div');
                tableContainer.className = 'table-container';
                const tableScroll = document.createElement('div');
                tableScroll.className = 'table-scroll';
                const table = document.createElement('table');
                table.className = 'data-table';
                const thead = document.createElement('thead');
                const headRow = document.createElement('tr');
                ['Action', 'Skill ID', 'Name', 'Detail', 'Time'].forEach(function(text) {
                    const th = document.createElement('th');
                    th.textContent = text;
                    headRow.append(th);
                });
                thead.append(headRow);
                const tbody = document.createElement('tbody');
                changelog.forEach(function(entry) {
                    let actionClass = 'badge-gray';
                    switch(entry.action) {
                        case 'added': actionClass = 'badge-green'; break;
                        case 'updated': actionClass = 'badge-yellow'; break;
                        case 'deleted': actionClass = 'badge-red'; break;
                        case 'restored': actionClass = 'badge-blue'; break;
                    }
                    const tr = document.createElement('tr');
                    const td1 = document.createElement('td');
                    const actionBadge = document.createElement('span');
                    actionBadge.className = 'badge ' + actionClass;
                    actionBadge.textContent = entry.action;
                    td1.append(actionBadge);

                    const td2 = document.createElement('td');
                    const codeEl = document.createElement('code');
                    codeEl.style.cssText = 'background:var(--sp-bg-surface-raised);padding:1px 4px;border-radius:var(--sp-radius-xs);font-size:var(--sp-text-xs)';
                    codeEl.textContent = entry.skill_id;
                    td2.append(codeEl);

                    const td3 = document.createElement('td');
                    td3.textContent = entry.skill_name;

                    const td4 = document.createElement('td');
                    td4.style.color = 'var(--sp-text-secondary)';
                    td4.textContent = entry.detail;

                    const td5 = document.createElement('td');
                    const timeSpan = document.createElement('span');
                    timeSpan.title = app.formatDate(entry.created_at);
                    timeSpan.textContent = app.formatRelativeTime(entry.created_at);
                    td5.append(timeSpan);

                    tr.append(td1, td2, td3, td4, td5);
                    tbody.append(tr);
                });
                table.append(thead, tbody);
                tableScroll.append(table);
                tableContainer.append(tableScroll);
                container.append(tableContainer);
            } catch(err) {
                setEmptyState(container, 'Failed to load changelog.');
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

        form.addEventListener('submit', async (e) => {
            e.preventDefault();
            const formData = new FormData(form);
            const keywordsRaw = formData.get('keywords') || '';
            const keywords = keywordsRaw.split(',').map((t) => t.trim()).filter(Boolean);
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
            deleteBtn.addEventListener('click', () => {
                app.shared.showConfirmDialog('Delete Plugin?', 'Are you sure you want to delete this plugin? This cannot be undone.', 'Delete', async () => {
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

((app) => {
    'use strict';

    app.pluginWizardSteps = {
        renderCurrentStep: () => ''
    };
})(window.AdminApp);

(function(app) {
    'use strict';

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
        const steps = document.createElement('div');
        steps.className = 'wizard-steps';
        steps.style.cssText = 'display:flex;gap:var(--sp-space-1);margin-bottom:var(--sp-space-6);flex-wrap:wrap';
        for (let i = 1; i <= TOTAL_STEPS; i++) {
            const isActive = i === state.step;
            const isDone = i < state.step;
            const bgColor = isActive ? 'var(--sp-accent)' : (isDone ? 'var(--sp-success)' : 'var(--sp-bg-tertiary)');
            const textColor = (isActive || isDone) ? '#fff' : 'var(--sp-text-tertiary)';
            const stepDiv = document.createElement('div');
            stepDiv.style.cssText = 'display:flex;align-items:center;gap:var(--sp-space-2);padding:var(--sp-space-2) var(--sp-space-3);border-radius:var(--sp-radius-md);font-size:var(--sp-text-sm);background:' + bgColor + ';color:' + textColor + ';font-weight:' + (isActive ? '600' : '400');
            const numSpan = document.createElement('span');
            numSpan.style.cssText = 'width:20px;height:20px;border-radius:50%;background:rgba(255,255,255,0.2);display:inline-flex;align-items:center;justify-content:center;font-size:var(--sp-text-xs)';
            numSpan.textContent = i;
            const labelSpan = document.createElement('span');
            labelSpan.textContent = labels[i - 1];
            stepDiv.append(numSpan, labelSpan);
            steps.append(stepDiv);
        }
        container.replaceChildren(steps);
    }

    function renderNav() {
        const nav = document.getElementById('wizard-nav');
        if (!nav) return;
        const wrapper = document.createElement('div');
        wrapper.style.cssText = 'display:flex;gap:var(--sp-space-3);margin-top:var(--sp-space-6)';
        if (state.step > 1) {
            const prevBtn = document.createElement('button');
            prevBtn.type = 'button';
            prevBtn.className = 'btn btn-secondary';
            prevBtn.id = 'wizard-prev';
            prevBtn.textContent = 'Previous';
            wrapper.append(prevBtn);
        }
        if (state.step < TOTAL_STEPS) {
            const nextBtn = document.createElement('button');
            nextBtn.type = 'button';
            nextBtn.className = 'btn btn-primary';
            nextBtn.id = 'wizard-next';
            nextBtn.textContent = 'Next';
            wrapper.append(nextBtn);
        }
        if (state.step === TOTAL_STEPS) {
            const createBtn = document.createElement('button');
            createBtn.type = 'button';
            createBtn.className = 'btn btn-primary';
            createBtn.id = 'wizard-create';
            createBtn.textContent = 'Create Plugin';
            wrapper.append(createBtn);
        }
        nav.replaceChildren(wrapper);
    }

    function saveCurrentStepState() {
        if (!root) return;
        if (state.step === 1) {
            ['plugin_id', 'name', 'description', 'version', 'category'].forEach((name) => {
                const input = root.querySelector('[name="' + name + '"]');
                if (input) state.form[name] = input.tagName === 'TEXTAREA' ? input.value : input.value;
            });
        }
        if (state.step === 2) {
            state.selectedSkills = {};
            root.querySelectorAll('input[name="wizard-skills"]:checked').forEach((cb) => { state.selectedSkills[cb.value] = true; });
        }
        if (state.step === 3) {
            state.selectedAgents = {};
            root.querySelectorAll('input[name="wizard-agents"]:checked').forEach((cb) => { state.selectedAgents[cb.value] = true; });
        }
        if (state.step === 4) {
            state.selectedMcpServers = {};
            root.querySelectorAll('input[name="wizard-mcp"]:checked').forEach((cb) => { state.selectedMcpServers[cb.value] = true; });
        }
        if (state.step === 5) {
            const entries = root.querySelectorAll('.hook-entry');
            state.hooks = [];
            entries.forEach((entry) => {
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
            root.querySelectorAll('input[name="wizard-roles"]:checked').forEach((cb) => { state.form.roles[cb.value] = true; });
            const authorInput = root.querySelector('[name="author_name"]');
            if (authorInput) state.form.author_name = authorInput.value;
            const keywordsInput = root.querySelector('[name="keywords"]');
            if (keywordsInput) state.form.keywords = keywordsInput.value;
        }
    }

    function renderStep() {
        const contentEl = document.getElementById('wizard-step-content');
        if (!contentEl) return;
        contentEl.replaceChildren();

        if (state.step === 7) {
            const frag = getTemplate('tpl-step-7');
            contentEl.append(frag);
            renderReview();
        } else if (state.step === 5) {
            const frag5 = getTemplate('tpl-step-5');
            contentEl.append(frag5);
            renderHooks();
        } else {
            const frag2 = getTemplate('tpl-step-' + state.step);
            contentEl.append(frag2);
            restoreStepState();
        }

        renderStepIndicator();
        renderNav();
        app.formUtils.attachFilterHandlers(contentEl);
    }

    function restoreStepState() {
        if (state.step === 1) {
            ['plugin_id', 'name', 'description', 'version', 'category'].forEach((name) => {
                const input = root.querySelector('[name="' + name + '"]');
                if (input && state.form[name]) {
                    if (input.tagName === 'TEXTAREA') input.value = state.form[name];
                    else input.value = state.form[name];
                }
            });
        }
        if (state.step === 2) {
            Object.keys(state.selectedSkills).forEach((val) => {
                if (!state.selectedSkills[val]) return;
                const cb = root.querySelector('input[name="wizard-skills"][value="' + val + '"]');
                if (cb) cb.checked = true;
            });
        }
        if (state.step === 3) {
            Object.keys(state.selectedAgents).forEach((val) => {
                if (!state.selectedAgents[val]) return;
                const cb = root.querySelector('input[name="wizard-agents"][value="' + val + '"]');
                if (cb) cb.checked = true;
            });
        }
        if (state.step === 4) {
            Object.keys(state.selectedMcpServers).forEach((val) => {
                if (!state.selectedMcpServers[val]) return;
                const cb = root.querySelector('input[name="wizard-mcp"][value="' + val + '"]');
                if (cb) cb.checked = true;
            });
        }
        if (state.step === 6) {
            Object.keys(state.form.roles).forEach((val) => {
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
        list.replaceChildren();
        state.hooks.forEach((hook) => {
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
            list.append(frag);
        });
    }

    function renderReview() {
        const el = document.getElementById('wizard-review');
        if (!el) return;
        const f = state.form;
        const selectedSkills = Object.keys(state.selectedSkills).filter((k) => state.selectedSkills[k]);
        const selectedAgents = Object.keys(state.selectedAgents).filter((k) => state.selectedAgents[k]);
        const selectedMcp = Object.keys(state.selectedMcpServers).filter((k) => state.selectedMcpServers[k]);
        const selectedRoles = Object.keys(f.roles).filter((k) => f.roles[k]);

        function buildBadgeList(items, emptyMsg) {
            const container = document.createDocumentFragment();
            if (!items.length) {
                const empty = document.createElement('span');
                empty.style.cssText = 'color:var(--sp-text-tertiary)';
                empty.textContent = emptyMsg;
                container.append(empty);
                return container;
            }
            items.forEach(function(item) {
                const badge = document.createElement('span');
                badge.className = 'badge badge-blue';
                badge.style.cssText = 'margin:var(--sp-space-1)';
                badge.textContent = item;
                container.append(badge);
            });
            return container;
        }

        function addRow(frag, labelText, valueText) {
            const strong = document.createElement('strong');
            strong.textContent = labelText;
            const span = document.createElement('span');
            span.textContent = valueText;
            frag.append(strong, span);
        }

        function addBadgeRow(frag, labelText, items, emptyMsg, wrap) {
            const strong = document.createElement('strong');
            strong.textContent = labelText;
            const div = document.createElement('div');
            if (wrap) div.style.cssText = 'display:flex;flex-wrap:wrap';
            div.append(buildBadgeList(items, emptyMsg));
            frag.append(strong, div);
        }

        const frag = document.createDocumentFragment();
        addRow(frag, 'Plugin ID:', f.plugin_id || '-');
        addRow(frag, 'Name:', f.name || '-');
        addRow(frag, 'Description:', f.description || '-');
        addRow(frag, 'Version:', f.version || '0.1.0');
        addRow(frag, 'Category:', f.category || '-');
        addRow(frag, 'Author:', f.author_name || '-');
        addRow(frag, 'Keywords:', f.keywords || '-');
        addBadgeRow(frag, 'Roles:', selectedRoles, 'None selected', false);
        addBadgeRow(frag, 'Skills (' + selectedSkills.length + '):', selectedSkills, 'None selected', true);
        addBadgeRow(frag, 'Agents (' + selectedAgents.length + '):', selectedAgents, 'None selected', true);
        addBadgeRow(frag, 'MCP (' + selectedMcp.length + '):', selectedMcp, 'None selected', true);

        const hooksStrong = document.createElement('strong');
        hooksStrong.textContent = 'Hooks (' + state.hooks.length + '):';
        const hooksSpan = document.createElement('span');
        hooksSpan.textContent = state.hooks.length > 0
            ? state.hooks.map(function(h) { return h.event + ': ' + (h.command || '?'); }).join(', ')
            : 'None';
        frag.append(hooksStrong, hooksSpan);

        el.replaceChildren(frag);
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
            keywords: (f.keywords || '').split(',').map((t) => t.trim()).filter(Boolean),
            author: { name: f.author_name || '' },
            roles: Object.keys(f.roles).filter((k) => f.roles[k]),
            skills: Object.keys(state.selectedSkills).filter((k) => state.selectedSkills[k]),
            agents: Object.keys(state.selectedAgents).filter((k) => state.selectedAgents[k]),
            mcp_servers: Object.keys(state.selectedMcpServers).filter((k) => state.selectedMcpServers[k]),
            hooks: state.hooks.filter((h) => h.command).map((h) => {
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

        root.addEventListener('click', (e) => {
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
                if (container) container.querySelectorAll('input[type="checkbox"]').forEach((cb) => { cb.checked = true; });
                return;
            }
            const deselectAllBtn = e.target.closest('[data-deselect-all]');
            if (deselectAllBtn) {
                const listId2 = deselectAllBtn.getAttribute('data-deselect-all');
                const container2 = root.querySelector('[data-checklist="' + listId2 + '"]');
                if (container2) container2.querySelectorAll('input[type="checkbox"]').forEach((cb) => { cb.checked = false; });
                return;
            }
        });

        root.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' && e.target.id === 'import-url') { e.preventDefault(); submitImport(); }
            if (e.key === 'Escape') hideImportModal();
        });
    };
})(window.AdminApp);

(function(app) {
    'use strict';

    function copyToClipboard(text, btn) {
        navigator.clipboard.writeText(text).then(() => {
            const orig = btn.textContent;
            btn.textContent = 'Copied!';
            btn.classList.add('copied');
            setTimeout(() => {
                btn.textContent = orig;
                btn.classList.remove('copied');
            }, 2000);
        }).catch(() => {
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
        const lines = ['#!/bin/bash', '# Install script for Enterprise Demo plugins', 'set -e', ''];
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
        if (icon) icon.textContent = open ? '\u25B6' : '\u25BC';
    }

    async function downloadZip(data) {
        const btn = document.getElementById('btn-download-zip');
        if (!btn) return;
        const origText = btn.textContent;
        btn.textContent = 'Generating...';
        btn.disabled = true;
        try {
            const JSZip = await app.shared.loadJSZip();
            const zip = new JSZip();
            const plugins = data.plugins || [];
            plugins.forEach((plugin) => {
                const folder = zip.folder(plugin.id);
                (plugin.files || []).forEach((file) => {
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
            a.download = 'plugins.zip';
            document.body.append(a);
            a.click();
            a.remove();
            URL.revokeObjectURL(url);
            btn.textContent = origText;
            btn.disabled = false;
            app.Toast.show('ZIP downloaded successfully', 'success');
        } catch (err) {
            btn.textContent = origText;
            btn.disabled = false;
            app.Toast.show('Failed to generate ZIP: ' + err.message, 'error');
        }
    }

    app.exportInteractions = (exportData) => {
        if (!exportData) return;

        app.events.on('click', '#btn-download-zip', () => {
            downloadZip(exportData);
        });

        app.events.on('click', '[data-action="toggle-bundle"]', (e, el) => {
            toggleBundle(el.getAttribute('data-idx'));
        });

        app.events.on('click', '[data-action="copy-content"]', (e, el) => {
            const pluginIdx = parseInt(el.getAttribute('data-plugin-idx'), 10);
            const fileIdx = parseInt(el.getAttribute('data-file-idx'), 10);
            const plugin = (exportData.plugins || [])[pluginIdx];
            if (plugin) {
                const file = (plugin.files || [])[fileIdx];
                if (file) copyToClipboard(file.content, el);
            }
        });

        app.events.on('click', '[data-action="copy-script"]', (e, el) => {
            const script = generateInstallScript(exportData);
            copyToClipboard(script, el);
        });
    };
})(window.AdminApp);

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

(function(app) {
    'use strict';

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
        items.forEach((a) => {
            const cat = a.category || 'Other';
            if (!groups[cat]) groups[cat] = [];
            groups[cat].push(a);
        });
        return groups;
    }
    function renderAchievementCard(a) {
        const unlocked = a.total_unlocked > 0;
        const card = document.createElement('div');
        card.className = unlocked ? 'achievement-card unlocked' : 'achievement-card locked';
        const pct = unlocked ? 100 : (a.unlock_percentage || 0);
        const icon = CATEGORY_ICONS[a.category] || '\u2B50';

        const iconDiv = document.createElement('div');
        iconDiv.className = 'achievement-icon';
        iconDiv.textContent = icon;

        const nameDiv = document.createElement('div');
        nameDiv.style.cssText = 'font-weight:600;font-size:var(--sp-text-sm);color:var(--sp-text-primary)';
        nameDiv.textContent = a.name;

        const descDiv = document.createElement('div');
        descDiv.style.cssText = 'font-size:var(--sp-text-xs);color:var(--sp-text-tertiary);margin-top:var(--sp-space-1)';
        descDiv.textContent = a.description;

        const countDiv = document.createElement('div');
        countDiv.style.cssText = 'font-size:var(--sp-text-xs);color:var(--sp-text-tertiary);margin-top:var(--sp-space-1)';
        countDiv.textContent = a.total_unlocked + ' unlocked';

        const bar = document.createElement('div');
        bar.className = 'unlock-bar';
        const barFill = document.createElement('div');
        barFill.className = 'unlock-bar-fill';
        barFill.style.width = pct + '%';
        bar.append(barFill);

        card.append(iconDiv, nameDiv, descDiv, countDiv, bar);
        return card;
    }
    function renderAchievementsContent(data, root) {
        const items = Array.isArray(data) ? data : (data.achievements || []);
        root.replaceChildren();
        if (!items.length) {
            const empty = document.createElement('div');
            empty.className = 'empty-state';
            const p = document.createElement('p');
            p.textContent = 'No achievements defined.';
            empty.append(p);
            root.append(empty);
            return;
        }
        const groups = groupByCategory(items);
        const categories = Object.keys(groups);
        categories.forEach((cat) => {
            const section = document.createElement('div');
            section.style.marginBottom = 'var(--sp-space-6)';

            const title = document.createElement('div');
            title.className = 'section-title';
            title.textContent = cat;

            const grid = document.createElement('div');
            grid.className = 'achievement-grid';
            groups[cat].forEach((a) => {
                grid.append(renderAchievementCard(a));
            });

            section.append(title, grid);
            root.append(section);
        });
    }
    app.renderAchievements = () => {
        const root = document.getElementById('achievements-content');
        if (!root) return;
        root.replaceChildren();
        const loadingDiv = document.createElement('div');
        loadingDiv.className = 'loading-center';
        const spinner = document.createElement('div');
        spinner.className = 'loading-spinner';
        loadingDiv.append(spinner);
        root.append(loadingDiv);
        app.api('/gamification/achievements').then((data) => {
            renderAchievementsContent(data, root);
        }).catch((err) => {
            root.replaceChildren();
            const empty = document.createElement('div');
            empty.className = 'empty-state';
            const p = document.createElement('p');
            p.textContent = 'Failed to load achievements.';
            empty.append(p);
            root.append(empty);
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
        if (!data) {
            const noData = document.createElement('p');
            noData.className = 'text-muted';
            noData.textContent = 'No detail data available.';
            return noData;
        }

        const frag = document.createDocumentFragment();

        const descSection = document.createElement('div');
        descSection.className = 'detail-section';
        const descLabel = document.createElement('strong');
        descLabel.textContent = 'Description';
        const descP = document.createElement('p');
        descP.style.cssText = 'margin:var(--sp-space-1) 0;color:var(--sp-text-secondary);font-size:var(--sp-text-sm)';
        descP.textContent = data.description || 'No description';
        descSection.append(descLabel, descP);
        frag.append(descSection);

        if (data.command) {
            const cmdSection = document.createElement('div');
            cmdSection.className = 'detail-section';
            const cmdLabel = document.createElement('strong');
            cmdLabel.textContent = 'Command';
            const cmdPre = document.createElement('pre');
            cmdPre.style.cssText = 'margin:var(--sp-space-1) 0;font-size:var(--sp-text-xs);background:var(--sp-bg-surface-raised);padding:var(--sp-space-2);border-radius:var(--sp-radius-sm);overflow-x:auto';
            cmdPre.textContent = data.command;
            cmdSection.append(cmdLabel, cmdPre);
            frag.append(cmdSection);
        }

        if (data.tags && data.tags.length) {
            const tagSection = document.createElement('div');
            tagSection.className = 'detail-section';
            const tagLabel = document.createElement('strong');
            tagLabel.textContent = 'Tags';
            tagSection.append(tagLabel, document.createElement('br'));
            const badgeRow = document.createElement('div');
            badgeRow.className = 'badge-row';
            badgeRow.style.marginTop = 'var(--sp-space-1)';
            data.tags.forEach((tag) => {
                const badge = document.createElement('span');
                badge.className = 'badge badge-gray';
                badge.textContent = tag;
                badgeRow.append(badge);
            });
            tagSection.append(badgeRow);
            frag.append(tagSection);
        }

        const jsonSection = document.createElement('div');
        jsonSection.className = 'detail-section';
        const details = document.createElement('details');
        const summary = document.createElement('summary');
        summary.style.cssText = 'cursor:pointer;font-size:var(--sp-text-sm);color:var(--sp-text-secondary)';
        summary.textContent = 'JSON Config';
        details.append(summary);
        details.append(app.OrgCommon.formatJson(data));
        jsonSection.append(details);
        frag.append(jsonSection);

        return frag;
    }

    function initExpandRows() {
        app.OrgCommon.initExpandRows('.data-table', (row, detailRow) => {
            const content = detailRow.querySelector('[data-skill-expand]');
            if (content && !content.hasAttribute('data-loaded')) {
                const skillId = content.getAttribute('data-skill-expand');
                content.replaceChildren();
                content.append(renderSkillExpand(skillId));
                content.setAttribute('data-loaded', 'true');
            }
        });
    }

    function initDeleteHandlers() {
        app.events.on('click', '[data-delete-skill]', (e, el) => {
            const skillId = el.getAttribute('data-delete-skill');
            if (!confirm('Are you sure you want to delete skill "' + skillId + '"? This cannot be undone.')) return;

            fetch('/api/admin/skills/' + encodeURIComponent(skillId), { method: 'DELETE' })
                .then((res) => {
                    if (res.ok) {
                        app.Toast.show('Skill deleted', 'success');
                        setTimeout(() => { window.location.reload(); }, 500);
                    } else {
                        app.Toast.show('Failed to delete skill', 'error');
                    }
                })
                .catch(() => {
                    app.Toast.show('Failed to delete skill', 'error');
                });
        });
    }

    function initForkHandlers() {
        app.events.on('click', '[data-fork-skill]', (e, el) => {
            const skillId = el.getAttribute('data-fork-skill');
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
            .then((res) => {
                if (res.ok) {
                    app.Toast.show('Skill customized', 'success');
                    setTimeout(() => { window.location.reload(); }, 500);
                } else {
                    app.Toast.show('Failed to customize skill', 'error');
                }
            })
            .catch(() => {
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

        app.events.on('click', '[data-assign-skill]', (e, el) => {
            const skillId = el.getAttribute('data-assign-skill');
            const skillName = el.getAttribute('data-skill-name') || skillId;
            const data = getSkillDetail(skillId);
            const currentPluginIds = data && data.assigned_plugin_ids ? data.assigned_plugin_ids : [];
            assignApi.open(skillId, skillName, currentPluginIds);
        });

        app.events.on('click', '[data-assign-save]', (e, el) => {
            const entityId = el.getAttribute('data-entity-id');
            const checkboxes = document.querySelectorAll('#assign-panel input[name="plugin_id"]');
            const selectedPlugins = [];
            checkboxes.forEach((cb) => {
                if (cb.checked) selectedPlugins.push(cb.value);
            });

            el.disabled = true;
            el.textContent = 'Saving...';

            const promises = allPlugins.map((plugin) => {
                return fetch('/api/admin/plugins/' + encodeURIComponent(plugin.id) + '/skills')
                    .then((res) => { return res.json(); })
                    .then((currentSkills) => {
                        let skillIds = (currentSkills || []).slice();
                        const shouldInclude = selectedPlugins.includes(plugin.id);
                        const hasSkill = skillIds.includes(entityId);

                        if (shouldInclude && !hasSkill) {
                            skillIds.push(entityId);
                        } else if (!shouldInclude && hasSkill) {
                            skillIds = skillIds.filter((s) => { return s !== entityId; });
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
                .then(() => {
                    app.Toast.show('Plugin assignments updated', 'success');
                    assignApi.close();
                    setTimeout(() => { window.location.reload(); }, 500);
                })
                .catch(() => {
                    app.Toast.show('Failed to update assignments', 'error');
                    el.disabled = false;
                    el.textContent = 'Save';
                });
        });
    }

    function initEditPanel() {
        const editPanel = app.OrgCommon.initEditPanel({
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

        app.events.on('click', '[data-edit-skill]', (e, el) => {
            const skillId = el.getAttribute('data-edit-skill');
            const data = getSkillDetail(skillId);
            if (data && editPanel) editPanel.open(skillId, data);
        });
    }

    function initBulkHandlers() {
        const bulk = app.OrgCommon.initBulkActions('.data-table', 'bulk-bar');
        if (!bulk) return;

        const allPlugins = getAllPlugins();
        const assignApi = app.OrgCommon.initAssignPanel({
            panelId: 'assign-panel',
            allPlugins: allPlugins
        });

        const deleteBtn = document.getElementById('bulk-delete-btn');
        if (deleteBtn) {
            deleteBtn.addEventListener('click', () => {
                const ids = bulk.getSelected();
                if (!ids.length) return;
                if (!confirm('Delete ' + ids.length + ' skill(s)? This action cannot be undone.')) return;
                Promise.all(ids.map((id) => {
                    return fetch('/api/admin/skills/' + encodeURIComponent(id), { method: 'DELETE' });
                })).then(() => {
                    app.Toast.show(ids.length + ' skills deleted', 'success');
                    setTimeout(() => { window.location.reload(); }, 500);
                }).catch(() => {
                    app.Toast.show('Failed to delete some skills', 'error');
                });
            });
        }

        const assignBtn = document.getElementById('bulk-assign-btn');
        if (assignBtn && assignApi) {
            assignBtn.addEventListener('click', () => {
                const ids = bulk.getSelected();
                if (!ids.length) return;
                assignApi.open(ids.join(','), ids.length + ' skills', []);
            });
        }

        const categoryBtn = document.getElementById('bulk-category-btn');
        if (categoryBtn) {
            categoryBtn.addEventListener('click', () => {
                const ids = bulk.getSelected();
                if (!ids.length) return;
                const category = prompt('Enter category for ' + ids.length + ' skill(s):');
                if (category === null) return;
                Promise.all(ids.map((id) => {
                    return fetch('/api/public/skills/' + encodeURIComponent(id), {
                        method: 'PUT',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ category_id: category })
                    });
                })).then(() => {
                    app.Toast.show('Category updated for ' + ids.length + ' skills', 'success');
                    setTimeout(() => { window.location.reload(); }, 500);
                }).catch(() => {
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
        if (!data) {
            const noData = document.createElement('p');
            noData.className = 'text-muted';
            noData.textContent = 'No detail data available.';
            return noData;
        }

        const frag = document.createDocumentFragment();

        if (data.system_prompt) {
            const promptSection = document.createElement('div');
            promptSection.className = 'detail-section';
            const promptLabel = document.createElement('strong');
            promptLabel.textContent = 'System Prompt';
            const promptPre = document.createElement('pre');
            promptPre.style.cssText = 'margin:var(--sp-space-1) 0;max-height:200px;overflow:auto;font-size:var(--sp-text-xs);background:var(--sp-bg-surface-raised);padding:var(--sp-space-2);border-radius:var(--sp-radius-sm);white-space:pre-wrap;word-break:break-word';
            promptPre.textContent = data.system_prompt;
            promptSection.append(promptLabel, promptPre);
            frag.append(promptSection);
        }

        if (data.port || data.endpoint) {
            const connSection = document.createElement('div');
            connSection.className = 'detail-section';
            const connLabel = document.createElement('strong');
            connLabel.textContent = 'Connection';
            const connDiv = document.createElement('div');
            connDiv.style.cssText = 'margin:var(--sp-space-1) 0;font-size:var(--sp-text-sm);color:var(--sp-text-secondary)';
            if (data.port) {
                const portRow = document.createElement('div');
                portRow.append('Port: ');
                const portCode = document.createElement('code');
                portCode.className = 'code-inline';
                portCode.textContent = String(data.port);
                portRow.append(portCode);
                connDiv.append(portRow);
            }
            if (data.endpoint) {
                const endpointRow = document.createElement('div');
                endpointRow.append('Endpoint: ');
                const endpointCode = document.createElement('code');
                endpointCode.className = 'code-inline';
                endpointCode.textContent = data.endpoint;
                endpointRow.append(endpointCode);
                connDiv.append(endpointRow);
            }
            connSection.append(connLabel, connDiv);
            frag.append(connSection);
        }

        if ((data.skill_count && data.skill_count > 0) || (data.mcp_count && data.mcp_count > 0)) {
            const capSection = document.createElement('div');
            capSection.className = 'detail-section';
            const capLabel = document.createElement('strong');
            capLabel.textContent = 'Capabilities';
            const badgeRow = document.createElement('div');
            badgeRow.className = 'badge-row';
            badgeRow.style.marginTop = 'var(--sp-space-1)';
            if (data.skill_count > 0) {
                const skillBadge = document.createElement('span');
                skillBadge.className = 'badge badge-green';
                skillBadge.textContent = data.skill_count + ' skill' + (data.skill_count !== 1 ? 's' : '');
                badgeRow.append(skillBadge);
            }
            if (data.mcp_count > 0) {
                const mcpBadge = document.createElement('span');
                mcpBadge.className = 'badge badge-yellow';
                mcpBadge.textContent = data.mcp_count + ' MCP server' + (data.mcp_count !== 1 ? 's' : '');
                badgeRow.append(mcpBadge);
            }
            capSection.append(capLabel, badgeRow);
            frag.append(capSection);
        }

        const jsonSection = document.createElement('div');
        jsonSection.className = 'detail-section';
        const details = document.createElement('details');
        const summary = document.createElement('summary');
        summary.style.cssText = 'cursor:pointer;font-size:var(--sp-text-sm);color:var(--sp-text-secondary)';
        summary.textContent = 'JSON Config';
        details.append(summary);
        details.append(app.OrgCommon.formatJson(data));
        jsonSection.append(details);
        frag.append(jsonSection);

        return frag;
    }

    function initExpandRows() {
        app.OrgCommon.initExpandRows('.data-table', (row, detailRow) => {
            const content = detailRow.querySelector('[data-agent-expand]');
            if (content && !content.hasAttribute('data-loaded')) {
                const agentId = content.getAttribute('data-agent-expand');
                content.replaceChildren();
                content.append(renderAgentExpand(agentId));
                content.setAttribute('data-loaded', 'true');
            }
        });
    }

    function initDeleteHandlers() {
        app.events.on('click', '[data-delete-agent]', (e, el) => {
            const agentId = el.getAttribute('data-delete-agent');
            if (!confirm('Are you sure you want to delete agent "' + agentId + '"? This cannot be undone.')) return;

            fetch('/api/admin/agents/' + encodeURIComponent(agentId), { method: 'DELETE' })
                .then((res) => {
                    if (res.ok) {
                        app.Toast.show('Agent deleted', 'success');
                        setTimeout(() => { window.location.reload(); }, 500);
                    } else {
                        app.Toast.show('Failed to delete agent', 'error');
                    }
                })
                .catch(() => {
                    app.Toast.show('Failed to delete agent', 'error');
                });
        });
    }

    function initForkHandlers() {
        app.events.on('click', '[data-fork-agent]', (e, el) => {
            const agentId = el.getAttribute('data-fork-agent');
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
            .then((res) => {
                if (res.ok) {
                    app.Toast.show('Agent customized', 'success');
                    setTimeout(() => { window.location.reload(); }, 500);
                } else {
                    app.Toast.show('Failed to customize agent', 'error');
                }
            })
            .catch(() => {
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

        app.events.on('click', '[data-assign-agent]', (e, el) => {
            const agentId = el.getAttribute('data-assign-agent');
            const agentName = el.getAttribute('data-agent-name') || agentId;
            const data = getAgentDetail(agentId);
            const currentPluginIds = data && data.assigned_plugin_ids ? data.assigned_plugin_ids : [];
            assignApi.open(agentId, agentName, currentPluginIds);
        });

        app.events.on('click', '[data-assign-save]', (e, el) => {
            const entityId = el.getAttribute('data-entity-id');
            const checkboxes = document.querySelectorAll('#assign-panel input[name="plugin_id"]');
            const selectedPlugins = [];
            checkboxes.forEach((cb) => {
                if (cb.checked) selectedPlugins.push(cb.value);
            });

            el.disabled = true;
            el.textContent = 'Saving...';

            const promises = allPlugins.map((plugin) => {
                return fetch('/api/admin/plugins/' + encodeURIComponent(plugin.id) + '/agents')
                    .then((res) => { return res.json(); })
                    .then((currentAgents) => {
                        let agentIds = (currentAgents || []).slice();
                        const shouldInclude = selectedPlugins.includes(plugin.id);
                        const hasAgent = agentIds.includes(entityId);

                        if (shouldInclude && !hasAgent) {
                            agentIds.push(entityId);
                        } else if (!shouldInclude && hasAgent) {
                            agentIds = agentIds.filter((a) => { return a !== entityId; });
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
                .then(() => {
                    app.Toast.show('Plugin assignments updated', 'success');
                    assignApi.close();
                    setTimeout(() => { window.location.reload(); }, 500);
                })
                .catch(() => {
                    app.Toast.show('Failed to update assignments', 'error');
                    el.disabled = false;
                    el.textContent = 'Save';
                });
        });
    }

    function initEditPanel() {
        const editPanel = app.OrgCommon.initEditPanel({
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

        app.events.on('click', '[data-edit-agent]', (e, el) => {
            const agentId = el.getAttribute('data-edit-agent');
            const data = getAgentDetail(agentId);
            if (data && editPanel) editPanel.open(agentId, data);
        });
    }

    function initBulkHandlers() {
        const bulk = app.OrgCommon.initBulkActions('.data-table', 'bulk-bar');
        if (!bulk) return;

        const allPlugins = getAllPlugins();
        const assignApi = app.OrgCommon.initAssignPanel({
            panelId: 'assign-panel',
            allPlugins: allPlugins
        });

        const deleteBtn = document.getElementById('bulk-delete-btn');
        if (deleteBtn) {
            deleteBtn.addEventListener('click', () => {
                const ids = bulk.getSelected();
                if (!ids.length) return;
                if (!confirm('Delete ' + ids.length + ' agent(s)? This action cannot be undone.')) return;
                Promise.all(ids.map((id) => {
                    return fetch('/api/admin/agents/' + encodeURIComponent(id), { method: 'DELETE' });
                })).then(() => {
                    app.Toast.show(ids.length + ' agents deleted', 'success');
                    setTimeout(() => { window.location.reload(); }, 500);
                }).catch(() => {
                    app.Toast.show('Failed to delete some agents', 'error');
                });
            });
        }

        const assignBtn = document.getElementById('bulk-assign-btn');
        if (assignBtn && assignApi) {
            assignBtn.addEventListener('click', () => {
                const ids = bulk.getSelected();
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
                rows.forEach((row) => {
                    const name = (row.getAttribute('data-name') || '').toLowerCase();
                    const match = !query || name.includes(query);
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

        app.events.on('click', '[data-hook-json]', (e, el) => {
            const hookId = el.getAttribute('data-hook-json');
            const container = document.querySelector('[data-hook-json-container="' + hookId + '"]');
            if (!container) return;

            if (container.style.display === 'none') {
                const script = document.querySelector('script[data-hook-detail="' + hookId + '"]');
                if (script) {
                    try {
                        const data = JSON.parse(script.textContent);
                        container.replaceChildren();
                        container.append(OrgCommon.formatJson(data));
                    } catch (err) {
                        container.replaceChildren();
                        const errSpan = document.createElement('span');
                        errSpan.className = 'text-muted';
                        errSpan.textContent = 'Failed to parse JSON';
                        container.append(errSpan);
                    }
                }
                container.style.display = 'block';
                el.textContent = 'Hide JSON';
            } else {
                container.style.display = 'none';
                el.textContent = 'Show JSON';
            }
        });

        app.events.on('click', '[data-action="delete"][data-entity-type="hook"]', (e, el) => {
            const id = el.getAttribute('data-entity-id');
            if (!confirm('Delete this hook? This cannot be undone.')) return;

            app.api('/hooks/' + encodeURIComponent(id), {
                method: 'DELETE'
            }).then(() => {
                app.Toast.show('Hook deleted', 'success');
                const row = document.querySelector('tr[data-entity-id="' + id + '"].clickable-row');
                if (row) {
                    const detail = row.nextElementSibling;
                    if (detail && detail.classList.contains('detail-row')) {
                        detail.remove();
                    }
                    row.remove();
                }
            }).catch((err) => {
                app.Toast.show(err.message || 'Failed to delete hook', 'error');
            });
        });

        app.events.on('click', '[data-hook-details]', (e, el) => {
            const hookId = el.getAttribute('data-hook-details');
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
                rows.forEach((row) => {
                    const name = (row.getAttribute('data-name') || '').toLowerCase();
                    const match = !query || name.includes(query);
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

        app.events.on('click', '[data-mcp-json]', (e, el) => {
            const mcpId = el.getAttribute('data-mcp-json');
            const container = document.querySelector('[data-mcp-json-container="' + mcpId + '"]');
            if (!container) return;

            if (container.style.display === 'none') {
                const script = document.querySelector('script[data-mcp-detail="' + mcpId + '"]');
                if (script) {
                    try {
                        const data = JSON.parse(script.textContent);
                        container.replaceChildren();
                        const formatted = document.createElement('div');
                        formatted.innerHTML = OrgCommon.formatJson(data);
                        container.append(...formatted.childNodes);
                    } catch (err) {
                        container.replaceChildren();
                        const span = document.createElement('span');
                        span.className = 'text-muted';
                        span.textContent = 'Failed to parse JSON';
                        container.append(span);
                    }
                }
                container.style.display = 'block';
                el.textContent = 'Hide JSON';
            } else {
                container.style.display = 'none';
                el.textContent = 'Show JSON';
            }
        });

        app.events.on('click', '[data-action="delete"][data-entity-type="mcp-server"]', (e, el) => {
            const id = el.getAttribute('data-entity-id');
            if (!confirm('Delete MCP server "' + id + '"? This cannot be undone.')) return;

            app.api('/mcp-servers/' + encodeURIComponent(id), {
                method: 'DELETE'
            }).then(() => {
                app.Toast.show('MCP server deleted', 'success');
                const row = document.querySelector('tr[data-entity-id="' + id + '"].clickable-row');
                if (row) {
                    const detail = row.nextElementSibling;
                    if (detail && detail.classList.contains('detail-row')) {
                        detail.remove();
                    }
                    row.remove();
                }
            }).catch((err) => {
                app.Toast.show(err.message || 'Failed to delete MCP server', 'error');
            });
        });

        const allPlugins = [];
        document.querySelectorAll('script[data-mcp-detail]').forEach((script) => {
            try {
                const data = JSON.parse(script.textContent);
                if (data.assigned_plugins) {
                    data.assigned_plugins.forEach((p) => {
                        if (!allPlugins.some((existing) => { return existing.id === p.id; })) {
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

        app.events.on('click', '[data-assign-mcp]', (e, el) => {
            const mcpId = el.getAttribute('data-assign-mcp');
            const mcpName = el.getAttribute('data-mcp-name') || mcpId;

            let currentPluginIds = [];
            const script = document.querySelector('script[data-mcp-detail="' + mcpId + '"]');
            if (script) {
                try {
                    const data = JSON.parse(script.textContent);
                    if (data.assigned_plugins) {
                        currentPluginIds = data.assigned_plugins.map((p) => { return p.id; });
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

    app.initAccessControl = () => {
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
        tabBar.addEventListener('click', (e) => {
            const btn = e.target.closest('[data-acl-tab]');
            if (!btn) return;
            activeTab = btn.getAttribute('data-acl-tab');
            tabBar.querySelectorAll('.sp-tab').forEach((b) => {
                const isActive = b === btn;
                b.classList.toggle('sp-tab--active', isActive);
                b.setAttribute('aria-selected', isActive ? 'true' : 'false');
            });
            document.querySelectorAll('[data-acl-panel]').forEach((panel) => {
                panel.style.display = panel.getAttribute('data-acl-panel') === activeTab ? '' : 'none';
            });
            clearSelection();
            updateCoverage();
        });
    }

    function initSearch() {
        const input = document.getElementById('acl-search');
        if (!input) return;
        input.addEventListener('input', debounce(() => {
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

        panel.querySelectorAll('.acl-entity-row').forEach((row) => {
            const name = row.getAttribute('data-name') || '';
            const matchesSearch = !query || name.includes(query);

            let matchesRole = true;
            let matchesDept = true;

            if (roleFilter || deptFilter) {
                const entityType = row.getAttribute('data-entity-type');
                const entityId = row.getAttribute('data-entity-id');
                const data = getEntityData(entityType, entityId);
                if (data) {
                    if (roleFilter) {
                        matchesRole = data.roles && data.roles.some((r) => {
                            return r.name === roleFilter && r.assigned;
                        });
                    }
                    if (deptFilter) {
                        matchesDept = data.departments && data.departments.some((d) => {
                            return d.name === deptFilter && d.assigned;
                        });
                    }
                }
            }

            row.style.display = (matchesSearch && matchesRole && matchesDept) ? '' : 'none';
        });
    }

    function initRowClicks() {
        app.events.on('click', '.acl-entity-row', (e, row) => {
            if (e.target.closest('[data-no-row-click]') || e.target.tagName === 'INPUT') return;
            const entityType = row.getAttribute('data-entity-type');
            const entityId = row.getAttribute('data-entity-id');
            openSidePanel(entityType, entityId);
        });
    }

    function initCheckboxes() {
        app.events.on('change', '.acl-select-all', (e, selectAll) => {
            const tabTarget = selectAll.getAttribute('data-acl-tab-target');
            const panel = document.querySelector('[data-acl-panel="' + tabTarget + '"]');
            if (!panel) return;
            panel.querySelectorAll('.acl-entity-checkbox').forEach((cb) => {
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

        app.events.on('change', '.acl-entity-checkbox', (e, cb) => {
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
        document.querySelectorAll('.acl-entity-checkbox, .acl-select-all').forEach((cb) => {
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
        if (body) {
            body.replaceChildren();
            body.append(buildPanelContent(data, entityType));
        }

        if (body) {
            body.querySelectorAll('input[name="department"]').forEach((cb) => {
                cb.addEventListener('change', () => {
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
        const frag = document.createDocumentFragment();

        const infoDiv = document.createElement('div');
        infoDiv.className = 'acl-entity-info';
        const primaryDiv = document.createElement('div');
        primaryDiv.className = 'cell-primary';
        primaryDiv.textContent = entity.name || entity.id;
        infoDiv.append(primaryDiv);
        if (entity.description) {
            const secondaryDiv = document.createElement('div');
            secondaryDiv.className = 'cell-secondary';
            secondaryDiv.textContent = entity.description;
            infoDiv.append(secondaryDiv);
        }
        const badgeWrap = document.createElement('div');
        badgeWrap.style.marginTop = 'var(--sp-space-2)';
        const typeBadge = document.createElement('span');
        typeBadge.className = 'badge badge-blue';
        typeBadge.textContent = entityType.replace('_', ' ');
        badgeWrap.append(typeBadge, document.createTextNode(' '));
        const statusBadge = document.createElement('span');
        if (entity.enabled) {
            statusBadge.className = 'badge badge-green';
            statusBadge.textContent = 'Active';
        } else {
            statusBadge.className = 'badge badge-gray';
            statusBadge.textContent = 'Disabled';
        }
        badgeWrap.append(statusBadge);
        infoDiv.append(badgeWrap);
        frag.append(infoDiv);

        const rolesSection = document.createElement('div');
        rolesSection.className = 'acl-panel-section';
        const rolesTitle = document.createElement('h3');
        rolesTitle.className = 'acl-panel-section-title';
        rolesTitle.textContent = 'Roles';
        const rolesDesc = document.createElement('p');
        rolesDesc.className = 'acl-panel-section-desc';
        rolesDesc.textContent = 'Select which roles can access this entity. Empty means accessible to all.';
        rolesSection.append(rolesTitle, rolesDesc);
        if (entity.roles && entity.roles.length) {
            entity.roles.forEach((role) => {
                const label = document.createElement('label');
                label.className = 'acl-checkbox-row';
                const input = document.createElement('input');
                input.type = 'checkbox';
                input.name = 'role';
                input.value = role.name;
                if (role.assigned) input.checked = true;
                const span = document.createElement('span');
                span.className = 'acl-checkbox-label';
                span.textContent = role.name;
                label.append(input, span);
                rolesSection.append(label);
            });
        } else {
            const noRoles = document.createElement('p');
            noRoles.style.cssText = 'color:var(--sp-text-tertiary);font-size:var(--sp-text-sm)';
            noRoles.textContent = 'No roles defined.';
            rolesSection.append(noRoles);
        }
        frag.append(rolesSection);

        const deptSection = document.createElement('div');
        deptSection.className = 'acl-panel-section';
        const deptTitle = document.createElement('h3');
        deptTitle.className = 'acl-panel-section-title';
        deptTitle.textContent = 'Departments';
        const deptDesc = document.createElement('p');
        deptDesc.className = 'acl-panel-section-desc';
        deptDesc.textContent = 'Assign to departments. "Default" means auto-enabled and enforced for all department members.';
        deptSection.append(deptTitle, deptDesc);
        if (entity.departments && entity.departments.length) {
            entity.departments.forEach((dept) => {
                const row = document.createElement('div');
                row.className = 'acl-dept-row';
                const label = document.createElement('label');
                label.className = 'acl-checkbox-row';
                label.style.flex = '1';
                const input = document.createElement('input');
                input.type = 'checkbox';
                input.name = 'department';
                input.value = dept.name;
                if (dept.assigned) input.checked = true;
                const span = document.createElement('span');
                span.className = 'acl-checkbox-label';
                span.textContent = dept.name + ' ';
                const countSpan = document.createElement('span');
                countSpan.className = 'acl-dept-count';
                countSpan.textContent = '(' + dept.user_count + ' members)';
                span.append(countSpan);
                label.append(input, span);
                const toggleLabel = document.createElement('label');
                toggleLabel.className = 'acl-default-toggle' + (dept.assigned ? '' : ' disabled');
                const defaultInput = document.createElement('input');
                defaultInput.type = 'checkbox';
                defaultInput.name = 'default_included';
                defaultInput.value = dept.name;
                if (dept.default_included) defaultInput.checked = true;
                if (!dept.assigned) defaultInput.disabled = true;
                const toggleSpan = document.createElement('span');
                toggleSpan.className = 'acl-toggle-label';
                toggleSpan.textContent = 'Default';
                toggleLabel.append(defaultInput, toggleSpan);
                row.append(label, toggleLabel);
                deptSection.append(row);
            });
        } else {
            const noDepts = document.createElement('p');
            noDepts.style.cssText = 'color:var(--sp-text-tertiary);font-size:var(--sp-text-sm)';
            noDepts.textContent = 'No departments found. Create users with departments first.';
            deptSection.append(noDepts);
        }
        frag.append(deptSection);

        return frag;
    }

    function savePanelRules() {
        if (!currentPanelEntity) return;

        const body = document.getElementById('acl-panel-body');
        if (!body) return;

        const rules = [];

        body.querySelectorAll('input[name="role"]:checked').forEach((cb) => {
            rules.push({
                rule_type: 'role',
                rule_value: cb.value,
                access: 'allow',
                default_included: false
            });
        });

        body.querySelectorAll('input[name="department"]:checked').forEach((cb) => {
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
        }).then(() => {
            if (app.Toast) app.Toast.show('Access rules updated', 'success');
            closeSidePanel();
            window.location.reload();
        }).catch((err) => {
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

        body.replaceChildren();

        const intro = document.createElement('p');
        intro.style.cssText = 'margin-bottom:var(--sp-space-4);color:var(--sp-text-secondary);font-size:var(--sp-text-sm)';
        const strong = document.createElement('strong');
        strong.textContent = count;
        intro.append('Applying to ', strong, ' selected entities. This will replace existing rules.');
        body.append(intro);

        const rolesSection = document.createElement('div');
        rolesSection.className = 'acl-panel-section';
        const rolesTitle = document.createElement('h3');
        rolesTitle.className = 'acl-panel-section-title';
        rolesTitle.textContent = 'Roles';
        rolesSection.append(rolesTitle);
        if (data && data.roles) {
            data.roles.forEach((role) => {
                const label = document.createElement('label');
                label.className = 'acl-checkbox-row';
                const input = document.createElement('input');
                input.type = 'checkbox';
                input.name = 'role';
                input.value = role.name;
                const span = document.createElement('span');
                span.className = 'acl-checkbox-label';
                span.textContent = role.name;
                label.append(input, span);
                rolesSection.append(label);
            });
        }
        body.append(rolesSection);

        const deptSection = document.createElement('div');
        deptSection.className = 'acl-panel-section';
        const deptSectionTitle = document.createElement('h3');
        deptSectionTitle.className = 'acl-panel-section-title';
        deptSectionTitle.textContent = 'Departments';
        deptSection.append(deptSectionTitle);
        if (data && data.departments) {
            data.departments.forEach((dept) => {
                const row = document.createElement('div');
                row.className = 'acl-dept-row';
                const label = document.createElement('label');
                label.className = 'acl-checkbox-row';
                label.style.flex = '1';
                const input = document.createElement('input');
                input.type = 'checkbox';
                input.name = 'department';
                input.value = dept.name;
                const span = document.createElement('span');
                span.className = 'acl-checkbox-label';
                span.textContent = dept.name + ' ';
                const countSpan = document.createElement('span');
                countSpan.className = 'acl-dept-count';
                countSpan.textContent = '(' + dept.user_count + ' members)';
                span.append(countSpan);
                label.append(input, span);
                const toggleLabel = document.createElement('label');
                toggleLabel.className = 'acl-default-toggle disabled';
                const defaultInput = document.createElement('input');
                defaultInput.type = 'checkbox';
                defaultInput.name = 'default_included';
                defaultInput.value = dept.name;
                defaultInput.disabled = true;
                const toggleSpan = document.createElement('span');
                toggleSpan.className = 'acl-toggle-label';
                toggleSpan.textContent = 'Default';
                toggleLabel.append(defaultInput, toggleSpan);
                row.append(label, toggleLabel);
                deptSection.append(row);
            });
        }
        body.append(deptSection);

        body.querySelectorAll('input[name="department"]').forEach((cb) => {
            cb.addEventListener('change', () => {
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
        Object.keys(selectedEntities).forEach((key) => {
            const parts = key.split(':');
            entities.push({ entity_type: parts[0], entity_id: parts[1] });
        });

        const rules = [];
        body.querySelectorAll('input[name="role"]:checked').forEach((cb) => {
            rules.push({ rule_type: 'role', rule_value: cb.value, access: 'allow', default_included: false });
        });
        body.querySelectorAll('input[name="department"]:checked').forEach((cb) => {
            const deptName = cb.value;
            const defaultCb = body.querySelector('input[name="default_included"][value="' + deptName + '"]');
            rules.push({
                rule_type: 'department',
                rule_value: deptName,
                access: 'allow',
                default_included: defaultCb ? defaultCb.checked : false
            });
        });

        const hasPlugins = entities.some((e) => { return e.entity_type === 'plugin'; });

        const applyBtn = document.getElementById('acl-bulk-apply');
        if (applyBtn) {
            applyBtn.disabled = true;
            applyBtn.textContent = 'Applying...';
        }

        app.api('/access-control/bulk', {
            method: 'PUT',
            body: JSON.stringify({ entities: entities, rules: rules, sync_yaml: hasPlugins })
        }).then(() => {
            if (app.Toast) app.Toast.show('Bulk assign complete', 'success');
            closeBulkPanel();
            window.location.reload();
        }).catch((err) => {
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
        rows.forEach((r) => {
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

    function debounce(fn, ms) {
        let timer;
        return function() {
            clearTimeout(timer);
            const args = arguments;
            const ctx = this;
            timer = setTimeout(() => { fn.apply(ctx, args); }, ms);
        };
    }

})(window.AdminApp);

(function(app) {
    'use strict';

    const mktFetch = (url, opts = {}) => fetch(url, { credentials: 'include', ...opts });

    app.initOrgMarketplaces = () => {
        const searchInput = document.getElementById('mkt-search');
        const deptFilter = document.getElementById('mkt-dept-filter');
        const table = document.getElementById('mkt-table');

        if (window.AdminApp.OrgCommon) {
            AdminApp.OrgCommon.initExpandRows('#mkt-table');
        }

        const mktDepts = {};
        document.querySelectorAll('script[data-marketplace-detail]').forEach((el) => {
            try {
                const data = JSON.parse(el.textContent);
                const id = el.getAttribute('data-marketplace-detail');
                mktDepts[id] = (data.departments || [])
                    .filter((d) => d.assigned)
                    .map((d) => d.name);
            } catch (e) {}
        });

        const filterRows = () => {
            if (!table) return;
            const query = (searchInput ? searchInput.value : '').toLowerCase();
            const dept = deptFilter ? deptFilter.value : '';
            const rows = table.querySelectorAll('tbody tr.clickable-row');
            rows.forEach((row) => {
                const name = row.getAttribute('data-name') || '';
                const entityId = row.getAttribute('data-entity-id') || '';
                const matchName = !query || name.includes(query);
                const matchDept = !dept || (mktDepts[entityId] && mktDepts[entityId].includes(dept));
                const visible = matchName && matchDept;
                row.style.display = visible ? '' : 'none';
                const detailRow = table.querySelector('tr[data-detail-for="' + entityId + '"]');
                if (detailRow && !visible) {
                    detailRow.classList.remove('visible');
                    detailRow.style.display = 'none';
                }
            });
        };

        if (searchInput) {
            let debounceTimer;
            searchInput.addEventListener('input', () => {
                clearTimeout(debounceTimer);
                debounceTimer = setTimeout(filterRows, 200);
            });
        }
        if (deptFilter) {
            deptFilter.addEventListener('change', filterRows);
        }

        app.events.on('click', '[data-toggle-json]', (e, jsonBtn) => {
            const id = jsonBtn.getAttribute('data-toggle-json');
            const container = document.querySelector('[data-json-container="' + id + '"]');
            if (container) {
                if (container.style.display === 'none') {
                    const dataEl = document.querySelector('script[data-marketplace-detail="' + id + '"]');
                    if (dataEl) {
                        try {
                            const data = JSON.parse(dataEl.textContent);
                            container.replaceChildren();
                            if (AdminApp.OrgCommon) {
                                container.append(AdminApp.OrgCommon.formatJson(data));
                            } else {
                                const pre = document.createElement('pre');
                                pre.textContent = JSON.stringify(data, null, 2);
                                container.append(pre);
                            }
                        } catch (err) {
                            container.replaceChildren();
                            const errP = document.createElement('p');
                            errP.textContent = 'Error parsing JSON';
                            container.append(errP);
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

        app.events.on('click', '.actions-trigger', (e, trigger) => {
            const menu = trigger.closest('.actions-menu');
            const dd = menu ? menu.querySelector('.actions-dropdown') : null;
            if (dd) {
                const isOpen = dd.classList.contains('open');
                app.shared.closeAllMenus();
                if (!isOpen) dd.classList.add('open');
            }
        });

        app.events.on('click', '[data-delete-marketplace]', (e, deleteBtn) => {
            const id = deleteBtn.getAttribute('data-delete-marketplace');
            showDeleteConfirm(id);
        });

        app.events.on('click', '[data-copy-install-link]', (e, btn) => {
            const id = btn.getAttribute('data-copy-install-link');
            const siteUrl = window.location.origin;
            const installUrl = siteUrl + '/api/public/marketplace/org/' + encodeURIComponent(id) + '.git';
            navigator.clipboard.writeText(installUrl).then(() => {
                app.Toast.show('Install link copied to clipboard', 'success');
            }).catch(() => {
                const textarea = document.createElement('textarea');
                textarea.value = installUrl;
                document.body.append(textarea);
                textarea.select();
                document.execCommand('copy');
                textarea.remove();
                app.Toast.show('Install link copied to clipboard', 'success');
            });
            app.shared.closeAllMenus();
        });

        app.events.on('click', '[data-sync-marketplace]', (e, btn) => {
            const id = btn.getAttribute('data-sync-marketplace');
            btn.disabled = true;
            const origText = btn.textContent;
            btn.textContent = 'Syncing...';
            app.shared.closeAllMenus();
            mktFetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id) + '/sync', {
                method: 'POST'
            })
            .then((resp) => resp.json().then((data) => ({ ok: resp.ok, data: data })))
            .then((result) => {
                if (result.ok) {
                    let msg = 'Sync completed: ' + (result.data.plugins_synced || 0) + ' plugins';
                    if (!result.data.changed) msg = 'Already up to date';
                    app.Toast.show(msg, 'success');
                    if (result.data.changed) setTimeout(() => { window.location.reload(); }, 1000);
                } else {
                    app.Toast.show(result.data.error || 'Sync failed', 'error');
                }
                btn.disabled = false;
                btn.textContent = origText;
            })
            .catch(() => {
                app.Toast.show('Network error during sync', 'error');
                btn.disabled = false;
                btn.textContent = origText;
            });
        });

        app.events.on('click', '[data-publish-marketplace]', (e, btn) => {
            const id = btn.getAttribute('data-publish-marketplace');
            app.shared.closeAllMenus();
            const overlay = document.createElement('div');
            overlay.className = 'confirm-overlay';
            const pubDialog = document.createElement('div');
            pubDialog.className = 'confirm-dialog';
            const pubH3 = document.createElement('h3');
            pubH3.style.margin = '0 0 var(--sp-space-3)';
            pubH3.textContent = 'Publish to GitHub?';
            const pubP = document.createElement('p');
            pubP.style.cssText = 'margin:0 0 var(--sp-space-4);color:var(--sp-text-secondary);font-size:var(--sp-text-sm)';
            pubP.textContent = 'This will push the current marketplace plugins to the linked GitHub repository. Any remote changes will be overwritten.';
            const pubBtnRow = document.createElement('div');
            pubBtnRow.style.cssText = 'display:flex;gap:var(--sp-space-3);justify-content:flex-end';
            const pubCancelBtn = document.createElement('button');
            pubCancelBtn.className = 'btn btn-secondary';
            pubCancelBtn.setAttribute('data-confirm-cancel', '');
            pubCancelBtn.textContent = 'Cancel';
            const pubConfirmBtn = document.createElement('button');
            pubConfirmBtn.className = 'btn btn-primary';
            pubConfirmBtn.setAttribute('data-confirm-publish', '');
            pubConfirmBtn.textContent = 'Publish';
            pubBtnRow.append(pubCancelBtn, pubConfirmBtn);
            pubDialog.append(pubH3, pubP, pubBtnRow);
            overlay.append(pubDialog);
            document.body.append(overlay);
            overlay.addEventListener('click', (ev) => {
                if (ev.target === overlay || ev.target.closest('[data-confirm-cancel]')) {
                    overlay.remove();
                    return;
                }
                const pubBtn = ev.target.closest('[data-confirm-publish]');
                if (pubBtn) {
                    pubBtn.disabled = true;
                    pubBtn.textContent = 'Publishing...';
                    mktFetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id) + '/publish', {
                        method: 'POST'
                    })
                    .then((resp) => resp.json().then((data) => ({ ok: resp.ok, data: data })))
                    .then((result) => {
                        overlay.remove();
                        if (result.ok) {
                            let msg = 'Published: ' + (result.data.plugins_synced || 0) + ' plugins';
                            if (!result.data.changed) msg = 'No changes to publish';
                            app.Toast.show(msg, 'success');
                            if (result.data.changed) setTimeout(() => { window.location.reload(); }, 1000);
                        } else {
                            app.Toast.show(result.data.error || 'Publish failed', 'error');
                        }
                    })
                    .catch(() => {
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
        const delDialog = document.createElement('div');
        delDialog.className = 'confirm-dialog';
        const delH3 = document.createElement('h3');
        delH3.style.margin = '0 0 var(--sp-space-3)';
        delH3.textContent = 'Delete Marketplace?';
        const delP1 = document.createElement('p');
        delP1.style.cssText = 'margin:0 0 var(--sp-space-2);color:var(--sp-text-secondary);font-size:var(--sp-text-sm)';
        delP1.append(document.createTextNode('You are about to delete '));
        const delStrong = document.createElement('strong');
        delStrong.textContent = marketplaceId;
        delP1.append(delStrong, document.createTextNode('.'));
        const delP2 = document.createElement('p');
        delP2.style.cssText = 'margin:0 0 var(--sp-space-5);color:var(--sp-text-secondary);font-size:var(--sp-text-sm)';
        delP2.textContent = 'This will remove the marketplace and all plugin associations. This action cannot be undone.';
        const delBtnRow = document.createElement('div');
        delBtnRow.style.cssText = 'display:flex;gap:var(--sp-space-3);justify-content:flex-end';
        const delCancelBtn = document.createElement('button');
        delCancelBtn.className = 'btn btn-secondary';
        delCancelBtn.setAttribute('data-confirm-cancel', '');
        delCancelBtn.textContent = 'Cancel';
        const delConfirmBtn = document.createElement('button');
        delConfirmBtn.className = 'btn btn-danger';
        delConfirmBtn.setAttribute('data-confirm-delete', marketplaceId);
        delConfirmBtn.textContent = 'Delete Marketplace';
        delBtnRow.append(delCancelBtn, delConfirmBtn);
        delDialog.append(delH3, delP1, delP2, delBtnRow);
        overlay.append(delDialog);
        document.body.append(overlay);

        overlay.addEventListener('click', async (e) => {
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
                    const resp = await mktFetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id), {
                        method: 'DELETE'
                    });
                    if (resp.ok) {
                        app.Toast.show('Marketplace deleted', 'success');
                        setTimeout(() => { window.location.reload(); }, 500);
                    } else {
                        const data = await (resp.headers.get('content-type')?.includes('json') ? resp.json() : Promise.resolve({}));
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

        app.events.on('click', '[data-manage-plugins]', (e, btn) => {
            const id = btn.getAttribute('data-manage-plugins');

            const dataEl = document.querySelector('script[data-marketplace-detail="' + id + '"]');
            if (!dataEl) return;
            let mktData;
            try { mktData = JSON.parse(dataEl.textContent); } catch (err) { return; }

            panelApi.setTitle('Manage Plugins - ' + (mktData.name || id));

            mktFetch(app.API_BASE + '/plugins')
                .then((r) => r.json())
                .then((allPlugins) => {
                    const currentIds = {};
                    (mktData.plugin_ids || []).forEach((pid) => { currentIds[pid] = true; });

                    const checklist = document.createElement('div');
                    checklist.className = 'assign-panel-checklist';
                    if (!allPlugins.length) {
                        const noPlugins = document.createElement('p');
                        noPlugins.style.cssText = 'color:var(--sp-text-tertiary);font-size:var(--sp-text-sm)';
                        noPlugins.textContent = 'No plugins available.';
                        checklist.append(noPlugins);
                    } else {
                        allPlugins.forEach(function(p) {
                            const pid = p.id || p.plugin_id;
                            const pname = p.name || pid;
                            const label = document.createElement('label');
                            label.className = 'acl-checkbox-row';
                            label.style.cssText = 'display:flex;align-items:center;gap:var(--sp-space-2);padding:var(--sp-space-1) 0;cursor:pointer';
                            const cb = document.createElement('input');
                            cb.type = 'checkbox';
                            cb.name = 'plugin_id';
                            cb.value = pid;
                            if (currentIds[pid]) cb.checked = true;
                            const nameSpan = document.createElement('span');
                            nameSpan.textContent = pname;
                            label.append(cb, nameSpan);
                            checklist.append(label);
                        });
                    }
                    panelApi.setBodyDom(checklist);

                    const footerFrag = document.createDocumentFragment();
                    const panelCancelBtn = document.createElement('button');
                    panelCancelBtn.className = 'btn btn-secondary';
                    panelCancelBtn.setAttribute('data-panel-close', '');
                    panelCancelBtn.textContent = 'Cancel';
                    const panelSaveBtn = document.createElement('button');
                    panelSaveBtn.className = 'btn btn-primary';
                    panelSaveBtn.id = 'mkt-save-plugins';
                    panelSaveBtn.textContent = 'Save';
                    footerFrag.append(panelCancelBtn, document.createTextNode(' '), panelSaveBtn);
                    panelApi.setFooterDom(footerFrag);

                    const footer = panelApi.panel.querySelector('[data-panel-footer]');
                    if (footer) {
                        const cancelBtn = footer.querySelector('[data-panel-close]');
                        if (cancelBtn) cancelBtn.addEventListener('click', panelApi.close);
                    }

                    const saveBtn = document.getElementById('mkt-save-plugins');
                    if (saveBtn) {
                        saveBtn.addEventListener('click', async () => {
                            const checked = panelApi.panel.querySelectorAll('input[name="plugin_id"]:checked');
                            const ids = [];
                            checked.forEach((cb) => { ids.push(cb.value); });
                            saveBtn.disabled = true;
                            saveBtn.textContent = 'Saving...';
                            try {
                                const resp = await mktFetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id) + '/plugins', {
                                    method: 'PUT',
                                    headers: { 'Content-Type': 'application/json' },
                                    body: JSON.stringify({ plugin_ids: ids })
                                });
                                if (resp.ok) {
                                    app.Toast.show('Plugins updated', 'success');
                                    panelApi.close();
                                    setTimeout(() => { window.location.reload(); }, 500);
                                } else {
                                    const data = await (resp.headers.get('content-type')?.includes('json') ? resp.json() : Promise.resolve({}));
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
                .catch(() => {
                    app.Toast.show('Failed to load plugins', 'error');
                });
        });
    }

    function initEditPanel() {
        if (!window.AdminApp.OrgCommon) return;
        const panelApi = AdminApp.OrgCommon.initSidePanel('mkt-edit-panel');
        if (!panelApi) return;

        const readJsonEl = (id) => {
            const el = document.getElementById(id);
            if (!el) return [];
            try { return JSON.parse(el.textContent); } catch (e) { return []; }
        };

        const allRoles = readJsonEl('mkt-all-roles');
        const allDepts = readJsonEl('mkt-all-departments');

        const openEdit = (marketplaceId) => {
            const isEdit = !!marketplaceId;
            let mktData = {};

            if (isEdit) {
                const dataEl = document.querySelector('script[data-marketplace-detail="' + marketplaceId + '"]');
                if (dataEl) {
                    try { mktData = JSON.parse(dataEl.textContent); } catch (e) {}
                }
            }

            panelApi.setTitle(isEdit ? 'Edit Marketplace' : 'Create Marketplace');

            mktFetch(app.API_BASE + '/plugins')
                .then((r) => r.json())
                .then((allPlugins) => {
                    const currentPluginIds = {};
                    (mktData.plugin_ids || []).forEach((pid) => { currentPluginIds[pid] = true; });

                    const currentRoles = {};
                    (mktData.roles || []).forEach((r) => {
                        if (r.assigned) currentRoles[r.name] = true;
                    });

                    const currentDepts = {};
                    const deptDefaults = {};
                    (mktData.departments || []).forEach((d) => {
                        if (d.assigned) {
                            currentDepts[d.name] = true;
                            deptDefaults[d.name] = d.default_included;
                        }
                    });

                    const form = document.createElement('form');
                    form.id = 'panel-edit-form';

                    function makeFormGroup(labelText, inputEl) {
                        const group = document.createElement('div');
                        group.className = 'form-group';
                        const lbl = document.createElement('label');
                        lbl.className = 'field-label';
                        lbl.textContent = labelText;
                        group.append(lbl, inputEl);
                        return group;
                    }

                    if (!isEdit) {
                        const idInput = document.createElement('input');
                        idInput.type = 'text';
                        idInput.className = 'field-input';
                        idInput.name = 'marketplace_id';
                        idInput.required = true;
                        idInput.placeholder = 'e.g. my-marketplace';
                        form.append(makeFormGroup('Marketplace ID', idInput));
                    }

                    const nameInput = document.createElement('input');
                    nameInput.type = 'text';
                    nameInput.className = 'field-input';
                    nameInput.name = 'name';
                    nameInput.required = true;
                    nameInput.value = mktData.name || '';
                    form.append(makeFormGroup('Name', nameInput));

                    const descTextarea = document.createElement('textarea');
                    descTextarea.className = 'field-input';
                    descTextarea.name = 'description';
                    descTextarea.rows = 3;
                    descTextarea.textContent = mktData.description || '';
                    form.append(makeFormGroup('Description', descTextarea));

                    const ghGroup = document.createElement('div');
                    ghGroup.className = 'form-group';
                    const ghLabel = document.createElement('label');
                    ghLabel.className = 'field-label';
                    ghLabel.textContent = 'GitHub Repository URL';
                    const ghInput = document.createElement('input');
                    ghInput.type = 'url';
                    ghInput.className = 'field-input';
                    ghInput.name = 'github_repo_url';
                    ghInput.placeholder = 'https://github.com/org/repo';
                    ghInput.value = mktData.github_repo_url || '';
                    const ghHint = document.createElement('span');
                    ghHint.className = 'field-hint';
                    ghHint.textContent = 'Link to a GitHub repository to enable sync/publish';
                    ghGroup.append(ghLabel, ghInput, ghHint);
                    form.append(ghGroup);

                    const rolesGroup = document.createElement('div');
                    rolesGroup.className = 'form-group';
                    const rolesLabel = document.createElement('label');
                    rolesLabel.className = 'field-label';
                    rolesLabel.textContent = 'Roles';
                    const rolesWrap = document.createElement('div');
                    rolesWrap.style.cssText = 'display:flex;flex-wrap:wrap;gap:var(--sp-space-1);padding:var(--sp-space-2) 0';
                    allRoles.forEach(function(r) {
                        const val = r.value || r;
                        const rLabel = document.createElement('label');
                        rLabel.style.cssText = 'display:inline-flex;align-items:center;gap:var(--sp-space-2);margin-right:var(--sp-space-3);font-size:var(--sp-text-sm);cursor:pointer';
                        const rCb = document.createElement('input');
                        rCb.type = 'checkbox';
                        rCb.name = 'roles';
                        rCb.value = val;
                        if (currentRoles[val]) rCb.checked = true;
                        rLabel.append(rCb, document.createTextNode(' ' + val));
                        rolesWrap.append(rLabel);
                    });
                    rolesGroup.append(rolesLabel, rolesWrap);
                    form.append(rolesGroup);

                    const deptGroup = document.createElement('div');
                    deptGroup.className = 'form-group';
                    const deptLabel = document.createElement('label');
                    deptLabel.className = 'field-label';
                    deptLabel.textContent = 'Departments';
                    const deptContainer = document.createElement('div');
                    deptContainer.className = 'checklist-container';
                    deptContainer.style.cssText = 'max-height:300px;overflow-y:auto;border:1px solid var(--sp-border-subtle);border-radius:var(--sp-radius-md);padding:var(--sp-space-2)';

                    const checkAllItem = document.createElement('div');
                    checkAllItem.className = 'checklist-item';
                    checkAllItem.style.cssText = 'display:flex;align-items:center;gap:var(--sp-space-2);padding:var(--sp-space-2);border-bottom:1px solid var(--sp-border-subtle)';
                    const checkAllCb = document.createElement('input');
                    checkAllCb.type = 'checkbox';
                    checkAllCb.id = 'panel-dept-check-all';
                    const checkAllLabel = document.createElement('label');
                    checkAllLabel.htmlFor = 'panel-dept-check-all';
                    checkAllLabel.style.cssText = 'flex:1;font-size:var(--sp-text-sm);cursor:pointer;color:var(--sp-text-primary);font-weight:600';
                    checkAllLabel.textContent = 'Check all';
                    checkAllItem.append(checkAllCb, checkAllLabel);
                    deptContainer.append(checkAllItem);

                    allDepts.forEach(function(d, i) {
                        const val = d.value || d.name || d;
                        const dItem = document.createElement('div');
                        dItem.className = 'checklist-item';
                        dItem.style.cssText = 'display:flex;align-items:center;gap:var(--sp-space-2);padding:var(--sp-space-2)';
                        const dCb = document.createElement('input');
                        dCb.type = 'checkbox';
                        dCb.name = 'departments';
                        dCb.value = val;
                        if (currentDepts[val]) dCb.checked = true;
                        dCb.id = 'panel-dept-' + i;
                        const dLabel = document.createElement('label');
                        dLabel.htmlFor = 'panel-dept-' + i;
                        dLabel.style.cssText = 'flex:1;font-size:var(--sp-text-sm);cursor:pointer;color:var(--sp-text-primary)';
                        dLabel.textContent = val;
                        const countBadge = document.createElement('span');
                        countBadge.className = 'badge badge-gray';
                        countBadge.style.fontSize = 'var(--sp-text-xs)';
                        countBadge.textContent = (d.user_count || 0) + ' users';
                        const defaultLabel = document.createElement('label');
                        defaultLabel.style.cssText = 'display:inline-flex;align-items:center;gap:4px;font-size:var(--sp-text-xs);color:var(--sp-text-secondary);cursor:pointer;white-space:nowrap';
                        const defaultCb = document.createElement('input');
                        defaultCb.type = 'checkbox';
                        defaultCb.name = 'dept_default_' + val;
                        if (deptDefaults[val]) defaultCb.checked = true;
                        defaultLabel.append(defaultCb, document.createTextNode(' Default'));
                        dItem.append(dCb, dLabel, countBadge, defaultLabel);
                        deptContainer.append(dItem);
                    });

                    const deptHint = document.createElement('span');
                    deptHint.className = 'field-hint';
                    deptHint.style.cssText = 'margin-top:var(--sp-space-2);display:block';
                    deptHint.textContent = 'At least one department is required.';
                    deptGroup.append(deptLabel, deptContainer, deptHint);
                    form.append(deptGroup);

                    const pluginGroup = document.createElement('div');
                    pluginGroup.className = 'form-group';
                    const pluginLabel = document.createElement('label');
                    pluginLabel.className = 'field-label';
                    pluginLabel.textContent = 'Plugins';
                    const pluginFilterInput = document.createElement('input');
                    pluginFilterInput.type = 'text';
                    pluginFilterInput.className = 'field-input';
                    pluginFilterInput.placeholder = 'Filter plugins...';
                    pluginFilterInput.id = 'panel-plugin-filter';
                    pluginFilterInput.style.marginBottom = 'var(--sp-space-2)';
                    const pluginContainer = document.createElement('div');
                    pluginContainer.className = 'checklist-container';
                    pluginContainer.style.cssText = 'max-height:200px;overflow-y:auto;border:1px solid var(--sp-border-subtle);border-radius:var(--sp-radius-md);padding:var(--sp-space-2)';
                    allPlugins.forEach(function(p, i) {
                        const pid = p.id || p.plugin_id;
                        const pname = p.name || pid;
                        const pItem = document.createElement('div');
                        pItem.className = 'checklist-item';
                        pItem.setAttribute('data-item-name', pname.toLowerCase());
                        const pCb = document.createElement('input');
                        pCb.type = 'checkbox';
                        pCb.name = 'plugin_ids';
                        pCb.value = pid;
                        if (currentPluginIds[pid]) pCb.checked = true;
                        pCb.id = 'panel-plugin-' + i;
                        const pLabel = document.createElement('label');
                        pLabel.htmlFor = 'panel-plugin-' + i;
                        pLabel.textContent = pname;
                        pItem.append(pCb, pLabel);
                        pluginContainer.append(pItem);
                    });
                    pluginGroup.append(pluginLabel, pluginFilterInput, pluginContainer);
                    form.append(pluginGroup);

                    panelApi.setBodyDom(form);

                    const editFooter = document.createDocumentFragment();
                    if (isEdit) {
                        const editDelBtn = document.createElement('button');
                        editDelBtn.className = 'btn btn-danger';
                        editDelBtn.id = 'mkt-edit-delete';
                        editDelBtn.style.marginRight = 'auto';
                        editDelBtn.textContent = 'Delete';
                        editFooter.append(editDelBtn, document.createTextNode(' '));
                    }
                    const editCancelBtn = document.createElement('button');
                    editCancelBtn.className = 'btn btn-secondary';
                    editCancelBtn.setAttribute('data-panel-close', '');
                    editCancelBtn.textContent = 'Cancel';
                    const editSaveBtn = document.createElement('button');
                    editSaveBtn.className = 'btn btn-primary';
                    editSaveBtn.id = 'mkt-edit-save';
                    editSaveBtn.textContent = isEdit ? 'Save Changes' : 'Create Marketplace';
                    editFooter.append(editCancelBtn, document.createTextNode(' '), editSaveBtn);
                    panelApi.setFooterDom(editFooter);

                    const footer = panelApi.panel.querySelector('[data-panel-footer]');
                    if (footer) {
                        const cancelBtn = footer.querySelector('[data-panel-close]');
                        if (cancelBtn) cancelBtn.addEventListener('click', panelApi.close);
                    }

                    const checkAll = document.getElementById('panel-dept-check-all');
                    if (checkAll) {
                        checkAll.addEventListener('change', () => {
                            const boxes = panelApi.panel.querySelectorAll('input[name="departments"]');
                            boxes.forEach((cb) => { cb.checked = checkAll.checked; });
                        });
                        const boxes = panelApi.panel.querySelectorAll('input[name="departments"]');
                        let allChecked = boxes.length > 0;
                        boxes.forEach((cb) => { if (!cb.checked) allChecked = false; });
                        checkAll.checked = allChecked;
                        panelApi.panel.addEventListener('change', (e) => {
                            if (e.target.name === 'departments') {
                                const boxes = panelApi.panel.querySelectorAll('input[name="departments"]');
                                let all = boxes.length > 0;
                                boxes.forEach((cb) => { if (!cb.checked) all = false; });
                                checkAll.checked = all;
                            }
                        });
                    }

                    const pluginFilter = document.getElementById('panel-plugin-filter');
                    if (pluginFilter) {
                        pluginFilter.addEventListener('input', () => {
                            const q = pluginFilter.value.toLowerCase();
                            panelApi.panel.querySelectorAll('.checklist-item[data-item-name]').forEach((item) => {
                                const name = item.getAttribute('data-item-name') || '';
                                item.style.display = (!q || name.includes(q)) ? '' : 'none';
                            });
                        });
                    }

                    const saveBtn = document.getElementById('mkt-edit-save');
                    if (saveBtn) {
                        saveBtn.addEventListener('click', () => {
                            handlePanelSave(isEdit, marketplaceId, saveBtn, panelApi);
                        });
                    }

                    const deleteBtn = document.getElementById('mkt-edit-delete');
                    if (deleteBtn) {
                        deleteBtn.addEventListener('click', () => {
                            panelApi.close();
                            showDeleteConfirm(marketplaceId);
                        });
                    }

                    panelApi.open();
                })
                .catch(() => {
                    app.Toast.show('Failed to load plugins', 'error');
                });
        };

        app.events.on('click', '[data-edit-marketplace]', (e, btn) => {
            openEdit(btn.getAttribute('data-edit-marketplace'));
        });

        app.events.on('click', '[data-create-marketplace]', (e, btn) => {
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
        form.querySelectorAll('input[name="plugin_ids"]:checked').forEach((cb) => { pluginIds.push(cb.value); });

        const selectedRoles = [];
        form.querySelectorAll('input[name="roles"]:checked').forEach((cb) => { selectedRoles.push(cb.value); });

        const deptRules = [];
        form.querySelectorAll('input[name="departments"]').forEach((cb) => {
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
        selectedRoles.forEach((role) => {
            aclRules.push({ rule_type: 'role', rule_value: role, access: 'allow', default_included: false });
        });
        aclRules = aclRules.concat(deptRules);

        const githubUrlInput = form.querySelector('input[name="github_repo_url"]');
        const githubUrl = githubUrlInput ? githubUrlInput.value.trim() : '';

        try {
            if (isEdit) {
                const body = {
                    name: form.querySelector('input[name="name"]').value,
                    description: form.querySelector('textarea[name="description"]').value,
                    github_repo_url: githubUrl || null,
                    plugin_ids: pluginIds
                };
                const resp = await mktFetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(marketplaceId), {
                    method: 'PUT',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(body)
                });
                if (resp.ok) {
                    await mktFetch(app.API_BASE + '/access-control/entity/marketplace/' + encodeURIComponent(marketplaceId), {
                        method: 'PUT',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ rules: aclRules, sync_yaml: false })
                    });
                    app.Toast.show('Marketplace updated', 'success');
                    panelApi.close();
                    setTimeout(() => { window.location.reload(); }, 500);
                } else {
                    const data = await (resp.headers.get('content-type')?.includes('json') ? resp.json() : Promise.resolve({}));
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
                const resp = await mktFetch(app.API_BASE + '/org/marketplaces', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(body)
                });
                if (resp.ok || resp.status === 201) {
                    const created = await (resp.headers.get('content-type')?.includes('json') ? resp.json() : Promise.resolve({}));
                    const createdId = created.id || body.id;
                    if (aclRules.length > 0 && createdId) {
                        await mktFetch(app.API_BASE + '/access-control/entity/marketplace/' + encodeURIComponent(createdId), {
                            method: 'PUT',
                            headers: { 'Content-Type': 'application/json' },
                            body: JSON.stringify({ rules: aclRules, sync_yaml: false })
                        });
                    }
                    app.Toast.show('Marketplace created', 'success');
                    panelApi.close();
                    setTimeout(() => { window.location.reload(); }, 500);
                } else {
                    const data = await (resp.headers.get('content-type')?.includes('json') ? resp.json() : Promise.resolve({}));
                    app.Toast.show(data.error || 'Failed to create', 'error');
                }
            }
        } catch (err) {
            app.Toast.show('Network error', 'error');
        }

        saveBtn.disabled = false;
        saveBtn.textContent = isEdit ? 'Save Changes' : 'Create Marketplace';
    }

    app.initMarketplaceEditForm = () => {
        const form = document.getElementById('marketplace-edit-form');
        if (!form) return;

        const isEdit = !!form.querySelector('input[name="marketplace_id"][readonly]');

        form.addEventListener('submit', async (e) => {
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
            pluginCheckboxes.forEach((cb) => { pluginIds.push(cb.value); });

            const roleCheckboxes = form.querySelectorAll('input[name="roles"]:checked');
            const selectedRoles = [];
            roleCheckboxes.forEach((cb) => { selectedRoles.push(cb.value); });

            const deptCheckboxes = form.querySelectorAll('input[name="departments"]');
            const deptRules = [];
            deptCheckboxes.forEach((cb) => {
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
            selectedRoles.forEach((role) => {
                aclRules.push({ rule_type: 'role', rule_value: role, access: 'allow', default_included: false });
            });
            aclRules = aclRules.concat(deptRules);

            const formGithubInput = form.querySelector('input[name="github_repo_url"]');
            const formGithubUrl = formGithubInput ? formGithubInput.value.trim() : '';

            if (isEdit) {
                const id = form.querySelector('input[name="marketplace_id"]').value;
                const body = {
                    name: form.querySelector('input[name="name"]').value,
                    description: form.querySelector('textarea[name="description"]').value,
                    github_repo_url: formGithubUrl || null,
                    plugin_ids: pluginIds
                };

                try {
                    const resp = await mktFetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id), {
                        method: 'PUT',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(body)
                    });
                    if (resp.ok) {
                        await mktFetch(app.API_BASE + '/access-control/entity/marketplace/' + encodeURIComponent(id), {
                            method: 'PUT',
                            headers: { 'Content-Type': 'application/json' },
                            body: JSON.stringify({ rules: aclRules, sync_yaml: false })
                        });
                        app.Toast.show('Marketplace updated', 'success');
                        setTimeout(() => { window.location.href = '/admin/org/marketplaces/'; }, 500);
                    } else {
                        const data = await (resp.headers.get('content-type')?.includes('json') ? resp.json() : Promise.resolve({}));
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
                    const resp = await mktFetch(app.API_BASE + '/org/marketplaces', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(body)
                    });
                    if (resp.ok || resp.status === 201) {
                        const created = await (resp.headers.get('content-type')?.includes('json') ? resp.json() : Promise.resolve({}));
                        const createdId = created.id || body.id;
                        if (aclRules.length > 0 && createdId) {
                            await mktFetch(app.API_BASE + '/access-control/entity/marketplace/' + encodeURIComponent(createdId), {
                                method: 'PUT',
                                headers: { 'Content-Type': 'application/json' },
                                body: JSON.stringify({ rules: aclRules, sync_yaml: false })
                            });
                        }
                        app.Toast.show('Marketplace created', 'success');
                        setTimeout(() => { window.location.href = '/admin/org/marketplaces/'; }, 500);
                    } else {
                        const data = await (resp.headers.get('content-type')?.includes('json') ? resp.json() : Promise.resolve({}));
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
            deleteBtn.addEventListener('click', () => {
                const idInput = form.querySelector('input[name="marketplace_id"]');
                if (idInput) showDeleteConfirm(idInput.value);
            });
        }

        const checkAllDept = form.querySelector('#dept-check-all');
        if (checkAllDept) {
            checkAllDept.addEventListener('change', () => {
                const boxes = form.querySelectorAll('input[name="departments"]');
                boxes.forEach((cb) => { cb.checked = checkAllDept.checked; });
            });
            form.addEventListener('change', (e) => {
                if (e.target.name === 'departments') {
                    const boxes = form.querySelectorAll('input[name="departments"]');
                    let allChecked = boxes.length > 0;
                    boxes.forEach((cb) => { if (!cb.checked) allChecked = false; });
                    checkAllDept.checked = allChecked;
                }
            });
            const boxes = form.querySelectorAll('input[name="departments"]');
            let allChecked = boxes.length > 0;
            boxes.forEach((cb) => { if (!cb.checked) allChecked = false; });
            checkAllDept.checked = allChecked;
        }
    };

})(window.AdminApp = window.AdminApp || {});

(function(app) {
    'use strict';

    const MyCommon = {

        initExpandRows: (tableSelector, renderCallback) => {
            const table = document.querySelector(tableSelector);
            if (!table) return;

            table.addEventListener('click', (e) => {
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

                MyCommon.handleRowClick(row, detailRow);

                if (renderCallback && detailRow.classList.contains('visible')) {
                    renderCallback(row, detailRow);
                }
            });
        },

        handleRowClick: (row, detailRow) => {
            const isVisible = detailRow.classList.contains('visible');

            const table = row.closest('table');
            if (table) {
                table.querySelectorAll('tr.detail-row.visible').forEach((r) => {
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

        initSidePanel: (panelId) => {
            const panel = document.getElementById(panelId);
            if (!panel) return null;

            const overlayId = panel.getAttribute('data-overlay') || (panelId + '-overlay');
            const overlay = document.getElementById(overlayId);
            const closeBtn = panel.querySelector('[data-panel-close]');

            const api = {
                open: () => {
                    panel.classList.add('open');
                    if (overlay) overlay.classList.add('active');
                },
                close: () => {
                    panel.classList.remove('open');
                    if (overlay) overlay.classList.remove('active');
                },
                setTitle: (text) => {
                    const title = panel.querySelector('[data-panel-title]');
                    if (title) title.textContent = text;
                },
                setBodyText: (text) => {
                    const body = panel.querySelector('[data-panel-body]');
                    if (!body) return;
                    body.replaceChildren();
                    const p = document.createElement('p');
                    p.style.cssText = 'color:var(--sp-text-tertiary);text-align:center;padding:var(--sp-space-4)';
                    p.textContent = text;
                    body.append(p);
                },
                setBodyDom: (el) => {
                    const body = panel.querySelector('[data-panel-body]');
                    if (!body) return;
                    body.replaceChildren();
                    body.append(el);
                },
                setFooterDom: (el) => {
                    const footer = panel.querySelector('[data-panel-footer]');
                    if (!footer) return;
                    footer.replaceChildren();
                    if (el) footer.append(el);
                },
                panel: panel
            };

            if (closeBtn) closeBtn.addEventListener('click', api.close);
            if (overlay) overlay.addEventListener('click', api.close);

            return api;
        },

        initBulkActions: (tableSelector, barId) => {
            const table = document.querySelector(tableSelector);
            if (!table) return null;

            let selected = {};

            const updateCount = () => {
                const count = Object.keys(selected).length;
                const countEl = document.querySelector('[data-bulk-count]');
                if (countEl) countEl.textContent = count;
                const bar = document.getElementById(barId);
                if (bar) bar.style.display = count > 0 ? 'flex' : 'none';
            };

            table.addEventListener('change', (e) => {
                if (e.target.classList.contains('bulk-select-all')) {
                    const checked = e.target.checked;
                    table.querySelectorAll('.bulk-checkbox').forEach((cb) => {
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
                getSelected: () => { return Object.keys(selected); },
                clear: () => {
                    selected = {};
                    table.querySelectorAll('.bulk-checkbox, .bulk-select-all').forEach((cb) => {
                        cb.checked = false;
                    });
                    updateCount();
                }
            };
        },

        initSearch: (inputId, tableSelector) => {
            const input = document.getElementById(inputId);
            const table = document.querySelector(tableSelector);
            if (!input || !table) return;

            let timer = null;
            input.addEventListener('input', () => {
                clearTimeout(timer);
                timer = setTimeout(() => {
                    const query = input.value.toLowerCase().trim();
                    const rows = table.querySelectorAll('tbody tr.clickable-row');
                    rows.forEach((row) => {
                        const text = row.textContent.toLowerCase();
                        const matches = !query || text.includes(query);
                        row.style.display = matches ? '' : 'none';
                        const detail = row.nextElementSibling;
                        if (detail && detail.classList.contains('detail-row')) {
                            detail.style.display = matches ? '' : 'none';
                        }
                    });
                }, 200);
            });
        },

        initFilterSelect: (selectId, tableSelector, dataAttr) => {
            const select = document.getElementById(selectId);
            const table = document.querySelector(tableSelector);
            if (!select || !table) return;

            select.addEventListener('change', () => {
                const value = select.value;
                const rows = table.querySelectorAll('tbody tr.clickable-row');
                rows.forEach((row) => {
                    const attrVal = row.getAttribute(dataAttr) || '';
                    const matches = !value || attrVal === value;
                    row.style.display = matches ? '' : 'none';
                    const detail = row.nextElementSibling;
                    if (detail && detail.classList.contains('detail-row')) {
                        detail.style.display = matches ? '' : 'none';
                    }
                });
            });
        },

        initForkPanel: (config) => {
            const panelApi = MyCommon.initSidePanel(config.panelId);
            if (!panelApi) return null;

            return {
                open: () => {
                    panelApi.setTitle('Fork from Org: ' + (config.entityLabel || config.entityType));
                    panelApi.setBodyText('Loading...');
                    panelApi.setFooterDom(null);
                    panelApi.open();

                    fetch(app.API_BASE + '/user/forkable/' + config.entityType)
                        .then((res) => { return res.json(); })
                        .then((data) => {
                            const items = data[config.entityType] || data.plugins || data.skills || data.agents || data.mcp_servers || data.hooks || [];
                            if (items.length === 0) {
                                panelApi.setBodyText('No org entities available to fork.');
                                return;
                            }

                            const checklist = document.createElement('div');
                            checklist.className = 'add-checklist';
                            items.forEach((item) => {
                                const label = document.createElement('label');
                                label.className = 'acl-checkbox-row';
                                const input = document.createElement('input');
                                input.type = 'checkbox';
                                input.name = 'fork_id';
                                input.value = item.id;
                                if (item.already_forked) input.disabled = true;
                                const span = document.createElement('span');
                                span.className = 'acl-checkbox-label';
                                span.textContent = (item.name || item.id) + (item.already_forked ? ' (already forked)' : '');
                                label.append(input, span);
                                checklist.append(label);
                            });
                            panelApi.setBodyDom(checklist);

                            const footerFrag = document.createDocumentFragment();
                            const cancelBtn = document.createElement('button');
                            cancelBtn.className = 'btn btn-secondary';
                            cancelBtn.setAttribute('data-panel-close', '');
                            cancelBtn.textContent = 'Cancel';
                            const saveBtn = document.createElement('button');
                            saveBtn.className = 'btn btn-primary';
                            saveBtn.setAttribute('data-fork-save', '');
                            saveBtn.textContent = 'Fork Selected';
                            footerFrag.append(cancelBtn, document.createTextNode(' '), saveBtn);
                            panelApi.setFooterDom(footerFrag);

                            cancelBtn.addEventListener('click', panelApi.close);

                            saveBtn.addEventListener('click', () => {
                                const checked = panelApi.panel.querySelectorAll('input[name="fork_id"]:checked');
                                if (checked.length === 0) {
                                    app.Toast.show('Select at least one entity to fork', 'warning');
                                    return;
                                }
                                saveBtn.disabled = true;
                                saveBtn.textContent = 'Forking...';

                                const promises = [];
                                const typeKey = config.entityType.replace(/s$/, '');
                                checked.forEach((cb) => {
                                    const body = {};
                                    body['org_' + typeKey + '_id'] = cb.value;
                                    promises.push(
                                        fetch(app.API_BASE + '/user/fork/' + typeKey.replace('_', '-'), {
                                            method: 'POST',
                                            headers: { 'Content-Type': 'application/json' },
                                            body: JSON.stringify(body)
                                        })
                                    );
                                });

                                Promise.all(promises).then((results) => {
                                    const ok = results.filter((r) => { return r.ok; }).length;
                                    app.Toast.show('Forked ' + ok + ' ' + config.entityLabel + '(s)', 'success');
                                    panelApi.close();
                                    if (config.onForked) config.onForked();
                                    else setTimeout(() => { window.location.reload(); }, 500);
                                }).catch(() => {
                                    app.Toast.show('Fork failed', 'error');
                                    saveBtn.disabled = false;
                                    saveBtn.textContent = 'Fork Selected';
                                });
                            });
                        })
                        .catch(() => {
                            const errP = document.createElement('p');
                            errP.style.cssText = 'color:var(--sp-danger);text-align:center;padding:var(--sp-space-4)';
                            errP.textContent = 'Failed to load forkable entities.';
                            panelApi.setBodyDom(errP);
                        });
                },
                close: panelApi.close,
                panel: panelApi
            };
        },

        formatJson: (data) => {
            if (typeof data === 'string') {
                try { data = JSON.parse(data); } catch (e) {
                    const span = document.createElement('span');
                    span.textContent = data;
                    return span;
                }
            }
            const pre = document.createElement('pre');
            pre.className = 'json-view';
            pre.textContent = JSON.stringify(data, null, 2);
            return pre;
        },

        renderSourceBadge: (baseId) => {
            const container = document.createElement('span');
            const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
            svg.setAttribute('class', 'fork-icon');
            svg.setAttribute('viewBox', '0 0 16 16');
            svg.setAttribute('fill', 'currentColor');
            const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
            if (baseId) {
                container.className = 'fork-indicator forked';
                path.setAttribute('d', 'M5 3.25a.75.75 0 11-1.5 0 .75.75 0 011.5 0zm0 2.122a2.25 2.25 0 10-1.5 0v.878A2.25 2.25 0 005.75 8.5h1.5v2.128a2.251 2.251 0 101.5 0V8.5h1.5a2.25 2.25 0 002.25-2.25v-.878a2.25 2.25 0 10-1.5 0v.878a.75.75 0 01-.75.75h-4.5A.75.75 0 015 6.25v-.878z');
                svg.append(path);
                container.append(svg, 'forked');
            } else {
                container.className = 'fork-indicator custom';
                path.setAttribute('d', 'M8 2a6 6 0 100 12A6 6 0 008 2zm.75 3.75v2.5h2.5v1.5h-2.5v2.5h-1.5v-2.5h-2.5v-1.5h2.5v-2.5h1.5z');
                svg.append(path);
                container.append(svg, 'custom');
            }
            return container;
        }
    };

    app.MyCommon = MyCommon;

    app.initMyPlugins = () => {
        MyCommon.initExpandRows('#my-plugins-table');
        MyCommon.initSearch('my-plugins-search', '#my-plugins-table');
        MyCommon.initFilterSelect('my-plugins-category-filter', '#my-plugins-table', 'data-category');
        MyCommon.initBulkActions('#my-plugins-table', 'my-plugins-bulk-bar');

        const forkBtn = document.getElementById('my-plugins-fork-btn');
        if (forkBtn) {
            const forkPanel = MyCommon.initForkPanel({
                panelId: 'fork-panel',
                entityType: 'plugins',
                entityLabel: 'plugin'
            });
            if (forkPanel) {
                forkBtn.addEventListener('click', forkPanel.open);
            }
        }
    };

    app.initMySkills = () => {
        MyCommon.initExpandRows('#my-skills-table');
        MyCommon.initSearch('my-skills-search', '#my-skills-table');
        MyCommon.initFilterSelect('my-skills-tag-filter', '#my-skills-table', 'data-tags');
        MyCommon.initBulkActions('#my-skills-table', 'my-skills-bulk-bar');

        const forkBtn = document.getElementById('my-skills-fork-btn');
        if (forkBtn) {
            const forkPanel = MyCommon.initForkPanel({
                panelId: 'fork-panel',
                entityType: 'skills',
                entityLabel: 'skill'
            });
            if (forkPanel) {
                forkBtn.addEventListener('click', forkPanel.open);
            }
        }
    };

    app.initMyAgents = () => {
        MyCommon.initExpandRows('#my-agents-table');
        MyCommon.initSearch('my-agents-search', '#my-agents-table');
        MyCommon.initBulkActions('#my-agents-table', 'my-agents-bulk-bar');

        const forkBtn = document.getElementById('my-agents-fork-btn');
        if (forkBtn) {
            const forkPanel = MyCommon.initForkPanel({
                panelId: 'fork-panel',
                entityType: 'agents',
                entityLabel: 'agent'
            });
            if (forkPanel) {
                forkBtn.addEventListener('click', forkPanel.open);
            }
        }
    };

    app.initMyMcpServers = () => {
        MyCommon.initExpandRows('#my-mcp-table');
        MyCommon.initSearch('my-mcp-search', '#my-mcp-table');
        MyCommon.initBulkActions('#my-mcp-table', 'my-mcp-bulk-bar');

        const forkBtn = document.getElementById('my-mcp-fork-btn');
        if (forkBtn) {
            const forkPanel = MyCommon.initForkPanel({
                panelId: 'fork-panel',
                entityType: 'mcp-servers',
                entityLabel: 'MCP server'
            });
            if (forkPanel) {
                forkBtn.addEventListener('click', forkPanel.open);
            }
        }
    };

    app.initMyHooks = () => {
        MyCommon.initExpandRows('#my-hooks-table');
        MyCommon.initSearch('my-hooks-search', '#my-hooks-table');
        MyCommon.initBulkActions('#my-hooks-table', 'my-hooks-bulk-bar');

        const forkBtn = document.getElementById('my-hooks-fork-btn');
        if (forkBtn) {
            const forkPanel = MyCommon.initForkPanel({
                panelId: 'fork-panel',
                entityType: 'hooks',
                entityLabel: 'hook'
            });
            if (forkPanel) {
                forkBtn.addEventListener('click', forkPanel.open);
            }
        }
    };

    app.initMyMarketplace = () => {
        MyCommon.initExpandRows('#my-marketplace-table');
        MyCommon.initSearch('my-marketplace-search', '#my-marketplace-table');
        MyCommon.initFilterSelect('my-marketplace-source-filter', '#my-marketplace-table', 'data-source');
        MyCommon.initFilterSelect('my-marketplace-category-filter', '#my-marketplace-table', 'data-category');

        app.events.on('click', '[data-customize-plugin]', (e, btn) => {
            const pluginId = btn.getAttribute('data-customize-plugin');
            btn.disabled = true;
            btn.textContent = 'Customizing...';

            fetch(app.API_BASE + '/user/fork/plugin', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ org_plugin_id: pluginId })
            }).then((res) => {
                if (res.ok) {
                    return res.json().then((data) => {
                        app.Toast.show('Plugin customized successfully', 'success');
                        if (data.plugin && data.plugin.plugin_id) {
                            setTimeout(() => {
                                window.location.href = '/admin/my/plugins/edit?id=' + encodeURIComponent(data.plugin.plugin_id);
                            }, 500);
                        } else {
                            setTimeout(() => { window.location.reload(); }, 500);
                        }
                    });
                } else {
                    app.Toast.show('Failed to customize plugin', 'error');
                    btn.disabled = false;
                    btn.textContent = 'Customize';
                }
            }).catch(() => {
                app.Toast.show('Failed to customize plugin', 'error');
                btn.disabled = false;
                btn.textContent = 'Customize';
            });
        });
    };

})(window.AdminApp || (window.AdminApp = {}));

// Time range picker — progressive enhancement.
// Reveals custom inputs when "Custom" preset is clicked without a full
// page nav, so users can choose dates before submitting.
(function () {
  'use strict';

  function init(root) {
    var customPanel = root.querySelector('[data-time-range-custom]');
    if (!customPanel) return;

    root.querySelectorAll('.time-range__btn').forEach(function (btn) {
      btn.addEventListener('click', function (event) {
        if (btn.dataset.preset !== 'custom') return;
        // Only intercept if custom panel is currently hidden — otherwise
        // let the link navigate normally to apply the custom range.
        if (!customPanel.hasAttribute('hidden')) return;
        event.preventDefault();
        customPanel.removeAttribute('hidden');
        root.querySelectorAll('.time-range__btn').forEach(function (b) {
          b.classList.remove('time-range__btn--active');
          b.removeAttribute('aria-current');
        });
        btn.classList.add('time-range__btn--active');
        btn.setAttribute('aria-current', 'true');
        var fromInput = customPanel.querySelector('[data-time-range-from]');
        if (fromInput) fromInput.focus();
      });
    });
  }

  function boot() {
    document.querySelectorAll('[data-time-range]').forEach(init);
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', boot);
  } else {
    boot();
  }
})();

// Identity filter ribbon — progressive enhancement: typeahead inside dropdown
// groups when option lists exceed 20 items. SSR-rendered checkboxes already
// work without JS; this only filters which <li> entries are visible.
(function () {
  'use strict';

  var TYPEAHEAD_THRESHOLD = 20;

  function initGroup(group) {
    var search = group.querySelector('[data-filter-typeahead]');
    var list = group.querySelector('[data-filter-list]');
    if (!search || !list) return;

    var items = Array.prototype.slice.call(
      list.querySelectorAll('.filter-ribbon__group-item'),
    );
    if (items.length <= TYPEAHEAD_THRESHOLD) {
      search.hidden = true;
      return;
    }

    search.addEventListener('input', function () {
      var query = search.value.trim().toLowerCase();
      items.forEach(function (item) {
        var label = (item.dataset.label || '').toLowerCase();
        if (!query || label.indexOf(query) !== -1) {
          item.hidden = false;
        } else {
          item.hidden = true;
        }
      });
    });
  }

  function closeOthersOnOpen(root) {
    var groups = root.querySelectorAll('details.filter-ribbon__group');
    groups.forEach(function (g) {
      g.addEventListener('toggle', function () {
        if (!g.open) return;
        groups.forEach(function (other) {
          if (other !== g) other.open = false;
        });
      });
    });
  }

  function init(root) {
    root.querySelectorAll('details.filter-ribbon__group').forEach(initGroup);
    closeOthersOnOpen(root);
  }

  function boot() {
    document.querySelectorAll('[data-filter-ribbon]').forEach(init);
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', boot);
  } else {
    boot();
  }
})();

// Sparkline auto-mounter — finds every `[data-sparkline]` element with a JSON
// array of numbers and draws a line + faint area underneath inside an injected
// canvas. Color comes from CSS `currentColor`, which the sparkline-card maps
// from `data-sparkline-color`. Hidpi-aware.
(function () {
  'use strict';

  function readPoints(el) {
    var raw = el.getAttribute('data-sparkline');
    if (!raw) return null;
    try {
      var parsed = JSON.parse(raw);
      if (!Array.isArray(parsed) || parsed.length < 2) return null;
      return parsed.map(function (n) {
        var v = Number(n);
        return Number.isFinite(v) ? v : 0;
      });
    } catch (_e) {
      return null;
    }
  }

  function fillCanvas(canvas, w, h) {
    var dpr = window.devicePixelRatio || 1;
    canvas.width = Math.max(1, Math.floor(w * dpr));
    canvas.height = Math.max(1, Math.floor(h * dpr));
    canvas.style.width = w + 'px';
    canvas.style.height = h + 'px';
    var ctx = canvas.getContext('2d');
    if (ctx) ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    return ctx;
  }

  function draw(el, points) {
    var rect = el.getBoundingClientRect();
    var w = rect.width || 120;
    var h = rect.height || 32;
    if (w < 4 || h < 4) return;

    var canvas = el.querySelector('canvas');
    if (!canvas) {
      canvas = document.createElement('canvas');
      el.textContent = '';
      el.append(canvas);
    }
    var ctx = fillCanvas(canvas, w, h);
    if (!ctx) return;

    var color = window.getComputedStyle(el).color || '#6366f1';
    var max = Math.max.apply(null, points);
    var min = Math.min.apply(null, points);
    var range = max - min || 1;
    var pad = 1.5;
    var step = (w - pad * 2) / (points.length - 1);

    var coords = points.map(function (v, i) {
      return {
        x: pad + i * step,
        y: pad + (h - pad * 2) - ((v - min) / range) * (h - pad * 2),
      };
    });

    // Faint area fill
    ctx.beginPath();
    ctx.moveTo(coords[0].x, h);
    for (var i = 0; i < coords.length; i++) ctx.lineTo(coords[i].x, coords[i].y);
    ctx.lineTo(coords[coords.length - 1].x, h);
    ctx.closePath();
    ctx.fillStyle = color;
    ctx.globalAlpha = 0.12;
    ctx.fill();
    ctx.globalAlpha = 1;

    // Line
    ctx.beginPath();
    ctx.moveTo(coords[0].x, coords[0].y);
    for (var j = 1; j < coords.length; j++) ctx.lineTo(coords[j].x, coords[j].y);
    ctx.lineWidth = 1.5;
    ctx.strokeStyle = color;
    ctx.stroke();
  }

  function mount() {
    document.querySelectorAll('[data-sparkline]').forEach(function (el) {
      var pts = readPoints(el);
      if (pts) draw(el, pts);
    });
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', mount);
  } else {
    mount();
  }

  // Redraw on resize (debounced)
  var t;
  window.addEventListener('resize', function () {
    clearTimeout(t);
    t = setTimeout(mount, 120);
  });
})();

// Portfolio stacked-area chart — reads JSON from
// <script type="application/json" id="portfolio-area-data"> and renders an
// allow/deny stacked area on <canvas id="portfolio-area-canvas">.
//
// Data shape:
//   { buckets: [{ label, allow, deny }, ...] }
//
// No external chart lib — single canvas pass with the same getContext/moveTo
// patterns used by admin-dashboard-charts.js. Tokens read from CSS via
// computed style; falls back to hard-coded sane colors if not set.
(function () {
  'use strict';

  function readData() {
    var el = document.getElementById('portfolio-area-data');
    if (!el) return null;
    try {
      var parsed = JSON.parse(el.textContent || '{}');
      if (!parsed || !Array.isArray(parsed.buckets) || parsed.buckets.length < 2) return null;
      return parsed;
    } catch (_e) {
      return null;
    }
  }

  function token(name, fallback) {
    var v = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    return v || fallback;
  }

  function fillCanvas(canvas, w, h) {
    var dpr = window.devicePixelRatio || 1;
    canvas.width = Math.max(1, Math.floor(w * dpr));
    canvas.height = Math.max(1, Math.floor(h * dpr));
    canvas.style.width = w + 'px';
    canvas.style.height = h + 'px';
    var ctx = canvas.getContext('2d');
    if (ctx) ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    return ctx;
  }

  function draw() {
    var data = readData();
    if (!data) return;
    var canvas = document.getElementById('portfolio-area-canvas');
    if (!canvas) return;

    var rect = canvas.getBoundingClientRect();
    var w = rect.width || 800;
    var h = rect.height || 220;
    var ctx = fillCanvas(canvas, w, h);
    if (!ctx) return;

    var padL = 36;
    var padR = 8;
    var padT = 8;
    var padB = 24;
    var plotW = w - padL - padR;
    var plotH = h - padT - padB;

    var buckets = data.buckets;
    var n = buckets.length;
    var totals = buckets.map(function (b) { return (b.allow || 0) + (b.deny || 0); });
    var max = Math.max.apply(null, totals);
    if (max === 0) max = 1;

    var step = plotW / (n - 1);

    var allowColor = token('--sp-success', '#16a34a');
    var denyColor = token('--sp-danger', '#dc2626');
    var gridColor = token('--sp-border-default', '#e5e7eb');
    var textColor = token('--sp-text-tertiary', '#888');

    // Y grid (4 lines including 0 and max)
    ctx.strokeStyle = gridColor;
    ctx.lineWidth = 1;
    ctx.fillStyle = textColor;
    ctx.font = '10px sans-serif';
    ctx.textAlign = 'right';
    ctx.textBaseline = 'middle';
    for (var g = 0; g <= 4; g++) {
      var yv = max * (1 - g / 4);
      var y = padT + (plotH * g) / 4;
      ctx.beginPath();
      ctx.moveTo(padL, y);
      ctx.lineTo(padL + plotW, y);
      ctx.stroke();
      ctx.fillText(Math.round(yv), padL - 4, y);
    }

    function pointAt(i, value) {
      return {
        x: padL + i * step,
        y: padT + plotH - (value / max) * plotH,
      };
    }

    // Allow band (bottom): polygon from baseline up to allow values
    ctx.beginPath();
    ctx.moveTo(padL, padT + plotH);
    for (var i = 0; i < n; i++) {
      var p = pointAt(i, buckets[i].allow || 0);
      ctx.lineTo(p.x, p.y);
    }
    ctx.lineTo(padL + plotW, padT + plotH);
    ctx.closePath();
    ctx.fillStyle = allowColor;
    ctx.globalAlpha = 0.55;
    ctx.fill();
    ctx.globalAlpha = 1;

    // Deny band (stacked on top): polygon from allow line up to allow+deny
    ctx.beginPath();
    var first = pointAt(0, buckets[0].allow || 0);
    ctx.moveTo(first.x, first.y);
    for (var k = 0; k < n; k++) {
      var bk = buckets[k];
      var top = pointAt(k, (bk.allow || 0) + (bk.deny || 0));
      ctx.lineTo(top.x, top.y);
    }
    for (var m = n - 1; m >= 0; m--) {
      var bottom = pointAt(m, buckets[m].allow || 0);
      ctx.lineTo(bottom.x, bottom.y);
    }
    ctx.closePath();
    ctx.fillStyle = denyColor;
    ctx.globalAlpha = 0.6;
    ctx.fill();
    ctx.globalAlpha = 1;

    // X labels — show ~6 evenly-spaced ticks
    ctx.fillStyle = textColor;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'top';
    var labelStep = Math.max(1, Math.floor(n / 6));
    for (var li = 0; li < n; li += labelStep) {
      var lx = padL + li * step;
      ctx.fillText(buckets[li].label || '', lx, padT + plotH + 4);
    }
  }

  function boot() {
    draw();
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', boot);
  } else {
    boot();
  }

  var t;
  window.addEventListener('resize', function () {
    clearTimeout(t);
    t = setTimeout(draw, 120);
  });
})();

(function (app) {
    'use strict';

    function init() {
        var page = document.querySelector('[data-page="conversations"]');
        if (!page) return;

        var toggle = document.getElementById('conversations-redaction-toggle');
        if (toggle && !toggle.disabled) {
            toggle.addEventListener('click', onToggleClick);
        }

        // Mark active jump-nav link as the user scrolls.
        var turnsContainer = document.getElementById('conversations-turns');
        if (turnsContainer && 'IntersectionObserver' in window) {
            var navLinks = page.querySelectorAll('.conversations-page__jumpnav a[data-jump]');
            var byOrdinal = {};
            navLinks.forEach(function (a) { byOrdinal[a.dataset.jump] = a; });
            var io = new IntersectionObserver(function (entries) {
                entries.forEach(function (entry) {
                    if (entry.isIntersecting) {
                        var ord = entry.target.dataset.ordinal;
                        navLinks.forEach(function (a) { a.classList.remove('is-active'); });
                        if (byOrdinal[ord]) byOrdinal[ord].classList.add('is-active');
                    }
                });
            }, { root: turnsContainer, threshold: 0.4 });
            turnsContainer.querySelectorAll('.transcript-turn').forEach(function (el) {
                io.observe(el);
            });
        }
    }

    function onToggleClick(ev) {
        var btn = ev.currentTarget;
        var sessionId = btn.dataset.sessionId;
        var body = document.querySelector('[data-conversation-detail]');
        if (!sessionId || !body) return;

        var mode = body.dataset.redactionMode || 'redacted';
        if (mode === 'raw') {
            switchToMode(body, btn, 'redacted');
            return;
        }

        // Need to fetch raw bodies before we can switch.
        if (btn.dataset.rawLoaded === '1') {
            switchToMode(body, btn, 'raw');
            return;
        }

        btn.disabled = true;
        var originalLabel = btn.textContent;
        btn.textContent = 'Loading…';

        fetchRaw(sessionId)
            .then(function (envelope) {
                applyRawTurns(body, envelope);
                btn.dataset.rawLoaded = '1';
                switchToMode(body, btn, 'raw');
            })
            .catch(function (err) {
                if (app && app.Toast) {
                    app.Toast.show('Could not load raw transcript: ' + err.message, 'error');
                } else {
                    console.error('raw transcript fetch failed', err);
                }
            })
            .then(function () {
                btn.disabled = false;
                if (btn.textContent === 'Loading…') btn.textContent = originalLabel;
            });
    }

    function fetchRaw(sessionId) {
        return fetch('/admin/api/conversations/' + encodeURIComponent(sessionId) + '/raw', {
            credentials: 'same-origin',
            headers: { Accept: 'application/json' }
        }).then(function (r) {
            if (r.status === 403) throw new Error('Forbidden — auditor role required');
            if (!r.ok) throw new Error('HTTP ' + r.status);
            return r.json();
        });
    }

    function applyRawTurns(body, envelope) {
        if (!envelope || !Array.isArray(envelope.turns)) return;
        var turns = body.querySelectorAll('.transcript-turn');
        turns.forEach(function (turnEl) {
            var ordinal = parseInt(turnEl.dataset.ordinal, 10);
            var raw = envelope.turns.find(function (t) { return t.ordinal === ordinal; });
            if (!raw) return;
            // If the partial didn't render the raw <div>, inject one. Otherwise overwrite.
            var bubble = turnEl.querySelector('.transcript-turn__bubble');
            if (!bubble) return;
            var rawEl = bubble.querySelector('.transcript-turn__content--raw');
            if (!rawEl) {
                rawEl = document.createElement('div');
                rawEl.className = 'transcript-turn__content transcript-turn__content--raw';
                rawEl.dataset.mode = 'raw';
                rawEl.hidden = true;
                bubble.appendChild(rawEl);
            }
            rawEl.textContent = raw.content || '';
        });
    }

    function switchToMode(body, btn, mode) {
        body.dataset.redactionMode = mode;
        var redacted = body.querySelectorAll('.transcript-turn__content--redacted');
        var raw = body.querySelectorAll('.transcript-turn__content--raw');
        redacted.forEach(function (el) { el.hidden = (mode === 'raw'); });
        raw.forEach(function (el) { el.hidden = (mode !== 'raw'); });
        btn.textContent = (mode === 'raw') ? 'Show redacted content' : 'Show raw content';
        btn.classList.toggle('is-active', mode === 'raw');
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})(window.AdminApp || (window.AdminApp = {}));

// Waterfall renderer for the per-trace detail page.
// Reads spans from <script type="application/json" id="trace-spans"> and
// draws an inline SVG with one row per span. Bars color-coded by `kind` via
// CSS class (`.waterfall__bar--{kind}`); deny/error get an extra outline.
// Each row is clickable: it dispatches `data-chain-id` which the existing
// chain-drawer.js picks up.
(function () {
  'use strict';

  var ROW_H = 26;
  var BAR_H = 14;
  var LABEL_W = 220;
  var RIGHT_W = 80;
  var AXIS_H = 22;
  var PAD_TOP = 8;
  var PAD_BOTTOM = 4;
  var MIN_BAR_PX = 3;

  function readSpans() {
    var el = document.getElementById('trace-spans');
    if (!el) return null;
    try {
      var arr = JSON.parse(el.textContent || '[]');
      return Array.isArray(arr) ? arr : null;
    } catch (_e) {
      return null;
    }
  }

  function svgEl(name, attrs) {
    var ns = 'http://www.w3.org/2000/svg';
    var n = document.createElementNS(ns, name);
    if (attrs) {
      for (var k in attrs) {
        if (Object.prototype.hasOwnProperty.call(attrs, k)) {
          n.setAttribute(k, attrs[k]);
        }
      }
    }
    return n;
  }

  function escapeText(s) {
    return String(s == null ? '' : s);
  }

  function formatDuration(ms) {
    if (ms < 1) return '<1 ms';
    if (ms < 1000) return ms + ' ms';
    if (ms < 60000) return (ms / 1000).toFixed(2) + ' s';
    return (ms / 60000).toFixed(1) + ' min';
  }

  function render(root, spans) {
    if (!spans || !spans.length) {
      root.innerHTML = '<div class="waterfall__empty">No spans in this trace.</div>';
      return;
    }

    var startsMs = spans.map(function (s) { return new Date(s.started_at).getTime(); });
    var endsMs = spans.map(function (s) {
      var e = new Date(s.ended_at).getTime();
      var st = new Date(s.started_at).getTime();
      // Point-in-time spans (duration_ms === 0) get a 1px nudge so they remain visible.
      return Math.max(e, st);
    });
    var totalStart = Math.min.apply(null, startsMs);
    var totalEnd = Math.max.apply(null, endsMs);
    var totalSpan = Math.max(1, totalEnd - totalStart);

    var rect = root.getBoundingClientRect();
    var width = Math.max(rect.width || 800, 600);
    var plotL = LABEL_W;
    var plotR = width - RIGHT_W;
    var plotW = Math.max(50, plotR - plotL);
    var height = PAD_TOP + AXIS_H + spans.length * ROW_H + PAD_BOTTOM;

    var svg = svgEl('svg', {
      'class': 'waterfall__svg',
      'viewBox': '0 0 ' + width + ' ' + height,
      'role': 'img',
      'aria-label': 'Trace waterfall with ' + spans.length + ' spans',
    });

    // Axis grid (4 ticks)
    var axisY = PAD_TOP + AXIS_H - 4;
    for (var t = 0; t <= 4; t++) {
      var x = plotL + (plotW * t) / 4;
      var ms = (totalSpan * t) / 4;
      var line = svgEl('line', {
        'class': 'waterfall__grid',
        'x1': x.toFixed(1), 'y1': PAD_TOP + AXIS_H,
        'x2': x.toFixed(1), 'y2': height - PAD_BOTTOM,
      });
      svg.append(line);
      var lbl = svgEl('text', {
        'class': 'waterfall__axis',
        'x': x.toFixed(1), 'y': axisY,
        'text-anchor': t === 0 ? 'start' : (t === 4 ? 'end' : 'middle'),
      });
      lbl.textContent = formatDuration(ms);
      svg.append(lbl);
    }

    // Per-span row
    spans.forEach(function (s, i) {
      var rowY = PAD_TOP + AXIS_H + i * ROW_H;
      var startMs = new Date(s.started_at).getTime() - totalStart;
      var dur = Math.max(0, s.duration_ms || 0);
      var x0 = plotL + (startMs / totalSpan) * plotW;
      var w = (dur / totalSpan) * plotW;
      if (w < MIN_BAR_PX) w = MIN_BAR_PX;

      // Row background — clickable area for chain-drawer
      var bg = svgEl('rect', {
        'class': 'waterfall__row-bg',
        'x': 0, 'y': rowY,
        'width': width, 'height': ROW_H,
        'fill': 'transparent',
      });
      svg.append(bg);

      // Group whose click triggers chain drawer via data-chain-id
      var g = svgEl('g', {
        'class': 'waterfall__row',
        'data-chain-id': s.id,
        'tabindex': '0',
        'role': 'button',
        'aria-label': 'Open chain envelope for ' + (s.name || s.kind),
      });

      // Left label (name)
      var label = svgEl('text', {
        'class': 'waterfall__label',
        'x': 8, 'y': rowY + ROW_H / 2 + 4,
      });
      var labelText = escapeText(s.name || s.kind);
      if (labelText.length > 30) labelText = labelText.slice(0, 30) + '…';
      label.textContent = labelText;
      g.append(label);

      // Bar
      var classes = ['waterfall__bar', 'waterfall__bar--' + (s.kind || 'tool')];
      if (s.status === 'deny') classes.push('waterfall__bar--deny');
      if (s.status === 'error') classes.push('waterfall__bar--error');
      var bar = svgEl('rect', {
        'class': classes.join(' '),
        'x': x0.toFixed(1),
        'y': rowY + (ROW_H - BAR_H) / 2,
        'width': w.toFixed(1),
        'height': BAR_H,
        'rx': 2, 'ry': 2,
      });
      var titleEl = svgEl('title');
      titleEl.textContent = (s.kind || '') + ' · ' + (s.name || '') +
        ' · ' + formatDuration(dur) + ' · ' + (s.status || '');
      bar.append(titleEl);
      g.append(bar);

      // Right-side duration label
      var right = svgEl('text', {
        'class': 'waterfall__label waterfall__label--right',
        'x': width - 8, 'y': rowY + ROW_H / 2 + 4,
        'text-anchor': 'end',
      });
      right.textContent = formatDuration(dur);
      g.append(right);

      svg.append(g);
    });

    root.textContent = '';
    root.append(svg);
  }

  function dispatchChainOpen(target) {
    // Walk up to find data-chain-id on the SVG group.
    var n = target;
    while (n && n !== document) {
      if (n.getAttribute && n.getAttribute('data-chain-id')) {
        // The chain-drawer.js picks up clicks via document delegation, but
        // SVG events sometimes don't bubble cleanly — synthesize a click on
        // a hidden anchor with the same attribute.
        var ghost = document.createElement('button');
        ghost.setAttribute('data-chain-id', n.getAttribute('data-chain-id'));
        ghost.style.display = 'none';
        document.body.append(ghost);
        ghost.click();
        ghost.remove();
        return;
      }
      n = n.parentNode;
    }
  }

  function bindClicks(root) {
    root.addEventListener('click', function (ev) {
      dispatchChainOpen(ev.target);
    });
    root.addEventListener('keydown', function (ev) {
      if (ev.key !== 'Enter' && ev.key !== ' ') return;
      dispatchChainOpen(ev.target);
    });
  }

  function boot() {
    var root = document.querySelector('[data-waterfall]');
    if (!root) return;
    var spans = readSpans();
    render(root, spans || []);
    bindClicks(root);
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', boot);
  } else {
    boot();
  }

  var t;
  window.addEventListener('resize', function () {
    clearTimeout(t);
    t = setTimeout(function () {
      var root = document.querySelector('[data-waterfall]');
      if (!root) return;
      var spans = readSpans();
      render(root, spans || []);
    }, 150);
  });
})();

// Live audit event stream — subscribes to /admin/api/sse/audit, renders
// severity-coded rows with click-to-open chain drawer, and maintains rolling
// 60s counters. Pause / autoscroll / severity filters / text search are all
// client-side. Notifications API is opt-in.
(function () {
  'use strict';

  var SSE_URL = '/admin/api/sse/audit';
  var MAX_ROWS = 500;
  var WINDOW_MS = 60_000;

  var state = {
    paused: false,
    autoScroll: true,
    notify: false,
    severityFilters: { info: true, warn: true, deny: true, breach: true, error: true },
    searchTerm: '',
    history: [],            // ringbuffer of {ts, severity}
    eventSource: null,
    retryTimeout: null,
    retryDelayMs: 1000,
  };

  function $(sel, root) { return (root || document).querySelector(sel); }
  function $$(sel, root) { return Array.prototype.slice.call((root || document).querySelectorAll(sel)); }

  function escapeText(s) {
    return String(s == null ? '' : s)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');
  }

  function setStatus(label, ok) {
    var stateEl = $('[data-stream-state]');
    var indicator = $('[data-stream-indicator]');
    if (stateEl) stateEl.textContent = label;
    if (indicator) {
      indicator.classList.remove('is-ok', 'is-warn', 'is-down');
      indicator.classList.add(ok === true ? 'is-ok' : (ok === false ? 'is-down' : 'is-warn'));
    }
  }

  function pruneHistory() {
    var cutoff = Date.now() - WINDOW_MS;
    while (state.history.length && state.history[0].ts < cutoff) {
      state.history.shift();
    }
  }

  function updateCounters() {
    pruneHistory();
    var totalRate = state.history.length / (WINDOW_MS / 1000);
    var denyCount = 0, breachCount = 0, errorCount = 0;
    state.history.forEach(function (h) {
      if (h.severity === 'deny' || h.severity === 'breach') denyCount++;
      if (h.severity === 'breach') breachCount++;
      if (h.severity === 'error') errorCount++;
    });
    var denyRate = denyCount / (WINDOW_MS / 1000);
    var rate = $('[data-rail-rate]');
    var denyRateEl = $('[data-rail-deny-rate]');
    var breachEl = $('[data-rail-breach]');
    var errorEl = $('[data-rail-error]');
    if (rate) rate.textContent = totalRate.toFixed(1);
    if (denyRateEl) denyRateEl.textContent = denyRate.toFixed(2);
    if (breachEl) breachEl.textContent = String(breachCount);
    if (errorEl) errorEl.textContent = String(errorCount);
  }

  function shouldShow(payload) {
    var sev = payload.severity || 'info';
    if (!state.severityFilters[sev]) return false;
    if (!state.searchTerm) return true;
    var hay = [
      payload.user_id, payload.tool_name, payload.policy,
      payload.decision, payload.model, payload.session_id, payload.trace_id,
    ].filter(Boolean).join(' ').toLowerCase();
    return hay.indexOf(state.searchTerm) !== -1;
  }

  function severityBadge(sev) {
    var label = String(sev || 'info').toUpperCase();
    return '<span class="event-row__severity event-row__severity--' +
      escapeText(sev) + '">' + escapeText(label) + '</span>';
  }

  function renderRow(payload) {
    var row = document.createElement('article');
    row.className = 'event-row event-row--' + escapeText(payload.severity || 'info');
    if (payload.id) row.setAttribute('data-chain-id', payload.id);
    row.tabIndex = 0;
    row.setAttribute('role', 'button');
    var when = payload.created_at
      ? new Date(payload.created_at).toLocaleTimeString()
      : new Date().toLocaleTimeString();
    var primary = payload.table === 'governance_decisions'
      ? (payload.policy || 'governance') + ' · ' + (payload.decision || '')
      : (payload.model || 'request') + ' · ' + (payload.status || '');
    var secondary = [
      payload.user_id ? 'user ' + payload.user_id : null,
      payload.tool_name ? 'tool ' + payload.tool_name : null,
      payload.tenant_id ? 'tenant ' + payload.tenant_id : null,
    ].filter(Boolean).join(' · ');

    row.innerHTML =
      '<span class="event-row__time">' + escapeText(when) + '</span>' +
      severityBadge(payload.severity) +
      '<span class="event-row__primary">' + escapeText(primary) + '</span>' +
      '<span class="event-row__secondary">' + escapeText(secondary) + '</span>';
    return row;
  }

  function maybeNotify(payload) {
    if (!state.notify) return;
    if (!('Notification' in window)) return;
    if (Notification.permission !== 'granted') return;
    if (payload.severity !== 'breach' && payload.severity !== 'error') return;
    try {
      new Notification('Audit ' + (payload.severity || ''), {
        body: (payload.policy || payload.model || 'event') + ' · ' +
          (payload.decision || payload.status || ''),
        tag: payload.id || String(Date.now()),
      });
    } catch (_e) {
      // Some browsers throw if called from non-secure context — ignore.
    }
  }

  function appendEvent(payload) {
    state.history.push({ ts: Date.now(), severity: payload.severity || 'info' });
    updateCounters();
    if (state.paused) return;
    if (!shouldShow(payload)) return;

    var list = $('[data-stream-list]');
    if (!list) return;
    var empty = $('[data-stream-empty]');
    if (empty) empty.remove();

    var row = renderRow(payload);
    list.append(row);

    while (list.children.length > MAX_ROWS) {
      list.removeChild(list.firstElementChild);
    }
    if (state.autoScroll) {
      list.scrollTop = list.scrollHeight;
    }
    maybeNotify(payload);
  }

  function connect() {
    if (state.eventSource) {
      try { state.eventSource.close(); } catch (_e) {}
    }
    setStatus('connecting…', null);
    var es = new EventSource(SSE_URL);
    state.eventSource = es;

    es.addEventListener('hello', function () {
      setStatus('live', true);
      state.retryDelayMs = 1000;
    });
    es.addEventListener('audit', function (ev) {
      try {
        var payload = JSON.parse(ev.data);
        appendEvent(payload);
      } catch (_e) {
        // ignore malformed payloads
      }
    });
    es.addEventListener('lagged', function (ev) {
      setStatus('lagging', false);
      try {
        var info = JSON.parse(ev.data);
        // surface dropped count as a synthetic warn row
        appendEvent({
          severity: 'warn',
          policy: 'stream',
          decision: 'lagged · ' + (info.skipped || '?') + ' dropped',
        });
      } catch (_e) {}
    });
    es.onerror = function () {
      setStatus('reconnecting…', false);
      es.close();
      state.eventSource = null;
      var delay = Math.min(state.retryDelayMs, 15_000);
      state.retryDelayMs = Math.min(state.retryDelayMs * 2, 15_000);
      clearTimeout(state.retryTimeout);
      state.retryTimeout = setTimeout(connect, delay);
    };
  }

  function bindControls() {
    var toggle = $('[data-stream-toggle]');
    if (toggle) {
      toggle.addEventListener('click', function () {
        state.paused = !state.paused;
        toggle.textContent = state.paused ? 'Resume' : 'Pause';
        toggle.classList.toggle('is-paused', state.paused);
      });
    }

    var auto = $('[data-stream-autoscroll]');
    if (auto) {
      auto.addEventListener('change', function () {
        state.autoScroll = auto.checked;
      });
    }

    var notify = $('[data-stream-notify]');
    if (notify) {
      notify.addEventListener('change', function () {
        if (!notify.checked) {
          state.notify = false;
          return;
        }
        if (!('Notification' in window)) {
          notify.checked = false;
          return;
        }
        if (Notification.permission === 'granted') {
          state.notify = true;
        } else if (Notification.permission !== 'denied') {
          Notification.requestPermission().then(function (p) {
            state.notify = p === 'granted';
            if (!state.notify) notify.checked = false;
          });
        } else {
          state.notify = false;
          notify.checked = false;
        }
      });
    }

    $$('[data-severity-filter]').forEach(function (cb) {
      cb.addEventListener('change', function () {
        state.severityFilters[cb.value] = cb.checked;
      });
    });

    var search = $('[data-stream-search]');
    if (search) {
      search.addEventListener('input', function () {
        state.searchTerm = search.value.trim().toLowerCase();
      });
    }
  }

  function bindChainOpen() {
    var list = $('[data-stream-list]');
    if (!list) return;
    list.addEventListener('keydown', function (ev) {
      if (ev.key !== 'Enter' && ev.key !== ' ') return;
      var target = ev.target;
      if (target && target.getAttribute && target.getAttribute('data-chain-id')) {
        target.click();
      }
    });
  }

  function boot() {
    if (!$('[data-stream-list]')) return;     // page is not events page
    bindControls();
    bindChainOpen();
    connect();
    setInterval(updateCounters, 1000);
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', boot);
  } else {
    boot();
  }
})();
