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
