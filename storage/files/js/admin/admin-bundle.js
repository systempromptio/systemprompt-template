window.AdminApp = window.AdminApp || {};
(function(app) {
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
    // Client-side only: extract username for display. NOT used for authorization decisions.
    // Server-side auth is handled via site_auth() cookie validation.
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
    const escapeHtml = app.escapeHtml;
    /**
     * Truncate a string to a maximum length with ellipsis.
     */
    function truncate(str, max) {
        if (!str) return '';
        if (str.length <= (max || 60)) return str;
        return str.substring(0, max || 60) + '...';
    }
    /**
     * Close all open three-dot action menus.
     */
    function closeAllMenus() {
        const openMenus = document.querySelectorAll('.actions-menu.open');
        openMenus.forEach((m) => { m.classList.remove('open'); });
    }
    /**
     * Close a delete confirmation overlay by its ID.
     */
    function closeDeleteConfirm(overlayId) {
        const overlay = document.getElementById(overlayId || 'delete-confirm');
        if (overlay) overlay.remove();
    }
    /**
     * Show a generic confirmation dialog.
     * Returns nothing; calls onConfirm() when the user clicks confirm.
     */
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
        overlay.querySelector('[data-action="cancel"]').addEventListener('click', () => {
            overlay.remove();
        });
        overlay.querySelector('[data-action="confirm"]').addEventListener('click', () => {
            overlay.remove();
            onConfirm();
        });
        overlay.addEventListener('click', (e) => {
            if (e.target === overlay) overlay.remove();
        });
        document.body.appendChild(overlay);
        return overlay;
    }
    /**
     * Show a delete confirmation dialog used by list pages (agents, hooks, skills, mcp-servers).
     * The confirm button includes a data-confirm-delete attribute for delegation.
     */
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
    /**
     * Set up debounced search with focus restore after re-render.
     * - inputId: the ID of the search input element
     * - onSearch: callback receiving the search value
     * - delay: debounce delay in ms (default 200)
     * Returns a cleanup-friendly handler to attach via addEventListener.
     */
    function createDebouncedSearch(root, inputId, onSearch, delay) {
        let searchTimer = null;
        root.addEventListener('input', (e) => {
            if (e.target.id === inputId) {
                clearTimeout(searchTimer);
                searchTimer = setTimeout(() => {
                    onSearch(e.target.value);
                    const input = document.getElementById(inputId);
                    if (input) {
                        input.focus();
                        input.selectionStart = input.selectionEnd = input.value.length;
                    }
                }, delay || 200);
            }
        });
    }
    /**
     * Register standard document-level listeners for list pages:
     * - Outside click to close menus
     * - Outside click / cancel to close delete confirm
     * - Escape key to close menus and delete confirm
     * - Confirm-delete button delegation
     *
     * onDelete(itemId, confirmBtn): called when user clicks the confirm-delete button.
     * Returns nothing. Safe to call multiple times (uses a guard flag internally).
     */
    const _registeredPages = {};
    function registerListPageListeners(pageKey, opts) {
        if (_registeredPages[pageKey]) return;
        _registeredPages[pageKey] = true;
        document.addEventListener('click', (e) => {
            if (e.target.id === 'delete-confirm' || e.target.closest('[data-confirm-cancel]')) {
                closeDeleteConfirm();
                return;
            }
            const confirmBtn = e.target.closest('[data-confirm-delete]');
            if (confirmBtn && opts.onDelete) {
                const itemId = confirmBtn.getAttribute('data-confirm-delete');
                opts.onDelete(itemId, confirmBtn);
                return;
            }
        });
        document.addEventListener('click', (e) => {
            if (!e.target.closest('.actions-menu')) {
                closeAllMenus();
            }
        });
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                closeAllMenus();
                closeDeleteConfirm();
            }
        });
    }
    /**
     * Handle three-dot menu toggle click (delegated).
     * Call from a root click handler: if it returns true, the event was handled.
     */
    function handleMenuToggle(e) {
        const menuBtn = e.target.closest('[data-action="menu"]');
        if (!menuBtn) return false;
        e.stopPropagation();
        const menu = menuBtn.closest('.actions-menu');
        const wasOpen = menu.classList.contains('open');
        closeAllMenus();
        if (!wasOpen) menu.classList.add('open');
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
            var script = document.createElement('script');
            script.src = 'https://cdnjs.cloudflare.com/ajax/libs/jszip/3.10.1/jszip.min.js';
            script.integrity = 'sha384-+mbV2IY1Zk/X1p/nWllGySJSUN8uMs+gUAN10Or95UBH0fpj6GfKgPmgC5EXieXG';
            script.crossOrigin = 'anonymous';
            script.onload = function() { resolve(window.JSZip); };
            script.onerror = function() { reject(new Error('Failed to load JSZip')); };
            document.head.appendChild(script);
        });
    }
    app.shared = {
        truncate,
        closeAllMenus,
        closeDeleteConfirm,
        showConfirmDialog,
        showDeleteConfirmDialog,
        createDebouncedSearch,
        registerListPageListeners,
        handleMenuToggle,
        showLoading,
        showEmpty,
        loadJSZip
    };
})(window.AdminApp);

(function(app) {
    const RANKS = [
        { level: 1, name: 'Spark', xp: 0, color: '#94a3b8' },
        { level: 2, name: 'Prompt Apprentice', xp: 50, color: '#60a5fa' },
        { level: 3, name: 'Token Tinkerer', xp: 150, color: '#34d399' },
        { level: 4, name: 'Context Crafter', xp: 400, color: '#60a5fa' },
        { level: 5, name: 'Neural Navigator', xp: 800, color: '#f59e0b' },
        { level: 6, name: 'Model Whisperer', xp: 1500, color: '#ec4899' },
        { level: 7, name: 'Pipeline Architect', xp: 3000, color: '#f97316' },
        { level: 8, name: 'Singularity Sage', xp: 5000, color: '#3b82f6' },
        { level: 9, name: 'Emergent Mind', xp: 8000, color: '#06b6d4' },
        { level: 10, name: 'Superintelligence', xp: 12000, color: '#eab308' },
    ];
    const ACHIEVEMENT_DEFS = {
        first_spark: { name: 'First Spark', description: 'Start your first AI session', category: 'First Steps', icon: '\u26A1' },
        first_tool: { name: 'Tool Time', description: 'Use your first AI skill', category: 'First Steps', icon: '\uD83D\uDD27' },
        first_agent: { name: 'Agent Whisperer', description: 'Interact with your first agent', category: 'First Steps', icon: '\uD83E\uDD16' },
        first_mcp: { name: 'Protocol Pioneer', description: 'Use an MCP tool', category: 'First Steps', icon: '\uD83D\uDD0C' },
        first_custom: { name: 'Skill Crafter', description: 'Create your first custom skill', category: 'First Steps', icon: '\u2728' },
        first_plugin: { name: 'Plugin Explorer', description: 'Use a non-common plugin', category: 'First Steps', icon: '\uD83D\uDD0D' },
        prompts_10: { name: 'Getting Started', description: 'Use 10 AI prompts', category: 'Volume', icon: '\uD83D\uDCCA' },
        prompts_50: { name: 'Warming Up', description: 'Use 50 AI prompts', category: 'Volume', icon: '\uD83D\uDD25' },
        prompts_100: { name: 'Century Club', description: 'Use 100 AI prompts', category: 'Volume', icon: '\uD83D\uDCAF' },
        prompts_250: { name: 'Quarter Thousand', description: '250 AI prompts', category: 'Volume', icon: '\uD83D\uDE80' },
        prompts_500: { name: 'Half K', description: '500 AI prompts', category: 'Volume', icon: '\u26A1' },
        prompts_1000: { name: 'The Thousandaire', description: '1,000 AI prompts', category: 'Volume', icon: '\uD83D\uDC51' },
        skills_5: { name: 'Skill Sampler', description: 'Try 5 different skills', category: 'Exploration', icon: '\uD83C\uDFAF' },
        skills_15: { name: 'Skill Collector', description: 'Try 15 different skills', category: 'Exploration', icon: '\uD83C\uDFC6' },
        skills_30: { name: 'Skill Connoisseur', description: 'Try 30 different skills', category: 'Exploration', icon: '\uD83C\uDF93' },
        plugins_3: { name: 'Plugin Hopper', description: 'Use 3 different plugins', category: 'Exploration', icon: '\uD83D\uDD00' },
        plugins_all: { name: 'Full Stack', description: 'Use all assigned plugins', category: 'Exploration', icon: '\uD83C\uDF1F' },
        master_skill: { name: 'Skill Master', description: 'Use one skill 50 times', category: 'Mastery', icon: '\uD83E\uDD47' },
        dept_champion: { name: 'Dept Champion', description: 'Top user in department', category: 'Mastery', icon: '\uD83C\uDFC5' },
        org_top10: { name: 'Top 10', description: 'Reach org top 10', category: 'Mastery', icon: '\uD83D\uDD1F' },
        custom_3: { name: 'Skill Builder', description: 'Create 3 custom skills', category: 'Mastery', icon: '\uD83D\uDEE0\uFE0F' },
        streak_3: { name: 'Three-peat', description: '3-day streak', category: 'Streaks', icon: '\uD83D\uDD25' },
        streak_7: { name: 'Week Warrior', description: '7-day streak', category: 'Streaks', icon: '\u2694\uFE0F' },
        streak_14: { name: 'Fortnight Force', description: '14-day streak', category: 'Streaks', icon: '\uD83D\uDCAA' },
        streak_30: { name: 'Monthly Machine', description: '30-day streak', category: 'Streaks', icon: '\uD83E\uDD16' },
        streak_60: { name: 'Unstoppable', description: '60-day streak', category: 'Streaks', icon: '\uD83C\uDF0B' },
        share_skill: { name: 'Skill Sharer', description: 'Skill adopted by another user', category: 'Social', icon: '\uD83E\uDD1D' },
        team_50: { name: 'Team Player', description: 'Dept hits 50 daily uses', category: 'Social', icon: '\uD83D\uDC65' },
    };
    const ROLES = ['admin', 'ceo', 'finance', 'sales', 'marketing', 'operations', 'hr', 'it', 'support'];
    const HOOK_EVENTS = ['PostToolUse', 'SessionStart', 'PreToolUse', 'Notification'];
    function getUserRank(xp) {
        let rank = RANKS[0];
        for (let i = RANKS.length - 1; i >= 0; i--) {
            if (xp >= RANKS[i].xp) { rank = RANKS[i]; break; }
        }
        return rank;
    }
    function getNextUserRank(currentRank) {
        const idx = RANKS.findIndex((r) => r.level === currentRank.level);
        if (idx < RANKS.length - 1) return RANKS[idx + 1];
        return null;
    }
    app.constants = {
        RANKS,
        ACHIEVEMENT_DEFS,
        ROLES,
        HOOK_EVENTS,
        getUserRank,
        getNextUserRank
    };
})(window.AdminApp);

(function(app) {
    const escapeHtml = app.escapeHtml;
    const shared = app.shared;
    /**
     * Generic list page factory. Creates a fully wired list page from config.
     *
     * config = {
     *   entityName:        'skill',
     *   pageKey:           'skills',
     *   searchInputId:     'skill-search',
     *   searchPlaceholder: 'Search skills...',
     *   newHref:           app.BASE + '/skills/edit/',
     *   newLabel:          '+ New Skill',
     *   apiPath:           '/skills',
     *   apiResponseKey:    'skills',           // key in response object, or null for array
     *   columns:           ['Name', 'Skill ID', 'Description', 'Status', ''],
     *   idAttr:            'skill-id',         // data attribute for delete/toggle
     *   filterFn:          (item, q) => bool,
     *   renderRow:         (item) => '<tr>...</tr>',
     *   hasToggle:         true,
     *   toggleApiPath:     (id) => '/skills/' + encodeURIComponent(id),
     *   toggleBody:        (enabled) => ({ enabled }),
     *   toggleSuccessMsg:  (enabled) => 'Skill ' + (enabled ? 'enabled' : 'disabled'),
     *   deleteDialogTitle: 'Delete Skill?',
     *   deleteApiPath:     (id) => '/skills/' + encodeURIComponent(id),
     *   deleteSuccessMsg:  'Skill deleted',
     * }
     */
    function createListPage(selector, config) {
        const root = document.querySelector(selector);
        if (!root) return;
        let allItems = [];
        let searchQuery = '';
        function getFiltered() {
            if (!searchQuery) return allItems;
            const q = searchQuery.toLowerCase();
            return allItems.filter((item) => config.filterFn(item, q));
        }
        function renderToolbar() {
            return '<div class="toolbar">' +
                '<div class="search-group">' +
                    '<input type="text" class="search-input" placeholder="' + escapeHtml(config.searchPlaceholder) + '" id="' + escapeHtml(config.searchInputId) + '" value="' + escapeHtml(searchQuery) + '">' +
                '</div>' +
                '<a href="' + config.newHref + '" class="btn btn-primary">' + escapeHtml(config.newLabel) + '</a>' +
            '</div>';
        }
        function renderTable(items) {
            if (!items.length) {
                return '<div class="empty-state"><p>No ' + escapeHtml(config.entityName) + 's match your search.</p></div>';
            }
            const headerCells = config.columns.map((col) => {
                const cls = col === '' ? ' class="col-actions"' : '';
                return '<th' + cls + '>' + escapeHtml(col) + '</th>';
            }).join('');
            const rows = items.map((item) => config.renderRow(item)).join('');
            return '<div class="table-container"><div class="table-scroll">' +
                '<table class="data-table">' +
                    '<thead><tr>' + headerCells + '</tr></thead>' +
                    '<tbody>' + rows + '</tbody>' +
                '</table>' +
            '</div></div>';
        }
        function renderPage() {
            root.innerHTML = renderToolbar() + renderTable(getFiltered());
        }
        async function loadItems() {
            app.shared.showLoading(root, 'Loading ' + config.entityName + 's...');
            try {
                const data = await app.api(config.apiPath);
                allItems = config.apiResponseKey ? (data[config.apiResponseKey] || data || []) : (data || []);
                renderPage();
            } catch (err) {
                root.innerHTML = '<div class="empty-state"><p>Failed to load ' + escapeHtml(config.entityName) + 's.</p></div>';
                app.Toast.show(err.message || 'Failed to load ' + config.entityName + 's', 'error');
            }
        }
        loadItems();
        shared.createDebouncedSearch(root, config.searchInputId, (value) => {
            searchQuery = value;
            renderPage();
        });
        root.addEventListener('click', (e) => {
            if (shared.handleMenuToggle(e)) return;
            const deleteBtn = e.target.closest('[data-action="delete"]');
            if (deleteBtn) {
                shared.closeAllMenus();
                const itemId = deleteBtn.getAttribute('data-' + config.idAttr);
                shared.showDeleteConfirmDialog(config.deleteDialogTitle, itemId);
            }
        });
        if (config.hasToggle) {
            root.addEventListener('change', async (e) => {
                const toggle = e.target.closest('[data-action="toggle"]');
                if (!toggle) return;
                const itemId = toggle.getAttribute('data-' + config.idAttr);
                const enabled = toggle.checked;
                try {
                    await app.api(config.toggleApiPath(itemId), {
                        method: 'PUT',
                        body: JSON.stringify(config.toggleBody(enabled))
                    });
                    app.Toast.show(config.toggleSuccessMsg(enabled), 'success');
                } catch (err) {
                    toggle.checked = !enabled;
                    app.Toast.show(err.message || 'Failed to update ' + config.entityName, 'error');
                }
            });
        }
        shared.registerListPageListeners(config.pageKey, {
            onDelete: async (itemId, confirmBtn) => {
                confirmBtn.disabled = true;
                confirmBtn.textContent = 'Deleting...';
                try {
                    await app.api(config.deleteApiPath(itemId), { method: 'DELETE' });
                    app.Toast.show(config.deleteSuccessMsg, 'success');
                    shared.closeDeleteConfirm();
                    loadItems();
                } catch (err) {
                    app.Toast.show(err.message || 'Failed to delete ' + config.entityName, 'error');
                    confirmBtn.disabled = false;
                    confirmBtn.textContent = 'Delete';
                }
            }
        });
    }
    app.shared.createListPage = createListPage;
})(window.AdminApp);

(function(app) {
    const escapeHtml = app.escapeHtml;
    /**
     * Render a filterable checklist with checkboxes.
     *
     * @param {string} id - Unique identifier for the checklist (used as input name)
     * @param {Array} items - Items to render (strings or objects with name/id)
     * @param {Array|Object} selected - Selected values as array or { val: true } map
     * @param {string} label - Label text
     * @param {Object} opts - Options: { hasSelectAll: bool }
     */
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
    /**
     * Attach filter input handlers for [data-filter-list] inputs.
     * Shows/hides .checklist-item elements based on data-item-name match.
     */
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
    /**
     * Get checked checkbox values from a form by input name.
     */
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
    var escapeHtml = app.escapeHtml;
    var shared = app.shared;
    /**
     * Generic edit/create page factory.
     *
     * config = {
     *   entityName:  'skill',
     *   listPath:    '/skills/',
     *   apiPath:     '/skills/',
     *   idParam:     'id',              // URL search param
     *   idField:     'skill_id',        // body field name for create
     *   formId:      'skill-form',
     *   renderForm:  (entity, extra) => '<form>...</form>',
     *   buildBody:   (form, formData) => ({...}),
     *   successMsg:  'Skill saved!',
     *   preload:     async () => data,  // optional: load prerequisite data
     * }
     */
    function createEditPage(selector, config) {
        var root = document.querySelector(selector);
        if (!root) return;
        var entityId = new URLSearchParams(window.location.search).get(config.idParam || 'id');
        var isEdit = !!entityId;
        async function load() {
            shared.showLoading(root, 'Loading ' + config.entityName + '...');
            try {
                var extra = config.preload ? await config.preload() : null;
                var entity = null;
                if (isEdit) {
                    entity = await app.api(config.apiPath + encodeURIComponent(entityId));
                    if (!entity) {
                        root.innerHTML = '<div class="detail-header"><a href="' + app.BASE + config.listPath + '" class="btn btn-secondary btn-sm">&larr; Back</a></div>' +
                            '<div class="empty-state"><p>' + escapeHtml(config.entityName) + ' not found.</p></div>';
                        return;
                    }
                }
                root.innerHTML = config.renderForm(entity, extra);
                attachFormHandler(root);
            } catch (err) {
                root.innerHTML = '<div class="empty-state"><p>Failed to load ' + escapeHtml(config.entityName) + '.</p></div>';
                app.Toast.show(err.message || 'Failed to load ' + config.entityName, 'error');
            }
        }
        function attachFormHandler(formRoot) {
            var form = formRoot.querySelector('#' + config.formId);
            if (!form) return;
            form.addEventListener('submit', async function(e) {
                e.preventDefault();
                var formData = new FormData(form);
                var body = config.buildBody(form, formData);
                var url, method;
                if (isEdit) {
                    url = config.apiPath + encodeURIComponent(entityId);
                    method = 'PUT';
                } else {
                    if (config.idField) {
                        body[config.idField] = formData.get(config.idField);
                    }
                    url = config.apiPath.replace(/\/$/, '');
                    method = 'POST';
                }
                try {
                    await app.api(url, { method: method, body: JSON.stringify(body) });
                    app.Toast.show(config.successMsg, 'success');
                    window.location.href = app.BASE + config.listPath;
                } catch (err) {
                    app.Toast.show(err.message || 'Failed to save ' + config.entityName, 'error');
                }
            });
        }
        if (isEdit) {
            load();
        } else {
            if (config.preload) {
                load();
            } else {
                root.innerHTML = config.renderForm(null, null);
                attachFormHandler(root);
            }
        }
    }
    app.shared.createEditPage = createEditPage;
})(window.AdminApp);

(function(app) {
    const escapeHtml = app.escapeHtml;
    function countAgentsAndMcp(plugins) {
        let agents = 0;
        let mcp = 0;
        plugins.forEach((p) => {
            const kind = (p.kind || p.type || '').toLowerCase();
            if (kind === 'mcp' || kind === 'mcp_server') {
                mcp++;
            } else {
                agents++;
            }
        });
        return { agents, mcp };
    }
    function renderStatsCards(users, plugins, stats) {
        const totalUsers = users.length;
        const now = Date.now();
        const oneDayMs = 24 * 60 * 60 * 1000;
        const activeToday = users.filter((u) => u.last_active && (now - new Date(u.last_active).getTime()) < oneDayMs).length;
        const totalPlugins = plugins.length;
        let totalSkills = 0;
        plugins.forEach((p) => {
            totalSkills += (p.skills || []).length;
        });
        const counts = countAgentsAndMcp(plugins);
        const eventsToday = stats ? stats.events_today : 0;
        const eventsWeek = stats ? stats.events_this_week : 0;
        const sessions = stats ? stats.total_sessions : 0;
        const errors = stats ? stats.error_count : 0;
        return '<div class="dashboard-stats">' +
            '<div class="stat-card">' +
                '<div class="label">Plugins</div>' +
                '<div class="value">' + totalPlugins + '</div>' +
                '<div class="kpi-subtitle">' + counts.agents + ' agents, ' + counts.mcp + ' MCP servers</div>' +
            '</div>' +
            '<div class="stat-card success">' +
                '<div class="label">Skills</div>' +
                '<div class="value">' + totalSkills + '</div>' +
                '<div class="kpi-subtitle">across ' + totalPlugins + ' plugins</div>' +
            '</div>' +
            '<div class="stat-card">' +
                '<div class="label">AI Uses Today</div>' +
                '<div class="value">' + eventsToday + '</div>' +
                '<div class="kpi-subtitle">' + eventsWeek + ' this week</div>' +
            '</div>' +
            '<div class="stat-card' + (errors > 0 ? ' error' : ' success') + '">' +
                '<div class="label">Sessions</div>' +
                '<div class="value">' + sessions + '</div>' +
                '<div class="kpi-subtitle">' + (errors > 0 ? errors + ' errors' : 'no errors') + '</div>' +
            '</div>' +
        '</div>';
    }
    function renderQuickActions() {
        return '<div class="toolbar">' +
            '<a href="/admin/plugins/" class="btn btn-secondary">Browse Plugins</a>' +
            '<a href="/admin/skills/" class="btn btn-secondary">Manage Skills</a>' +
            '<a href="/admin/export/" class="btn btn-primary">Export to Claude</a>' +
        '</div>';
    }
    function renderTimeline(timeline) {
        if (!timeline || !timeline.length) {
            return '<div class="activity-feed">' +
                '<div class="empty-state"><p>No AI activity recorded yet.</p></div>' +
            '</div>';
        }
        const items = timeline.map((ev) => {
            const initials = app.getUserInitials(ev.display_name || ev.user_id);
            const name = escapeHtml(ev.display_name || ('User-' + ev.user_id.slice(0, 8)));
            const tool = ev.tool_name
                ? '<span class="badge badge-blue">' + escapeHtml(ev.tool_name) + '</span>'
                : '<span class="badge badge-gray">' + escapeHtml(ev.event_type) + '</span>';
            const plugin = ev.plugin_id
                ? ' <span class="text-tertiary text-xs">via ' + escapeHtml(ev.plugin_id) + '</span>'
                : '';
            const time = app.formatRelativeTime(ev.created_at);
            const isError = ev.event_type.indexOf('error') >= 0 || ev.event_type.indexOf('fail') >= 0;
            const iconClass = isError ? 'activity-red' : 'activity-green';
            return '<div class="activity-item">' +
                '<div class="activity-icon ' + iconClass + '">' + escapeHtml(initials) + '</div>' +
                '<div class="activity-content">' +
                    '<span class="activity-text"><strong>' + name + '</strong> used ' + tool + plugin + '</span>' +
                    '<div class="activity-time">' + escapeHtml(time) + '</div>' +
                '</div>' +
            '</div>';
        }).join('');
        return '<div class="activity-feed">' + items + '</div>';
    }
    function renderTopUsers(topUsers) {
        if (!topUsers || !topUsers.length) {
            return '<div class="table-container"><div class="empty-state"><p>No AI usage yet.</p></div></div>';
        }
        const rows = topUsers.map((u, i) => {
            const rank = i + 1;
            const initials = app.getUserInitials(u.display_name || u.user_id);
            const name = escapeHtml(u.display_name || ('User-' + u.user_id.slice(0, 8)));
            const skill = u.top_skill
                ? '<span class="badge badge-blue">' + escapeHtml(u.top_skill) + '</span>'
                : '<span class="text-tertiary">-</span>';
            const rankClass = rank <= 3 ? ' leaderboard-rank-' + rank : '';
            return '<tr>' +
                '<td><span class="leaderboard-rank' + rankClass + '">' + rank + '</span></td>' +
                '<td><div class="user-cell"><div class="user-avatar">' + escapeHtml(initials) + '</div><span>' + name + '</span></div></td>' +
                '<td class="numeric">' + u.total_events + '</td>' +
                '<td>' + skill + '</td>' +
            '</tr>';
        }).join('');
        return '<div class="table-container"><div class="table-scroll">' +
            '<table class="data-table">' +
                '<thead><tr><th class="col-rank">#</th><th>User</th><th class="numeric">Events</th><th>Top Skill</th></tr></thead>' +
                '<tbody>' + rows + '</tbody>' +
            '</table>' +
        '</div></div>';
    }
    function renderPopularSkills(skills) {
        if (!skills || !skills.length) {
            return '<div class="card" class="p-5"><div class="empty-state"><p>No skill usage yet.</p></div></div>';
        }
        const maxCount = skills[0].count;
        const bars = skills.map((s) => {
            const pct = maxCount > 0 ? Math.round((s.count / maxCount) * 100) : 0;
            return '<div class="skill-bar-row">' +
                '<div class="skill-bar-label" title="' + escapeHtml(s.tool_name) + '">' + escapeHtml(s.tool_name) + '</div>' +
                '<div class="skill-bar-track">' +
                    '<div class="progress-bar"><div class="progress-fill progress-blue" style="width:' + pct + '%"></div></div>' +
                '</div>' +
                '<div class="skill-bar-count">' + s.count + '</div>' +
            '</div>';
        }).join('');
        return '<div class="card" class="p-5">' + bars + '</div>';
    }
    function renderDepartmentActivity(departments) {
        if (!departments || !departments.length) {
            return '<div class="card" class="p-5"><div class="empty-state"><p>No department data yet.</p></div></div>';
        }
        const maxCount = departments[0].count;
        const bars = departments.map((d) => {
            const pct = maxCount > 0 ? Math.round((d.count / maxCount) * 100) : 0;
            const label = d.department.charAt(0).toUpperCase() + d.department.slice(1);
            return '<div class="skill-bar-row">' +
                '<div class="skill-bar-label" title="' + escapeHtml(label) + '">' + escapeHtml(label) + '</div>' +
                '<div class="skill-bar-track">' +
                    '<div class="progress-bar"><div class="progress-fill progress-green" style="width:' + pct + '%"></div></div>' +
                '</div>' +
                '<div class="skill-bar-count">' + d.count + '</div>' +
            '</div>';
        }).join('');
        return '<div class="card" class="p-5">' + bars + '</div>';
    }
    function renderHourlyChart(hourlyData) {
        const hours = [];
        for (let i = 0; i < 24; i++) { hours[i] = 0; }
        let maxVal = 1;
        (hourlyData || []).forEach((h) => {
            hours[h.hour] = h.count;
            if (h.count > maxVal) maxVal = h.count;
        });
        const totalEvents = hours.reduce((sum, c) => sum + c, 0);
        if (totalEvents === 0) {
            return '<div class="card" class="p-5"><div class="empty-state"><p>No activity in the last 24 hours.</p></div></div>';
        }
        const bars = hours.map((count, i) => {
            const pct = Math.round((count / maxVal) * 100);
            const label = (i < 10 ? '0' : '') + i + ':00';
            return '<div class="mini-chart-bar" title="' + label + ': ' + count + ' events">' +
                '<div class="mini-chart-fill" style="height:' + pct + '%"></div>' +
                '<div class="mini-chart-label">' + (i % 3 === 0 ? i : '') + '</div>' +
            '</div>';
        }).join('');
        return '<div class="card" class="p-5">' +
            '<div class="mini-chart">' + bars + '</div>' +
        '</div>';
    }
    app.renderDashboard = async (selector) => {
        const root = document.querySelector(selector);
        if (!root) return;
        app.shared.showLoading(root, 'Loading dashboard...');
        try {
            const results = await Promise.allSettled([
                app.api('/users'),
                app.api('/plugins'),
                app.api('/dashboard')
            ]);
            const users = results[0].status === 'fulfilled' ? (results[0].value.users || results[0].value || []) : [];
            const plugins = results[1].status === 'fulfilled' ? (results[1].value.plugins || results[1].value || []) : [];
            const dash = results[2].status === 'fulfilled' ? results[2].value : {};
            root.innerHTML =
                renderStatsCards(users, plugins, dash.stats) +
                renderQuickActions() +
                '<div class="dashboard-two-col">' +
                    '<div>' +
                        '<div class="section-title">Activity Timeline</div>' +
                        renderTimeline(dash.timeline) +
                    '</div>' +
                    '<div>' +
                        '<div class="section-title">Top AI Users</div>' +
                        renderTopUsers(dash.top_users) +
                    '</div>' +
                '</div>' +
                '<div class="dashboard-two-col">' +
                    '<div>' +
                        '<div class="section-title">Popular Skills</div>' +
                        renderPopularSkills(dash.popular_skills) +
                    '</div>' +
                    '<div>' +
                        '<div class="section-title">Activity (Last 24h)</div>' +
                        renderHourlyChart(dash.hourly_activity) +
                    '</div>' +
                '</div>' +
                '<div class="dashboard-two-col">' +
                    '<div>' +
                        '<div class="section-title">Department Activity</div>' +
                        renderDepartmentActivity(dash.department_activity) +
                    '</div>' +
                    '<div></div>' +
                '</div>';
        } catch (err) {
            root.innerHTML = '<div class="empty-state"><p>Failed to load dashboard data.</p></div>';
            app.Toast.show(err.message || 'Failed to load dashboard', 'error');
        }
    };
})(window.AdminApp);

(function(app) {
    const escapeHtml = app.escapeHtml;
    function truncateJson(obj, maxLen) {
        if (!obj) return '-';
        const str = typeof obj === 'string' ? obj : JSON.stringify(obj);
        if (str.length <= (maxLen || 80)) return escapeHtml(str);
        return '<span title="' + escapeHtml(str) + '">' + escapeHtml(str.substring(0, maxLen || 80)) + '&hellip;</span>';
    }
    function renderHeader(user) {
        const name = user.display_name || user.user_id;
        const totalEvents = user.total_events || 0;
        const relTime = user.last_active ? app.formatRelativeTime(user.last_active) : 'Never';
        return '<div class="detail-header">' +
            '<a href="' + app.BASE + '/users/" class="btn btn-secondary btn-sm">&larr; Back</a>' +
        '</div>' +
        '<div class="dashboard-grid" style="grid-template-columns:repeat(3,1fr);margin-bottom:var(--space-6)">' +
            '<div class="stat-card">' +
                '<div class="label">User</div>' +
                '<div class="value" style="font-size:var(--text-lg)">' + escapeHtml(name) + '</div>' +
                '<div class="kpi-subtitle">' + escapeHtml(user.user_id) + '</div>' +
            '</div>' +
            '<div class="stat-card">' +
                '<div class="label">Total Events</div>' +
                '<div class="value">' + totalEvents + '</div>' +
            '</div>' +
            '<div class="stat-card">' +
                '<div class="label">Last Active</div>' +
                '<div class="value" style="font-size:var(--text-lg)">' + escapeHtml(relTime) + '</div>' +
            '</div>' +
        '</div>';
    }
    function renderProfileCard(user) {
        const rows = [];
        rows.push('<tr><td><strong>User ID</strong></td><td>' + escapeHtml(user.user_id) + '</td></tr>');
        if (user.display_name) {
            rows.push('<tr><td><strong>Display Name</strong></td><td>' + escapeHtml(user.display_name) + '</td></tr>');
        }
        if (user.email) {
            rows.push('<tr><td><strong>Email</strong></td><td>' + escapeHtml(user.email) + '</td></tr>');
        }
        if (user.department) {
            rows.push('<tr><td><strong>Department</strong></td><td><span class="badge badge-blue">' + escapeHtml(user.department) + '</span></td></tr>');
        }
        if (user.roles && user.roles.length) {
            const roleBadges = user.roles.map((r) => '<span class="badge badge-blue">' + escapeHtml(r) + '</span>').join(' ');
            rows.push('<tr><td><strong>Roles</strong></td><td>' + roleBadges + '</td></tr>');
        }
        if (user.is_active !== null && user.is_active !== undefined) {
            const statusBadge = user.is_active
                ? '<span class="badge badge-green">Active</span>'
                : '<span class="badge badge-gray">Inactive</span>';
            rows.push('<tr><td><strong>Status</strong></td><td>' + statusBadge + '</td></tr>');
        }
        rows.push('<tr><td><strong>Member Since</strong></td><td>' + escapeHtml(app.formatDate(user.created_at)) + '</td></tr>');
        return '<div class="card" style="margin-bottom:var(--space-6)">' +
            '<h3>Profile</h3>' +
            '<table class="data-table"><tbody>' + rows.join('') + '</tbody></table>' +
        '</div>';
    }
    function renderSkillsTable(skills) {
        if (!skills || !skills.length) {
            return '<div class="card" style="margin-bottom:var(--space-6)">' +
                '<h3>Custom Skills</h3>' +
                '<div class="empty-state"><p>No custom skills.</p></div>' +
            '</div>';
        }
        const rows = skills.map((s) => {
            const statusBadge = s.enabled
                ? '<span class="badge badge-green">Enabled</span>'
                : '<span class="badge badge-gray">Disabled</span>';
            const tags = (s.tags || []).map((t) => '<span class="badge badge-blue">' + escapeHtml(t) + '</span>').join(' ') || '-';
            return '<tr>' +
                '<td>' + escapeHtml(s.name) + '</td>' +
                '<td>' + escapeHtml(s.description || '-') + '</td>' +
                '<td>' + statusBadge + '</td>' +
                '<td>' + tags + '</td>' +
                '<td>' + escapeHtml(app.formatRelativeTime(s.updated_at)) + '</td>' +
            '</tr>';
        }).join('');
        return '<div class="card" style="margin-bottom:var(--space-6)">' +
            '<h3>Custom Skills</h3>' +
            '<div class="table-container"><div class="table-scroll">' +
                '<table class="data-table">' +
                    '<thead><tr><th>Name</th><th>Description</th><th>Status</th><th>Tags</th><th>Updated</th></tr></thead>' +
                    '<tbody>' + rows + '</tbody>' +
                '</table>' +
            '</div></div>' +
        '</div>';
    }
    function renderEventsTable(events) {
        if (!events || !events.length) {
            return '<div class="card">' +
                '<h3>Recent Activity</h3>' +
                '<div class="empty-state"><p>No events recorded.</p></div>' +
            '</div>';
        }
        const rows = events.map((ev) => {
            const relTime = app.formatRelativeTime(ev.created_at || ev.timestamp);
            const fullDate = app.formatDate(ev.created_at || ev.timestamp);
            return '<tr>' +
                '<td><span class="badge badge-blue">' + escapeHtml(ev.event_type || '-') + '</span></td>' +
                '<td>' + (ev.tool_name ? '<code style="font-size:var(--text-xs);background:var(--bg-surface-raised);padding:2px 6px;border-radius:var(--radius-xs)">' + escapeHtml(ev.tool_name) + '</code>' : '-') + '</td>' +
                '<td>' + escapeHtml(ev.plugin_id || '-') + '</td>' +
                '<td><span title="' + escapeHtml(fullDate) + '">' + escapeHtml(relTime) + '</span></td>' +
                '<td class="metadata-cell" style="font-family:monospace;font-size:var(--text-xs)">' + truncateJson(ev.metadata) + '</td>' +
            '</tr>';
        }).join('');
        return '<div class="card">' +
            '<h3>Recent Activity</h3>' +
            '<div class="table-container"><div class="table-scroll">' +
                '<table class="data-table">' +
                    '<thead><tr><th>Event Type</th><th>Tool</th><th>Plugin</th><th>Time</th><th>Metadata</th></tr></thead>' +
                    '<tbody>' + rows + '</tbody>' +
                '</table>' +
            '</div></div>' +
        '</div>';
    }
    function renderGamificationSection(gData) {
        const xp = gData.xp || 0;
        const rank = app.constants.getUserRank(xp);
        const nextRank = app.constants.getNextUserRank(rank);
        let progressPct = 100;
        if (nextRank) {
            const range = nextRank.xp - rank.xp;
            progressPct = range > 0 ? Math.min(100, Math.round(((xp - rank.xp) / range) * 100)) : 100;
        }
        const nextLabel = nextRank ? nextRank.xp : 'MAX';
        let html = '<div class="gamification-profile">' +
            '<h3>Gamification</h3>' +
            '<div class="card" style="margin-bottom:var(--space-4);padding:var(--space-5)">' +
                '<div style="display:flex;align-items:center;gap:var(--space-4);margin-bottom:var(--space-3)">' +
                    '<span class="rank-badge rank-' + rank.level + '" style="font-size:var(--text-sm);padding:var(--space-1) var(--space-3)">' + escapeHtml(rank.name) + '</span>' +
                    '<span style="font-size:var(--text-xs);color:var(--text-tertiary)">Level ' + rank.level + '</span>' +
                '</div>' +
                '<div style="font-size:var(--text-sm);color:var(--text-secondary);margin-bottom:var(--space-1)">' + xp + ' / ' + nextLabel + ' XP</div>' +
                '<div class="xp-progress"><div class="xp-progress-fill" style="width:' + progressPct + '%;background:' + rank.color + '"></div></div>' +
            '</div>' +
            '<div class="gamification-stats" style="margin-bottom:var(--space-6)">' +
                '<div class="stat-card"><div class="label">Total XP</div><div class="value">' + xp + '</div></div>' +
                '<div class="stat-card"><div class="label">Current Streak</div><div class="value">' + (gData.current_streak || 0) + 'd</div></div>' +
                '<div class="stat-card"><div class="label">Unique Skills</div><div class="value">' + (gData.unique_skills || 0) + '</div></div>' +
                '<div class="stat-card"><div class="label">Leaderboard</div><div class="value">#' + (gData.leaderboard_position || '-') + '</div></div>' +
            '</div>';
        // Achievement grid
        const userAchievements = gData.achievements || {};
        const cards = Object.keys(app.constants.ACHIEVEMENT_DEFS).map((key) => {
            const def = app.constants.ACHIEVEMENT_DEFS[key];
            const unlocked = !!userAchievements[key];
            const cls = unlocked ? 'achievement-card unlocked' : 'achievement-card locked';
            return '<div class="' + cls + '">' +
                '<div class="achievement-icon">' + def.icon + '</div>' +
                '<div style="font-weight:600;font-size:var(--text-xs);color:var(--text-primary)">' + escapeHtml(def.name) + '</div>' +
                '<div style="font-size:var(--text-xs);color:var(--text-tertiary)">' + escapeHtml(def.description) + '</div>' +
            '</div>';
        }).join('');
        html += '<div style="margin-bottom:var(--space-6)">' +
            '<div class="section-title">Achievements</div>' +
            '<div class="achievement-grid">' + cards + '</div>' +
        '</div></div>';
        return html;
    }
    app.renderUserDetail = (selector) => {
        const root = document.querySelector(selector);
        if (!root) return;
        const userId = new URLSearchParams(window.location.search).get('id');
        if (!userId) {
            root.innerHTML = '<div class="detail-header">' +
                '<a href="' + app.BASE + '/users/" class="btn btn-secondary btn-sm">&larr; Back</a>' +
            '</div>' +
            '<div class="empty-state"><p>No user ID provided.</p></div>';
            return;
        }
        root.innerHTML = '<div class="detail-header">' +
            '<a href="' + app.BASE + '/users/" class="btn btn-secondary btn-sm">&larr; Back</a>' +
        '</div>' +
        '<div class="loading-center"><div class="loading-spinner"></div></div>';
        async function load() {
            try {
                const user = await app.api('/users/' + encodeURIComponent(userId) + '/detail');
                const baseHtml =
                    renderHeader(user) +
                    renderProfileCard(user) +
                    renderSkillsTable(user.skills) +
                    renderEventsTable(user.recent_events);
                root.innerHTML = baseHtml +
                    '<div id="gamification-section"><div class="loading-center"><div class="loading-spinner"></div></div></div>';
                try {
                    const gData = await app.api('/gamification/user/' + encodeURIComponent(userId));
                    const gSection = document.getElementById('gamification-section');
                    if (gSection) gSection.innerHTML = renderGamificationSection(gData);
                } catch {
                    const gSection = document.getElementById('gamification-section');
                    if (gSection) gSection.innerHTML = '';
                }
            } catch (err) {
                root.innerHTML = '<div class="detail-header">' +
                    '<a href="' + app.BASE + '/users/" class="btn btn-secondary btn-sm">&larr; Back</a>' +
                '</div>' +
                '<div class="empty-state"><p>Failed to load user details.</p></div>';
                app.Toast.show(err.message || 'Failed to load user data', 'error');
            }
        }
        load();
    };
})(window.AdminApp);

(function(app) {
    const escapeHtml = app.escapeHtml;
    const DEPARTMENTS = ['', 'ceo', 'finance', 'sales', 'marketing', 'operations', 'hr', 'it', 'support'];
    const ROLES = ['admin', 'ceo', 'manager', 'finance', 'sales', 'marketing', 'operations', 'hr', 'it', 'support'];
    function openCreatePanel() {
        const deptOptions = DEPARTMENTS.map((d) => '<option value="' + d + '">' + (d || '-- Select --') + '</option>').join('');
        const roleCheckboxes = ROLES.map((r) => '<label class="checkbox-label"><input type="checkbox" name="roles" value="' + r + '"> ' + r + '</label>').join('');
        let overlay = document.getElementById('create-user-overlay');
        let panel = document.getElementById('create-user-panel');
        if (overlay && panel) {
            overlay.classList.add('open');
            panel.classList.add('open');
            const first = panel.querySelector('input');
            if (first) setTimeout(() => { first.focus(); }, 350);
            return;
        }
        overlay = document.createElement('div');
        overlay.className = 'panel-overlay';
        overlay.id = 'create-user-overlay';
        panel = document.createElement('div');
        panel.className = 'side-panel';
        panel.id = 'create-user-panel';
        panel.innerHTML =
            '<div class="panel-header">' +
                '<h2>Add User</h2>' +
                '<button class="panel-close">&times;</button>' +
            '</div>' +
            '<div class="panel-body">' +
                '<div class="form-group"><label>User ID</label><input class="field-input" type="text" id="new-user-id" placeholder="username or email"></div>' +
                '<div class="form-group"><label>Display Name</label><input class="field-input" type="text" id="new-user-name" placeholder="Full Name"></div>' +
                '<div class="form-group"><label>Email</label><input class="field-input" type="email" id="new-user-email" placeholder="user@company.com"></div>' +
                '<div class="form-group"><label>Department</label><select class="field-input" id="new-user-dept">' + deptOptions + '</select></div>' +
                '<div class="form-group"><label>Roles</label><div class="checkbox-group">' + roleCheckboxes + '</div></div>' +
            '</div>' +
            '<div class="panel-footer">' +
                '<button class="btn btn-secondary" data-action="cancel">Cancel</button>' +
                '<button class="btn btn-primary" data-action="save">Create User</button>' +
            '</div>';
        document.body.appendChild(overlay);
        document.body.appendChild(panel);
        requestAnimationFrame(() => {
            requestAnimationFrame(() => {
                overlay.classList.add('open');
                panel.classList.add('open');
            });
        });
        const first = panel.querySelector('input');
        if (first) setTimeout(() => { first.focus(); }, 400);
        return { overlay, panel };
    }
    function closeCreatePanel() {
        const overlay = document.getElementById('create-user-overlay');
        const panel = document.getElementById('create-user-panel');
        if (panel) panel.classList.remove('open');
        if (overlay) overlay.classList.remove('open');
        setTimeout(() => {
            if (overlay) overlay.remove();
            if (panel) panel.remove();
        }, 350);
    }
    function bindCreatePanelEvents(refreshFn) {
        document.addEventListener('click', async (e) => {
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
                const dept = document.getElementById('new-user-dept').value;
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
                            email,
                            department: dept,
                            roles
                        })
                    });
                    app.Toast.show('User created', 'success');
                    closeCreatePanel();
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
    const escapeHtml = app.escapeHtml;
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
    function renderUsersTable(users) {
        if (!users.length) {
            return '<div class="empty-state"><p>No users found.</p></div>';
        }
        const rows = users.map((u) => {
            const relTime = u.last_active ? app.formatRelativeTime(u.last_active) : 'Never';
            const fullDate = u.last_active ? app.formatDate(u.last_active) : '';
            const href = app.BASE + '/user/?id=' + encodeURIComponent(u.user_id);
            const displayName = u.display_name || u.user_id;
            const initials = app.getUserInitials(displayName);
            const dept = u.department || '-';
            const roles = (u.roles && u.roles.length) ? u.roles.join(', ') : '-';
            const isActive = u.is_active !== false;
            const statusBadge = isActive
                ? '<span class="badge badge-green">Active</span>'
                : '<span class="badge badge-gray">Inactive</span>';
            return '<tr data-user-name="' + escapeHtml(displayName.toLowerCase()) + '" data-user-email="' + escapeHtml((u.email || '').toLowerCase()) + '">' +
                '<td><a href="' + href + '" class="user-cell">' +
                    '<span class="avatar avatar-sm">' + escapeHtml(initials) + '</span>' +
                    '<span>' + escapeHtml(displayName) + '</span>' +
                '</a></td>' +
                '<td>' + escapeHtml(u.email || '-') + '</td>' +
                '<td>' + (dept !== '-' ? '<span class="badge badge-blue">' + escapeHtml(dept) + '</span>' : '-') + '</td>' +
                '<td>' + escapeHtml(roles) + '</td>' +
                '<td>' + statusBadge + '</td>' +
                '<td><span title="' + escapeHtml(fullDate) + '">' + escapeHtml(relTime) + '</span></td>' +
                '<td class="col-actions">' +
                    '<button class="btn-actions-trigger" data-user-id="' + escapeHtml(u.user_id) + '" aria-label="Actions" aria-haspopup="true" aria-expanded="false" type="button">&#8943;</button>' +
                '</td>' +
            '</tr>';
        }).join('');
        return '<div class="table-container"><div class="table-scroll">' +
            '<table class="data-table">' +
                '<thead><tr>' +
                    '<th>User</th>' +
                    '<th>Email</th>' +
                    '<th>Department</th>' +
                    '<th>Roles</th>' +
                    '<th>Status</th>' +
                    '<th>Last Active</th>' +
                    '<th class="col-actions"></th>' +
                '</tr></thead>' +
                '<tbody>' + rows + '</tbody>' +
            '</table>' +
        '</div></div>';
    }
    function filterTable(query) {
        const q = query.toLowerCase().trim();
        const rows = document.querySelectorAll('.data-table tbody tr');
        for (let i = 0; i < rows.length; i++) {
            const name = rows[i].getAttribute('data-user-name') || '';
            const email = rows[i].getAttribute('data-user-email') || '';
            const match = !q || name.indexOf(q) !== -1 || email.indexOf(q) !== -1;
            rows[i].style.display = match ? '' : 'none';
        }
    }
    function bindPopupEvents(root, users, refreshFn) {
        root.addEventListener('click', (e) => {
            const trigger = e.target.closest('.btn-actions-trigger');
            if (!trigger) return;
            e.stopPropagation();
            const userId = trigger.dataset.userId;
            const portal = getOrCreatePortal();
            const isOpen = portal.classList.contains('open') && activePopupId === userId;
            closeAllPopups();
            if (isOpen) return;
            const user = users.find((u) => u.user_id === userId);
            if (!user) return;
            const isActive = user.is_active !== false;
            const toggleLabel = isActive ? 'Deactivate' : 'Activate';
            const toggleIcon = isActive ? '&#10006;' : '&#10004;';
            const toggleClass = isActive ? ' actions-popup-item--danger' : '';
            portal.innerHTML =
                '<button class="actions-popup-item" data-action="edit" data-user-id="' + escapeHtml(userId) + '"><span class="popup-icon">&#9998;</span> Edit User</button>' +
                '<div class="actions-popup-separator"></div>' +
                '<button class="actions-popup-item' + toggleClass + '" data-action="toggle" data-user-id="' + escapeHtml(userId) + '"><span class="popup-icon">' + toggleIcon + '</span> ' + toggleLabel + '</button>';
            activePopupId = userId;
            portal.classList.add('open');
            trigger.classList.add('active');
            trigger.setAttribute('aria-expanded', 'true');
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
            portal.querySelectorAll('.actions-popup-item').forEach((item) => {
                item.addEventListener('click', async (ev) => {
                    ev.stopPropagation();
                    const action = item.dataset.action;
                    const itemUserId = item.dataset.userId;
                    closeAllPopups();
                    if (action === 'edit') {
                        window.location.href = app.BASE + '/user/?id=' + encodeURIComponent(itemUserId);
                    } else if (action === 'toggle') {
                        const targetUser = users.find((u) => u.user_id === itemUserId);
                        const currentlyActive = targetUser && targetUser.is_active !== false;
                        if (currentlyActive) {
                            showConfirmDialog(
                                'Deactivate User?',
                                'This will prevent ' + (targetUser.display_name || itemUserId) + ' from accessing the system. You can reactivate them later.',
                                'Deactivate',
                                async () => {
                                    try {
                                        await app.api('/users/' + encodeURIComponent(itemUserId), {
                                            method: 'PUT',
                                            body: JSON.stringify({ is_active: false })
                                        });
                                        app.Toast.show('User deactivated', 'success');
                                        refreshFn();
                                    } catch (err) {
                                        app.Toast.show(err.message || 'Failed to deactivate user', 'error');
                                    }
                                }
                            );
                        } else {
                            try {
                                await app.api('/users/' + encodeURIComponent(itemUserId), {
                                    method: 'PUT',
                                    body: JSON.stringify({ is_active: true })
                                });
                                app.Toast.show('User activated', 'success');
                                refreshFn();
                            } catch (err) {
                                app.Toast.show(err.message || 'Failed to activate user', 'error');
                            }
                        }
                    }
                });
            });
        });
        document.addEventListener('click', (e) => {
            if (!e.target.closest('.btn-actions-trigger') && !e.target.closest('#user-actions-popup')) {
                closeAllPopups();
            }
        });
        const tableScroll = root.querySelector('.table-scroll');
        if (tableScroll) {
            tableScroll.addEventListener('scroll', () => {
                closeAllPopups();
            });
        }
    }
    app.renderUsers = (selector) => {
        const root = document.querySelector(selector);
        if (!root) return;
        let currentUsers = [];
        async function refresh() {
            try {
                let users = await app.api('/users');
                users = users || [];
                currentUsers = users;
                const toolbar = '<div class="toolbar">' +
                    '<div class="search-group">' +
                        '<input type="text" class="search-input" placeholder="Search users..." id="user-search">' +
                    '</div>' +
                    '<button class="btn btn-primary" id="btn-add-user">+ Add User</button>' +
                '</div>';
                root.innerHTML = toolbar + renderUsersTable(users);
                bindPopupEvents(root, users, refresh);
            } catch (err) {
                root.innerHTML = '<div class="empty-state"><p>Failed to load users.</p></div>';
                app.Toast.show(err.message || 'Failed to load users', 'error');
            }
        }
        // Register event listeners ONCE via delegation on root
        root.addEventListener('input', (e) => {
            if (e.target.id === 'user-search') {
                filterTable(e.target.value);
            }
        });
        root.addEventListener('click', (e) => {
            if (e.target.closest('#btn-add-user')) {
                app.usersPanel.open();
            }
        });
        app.usersPanel.bindEvents(refresh);
        refresh();
    };
})(window.AdminApp);

(function(app) {
    const escapeHtml = app.escapeHtml;
    function renderSkillsTab(plugin) {
        const skills = plugin.skills || [];
        if (!skills.length) {
            return '<div class="empty-state"><p>No skills assigned to this plugin.</p></div>';
        }
        return skills.map((s) => {
            const sourceClass = s.source === 'custom' ? 'badge-green' : 'badge-blue';
            return '<div class="detail-item">' +
                '<div class="detail-item-info">' +
                    '<div class="detail-item-name">' + escapeHtml(s.name) +
                        ' <span class="badge ' + sourceClass + '">' + escapeHtml(s.source || 'system') + '</span>' +
                    '</div>' +
                    (s.description ? '<div class="detail-item-desc" style="font-size:var(--text-sm);color:var(--text-secondary);margin-top:var(--space-1)">' + escapeHtml(s.description) + '</div>' : '') +
                '</div>' +
                (s.command ? '<span style="font-size:var(--text-xs);color:var(--text-tertiary)">' + escapeHtml(s.command) + '</span>' : '') +
            '</div>';
        }).join('');
    }
    function renderAgentsTab(plugin) {
        const agents = plugin.agents || [];
        if (!agents.length) {
            return '<div class="empty-state"><p>No agents assigned to this plugin.</p></div>';
        }
        return agents.map((a) => '<div class="detail-item">' +
                '<div class="detail-item-info">' +
                    '<div class="detail-item-name">' + escapeHtml(a.name) + '</div>' +
                    (a.description ? '<div class="detail-item-desc" style="font-size:var(--text-sm);color:var(--text-secondary);margin-top:var(--space-1)">' + escapeHtml(a.description) + '</div>' : '') +
                '</div>' +
            '</div>').join('');
    }
    function renderMcpTab(plugin) {
        const servers = plugin.mcp_servers || [];
        if (!servers.length) {
            return '<div class="empty-state"><p>No MCP servers assigned to this plugin.</p></div>';
        }
        return servers.map((m) => '<div class="detail-item">' +
                '<div class="detail-item-info">' +
                    '<div class="detail-item-name">' + escapeHtml(m) + '</div>' +
                '</div>' +
            '</div>').join('');
    }
    function renderHooksTab(plugin) {
        const hooks = plugin.hooks || [];
        if (!hooks.length) {
            return '<div class="empty-state"><p>No hooks configured for this plugin.</p></div>';
        }
        return hooks.map((h) => {
            const asyncBadge = h.is_async ? ' <span class="badge badge-gray">async</span>' : '';
            return '<div class="detail-item">' +
                '<div class="detail-item-info">' +
                    '<div class="detail-item-name">' +
                        '<span class="badge badge-blue">' + escapeHtml(h.event) + '</span>' +
                        ' <span style="color:var(--text-secondary);font-size:var(--text-sm)">matcher:</span> ' +
                        '<code style="background:var(--bg-surface-raised);padding:2px 6px;border-radius:var(--radius-xs);font-size:var(--text-xs)">' + escapeHtml(h.matcher) + '</code>' +
                        asyncBadge +
                    '</div>' +
                    '<div class="detail-item-desc" style="font-size:var(--text-xs);color:var(--text-tertiary);margin-top:var(--space-1);font-family:monospace">' + escapeHtml(h.command) + '</div>' +
                '</div>' +
            '</div>';
        }).join('');
    }
    function renderPluginDetails(plugin, activeTab) {
        const skillCount = (plugin.skills || []).length;
        const agentCount = (plugin.agents || []).length;
        const mcpCount = (plugin.mcp_servers || []).length;
        const hookCount = (plugin.hooks || []).length;
        let tabs = '';
        tabs += '<button class="plugin-tab' + (activeTab === 'skills' ? ' active' : '') + '" data-tab="skills" data-plugin-owner="' + escapeHtml(plugin.id) + '">Skills <span class="plugin-tab-count">' + skillCount + '</span></button>';
        tabs += '<button class="plugin-tab' + (activeTab === 'agents' ? ' active' : '') + '" data-tab="agents" data-plugin-owner="' + escapeHtml(plugin.id) + '">Agents <span class="plugin-tab-count">' + agentCount + '</span></button>';
        tabs += '<button class="plugin-tab' + (activeTab === 'mcp' ? ' active' : '') + '" data-tab="mcp" data-plugin-owner="' + escapeHtml(plugin.id) + '">MCP Servers <span class="plugin-tab-count">' + mcpCount + '</span></button>';
        tabs += '<button class="plugin-tab' + (activeTab === 'hooks' ? ' active' : '') + '" data-tab="hooks" data-plugin-owner="' + escapeHtml(plugin.id) + '">Hooks <span class="plugin-tab-count">' + hookCount + '</span></button>';
        let content = '';
        if (activeTab === 'skills') content = renderSkillsTab(plugin);
        else if (activeTab === 'agents') content = renderAgentsTab(plugin);
        else if (activeTab === 'hooks') content = renderHooksTab(plugin);
        else content = renderMcpTab(plugin);
        return '<div style="border-top:1px solid var(--border-subtle);padding:var(--space-4)">' +
            '<div class="plugin-tabs" style="display:flex;gap:var(--space-2);margin-bottom:var(--space-4)">' + tabs + '</div>' +
            '<div class="plugin-tab-content">' + content + '</div>' +
        '</div>';
    }
    app.pluginDetails = {
        render: renderPluginDetails
    };
})(window.AdminApp);

(function(app) {
    const escapeHtml = app.escapeHtml;
    let plugins = [];
    let activeTab = 'skills';
    let searchQuery = '';
    let root = null;
    const expandedPlugins = {};
    function renderToolbar() {
        return '<div class="toolbar">' +
            '<div class="search-group">' +
                '<input type="text" class="search-input" placeholder="Search plugins..." id="plugin-search" value="' + escapeHtml(searchQuery) + '">' +
            '</div>' +
            '<a href="' + (app.BASE || '/admin') + '/plugins/create/" class="btn btn-primary">+ Create Plugin</a>' +
        '</div>';
    }
    function renderPluginCard(plugin) {
        const skillCount = (plugin.skills || []).length;
        const agentCount = (plugin.agents || []).length;
        const mcpCount = (plugin.mcp_servers || []).length;
        const hookCount = (plugin.hooks || []).length;
        const isExpanded = !!expandedPlugins[plugin.id];
        let detailsHtml = '';
        if (isExpanded) {
            detailsHtml = '<div class="plugin-details">' +
                app.pluginDetails.render(plugin, activeTab) +
            '</div>';
        }
        let actionsHtml = '';
        if (plugin.id !== 'custom') {
            actionsHtml = '<div class="actions-menu" data-actions-for="' + escapeHtml(plugin.id) + '">' +
                '<button class="actions-trigger" data-action="menu" title="Actions">&#8942;</button>' +
                '<div class="actions-dropdown">' +
                    '<a href="' + (app.BASE || '/admin') + '/plugins/edit/?id=' + encodeURIComponent(plugin.id) + '" class="actions-item">Edit</a>' +
                    '<button class="actions-item" data-generate-plugin="' + escapeHtml(plugin.id) + '" data-platform="unix">Generate (macOS/Linux)</button>' +
                    '<button class="actions-item" data-generate-plugin="' + escapeHtml(plugin.id) + '" data-platform="windows">Generate (Windows)</button>' +
                    '<div class="actions-popup-separator"></div>' +
                    '<button class="actions-item actions-item-danger" data-delete-plugin="' + escapeHtml(plugin.id) + '">Delete</button>' +
                '</div>' +
            '</div>';
        }
        return '<div class="plugin-card" style="border-left:3px solid var(--accent)" data-plugin-id="' + escapeHtml(plugin.id) + '">' +
            '<div class="plugin-header" data-toggle-plugin="' + escapeHtml(plugin.id) + '">' +
                '<div style="flex:1;min-width:0">' +
                    '<h3 style="margin:0;font-size:var(--text-base);font-weight:600;color:var(--text-primary)">' + escapeHtml(plugin.name) + '</h3>' +
                    (plugin.description ? '<p style="margin:var(--space-1) 0 0;font-size:var(--text-sm);color:var(--text-secondary)">' + escapeHtml(plugin.description) + '</p>' : '') +
                    '<div class="plugin-meta" style="display:flex;gap:var(--space-2);margin-top:var(--space-2)">' +
                        '<span class="badge badge-blue">' + skillCount + ' skill' + (skillCount !== 1 ? 's' : '') + '</span>' +
                        '<span class="badge badge-gray">' + agentCount + ' agent' + (agentCount !== 1 ? 's' : '') + '</span>' +
                        (mcpCount > 0 ? '<span class="badge badge-yellow">' + mcpCount + ' MCP</span>' : '') +
                        (hookCount > 0 ? '<span class="badge badge-green">' + hookCount + ' hook' + (hookCount !== 1 ? 's' : '') + '</span>' : '') +
                    '</div>' +
                '</div>' +
                '<div class="plugin-actions" style="display:flex;align-items:center;gap:var(--space-2)">' +
                    actionsHtml +
                    '<span class="expand-icon" style="font-size:var(--text-sm);color:var(--text-tertiary);transition:transform var(--duration-fast) var(--ease-out);display:inline-block;' + (isExpanded ? 'transform:rotate(180deg)' : '') + '">&#9660;</span>' +
                '</div>' +
            '</div>' +
            detailsHtml +
        '</div>';
    }
    function getFilteredPlugins() {
        if (!searchQuery) return plugins;
        const q = searchQuery.toLowerCase();
        return plugins.filter((p) => p.name.toLowerCase().indexOf(q) >= 0 ||
                   (p.description || '').toLowerCase().indexOf(q) >= 0);
    }
    function renderAll() {
        const filtered = getFilteredPlugins();
        const listHtml = filtered.length > 0
            ? filtered.map(renderPluginCard).join('')
            : '<div class="empty-state"><p>No plugins match your search.</p></div>';
        root.innerHTML = renderToolbar() +
            '<div class="plugins-list" style="display:flex;flex-direction:column;gap:var(--space-3)">' + listHtml + '</div>';
    }
    const closeAllMenus = app.shared.closeAllMenus;
    function showDeleteConfirm(pluginId) {
        const overlay = document.createElement('div');
        overlay.className = 'confirm-overlay';
        overlay.id = 'delete-confirm';
        overlay.innerHTML = '<div class="confirm-dialog">' +
            '<h3 style="margin:0 0 var(--space-3)">Delete Plugin?</h3>' +
            '<p style="margin:0 0 var(--space-2);color:var(--text-secondary);font-size:var(--text-sm)">You are about to delete <strong>' + escapeHtml(pluginId) + '</strong>.</p>' +
            '<p style="margin:0 0 var(--space-5);color:var(--text-secondary);font-size:var(--text-sm)">This will remove the plugin directory and all its configuration. This action cannot be undone.</p>' +
            '<div style="display:flex;gap:var(--space-3);justify-content:flex-end">' +
                '<button class="btn btn-secondary" data-confirm-cancel>Cancel</button>' +
                '<button class="btn btn-danger" data-confirm-delete="' + escapeHtml(pluginId) + '">Delete Plugin</button>' +
            '</div>' +
        '</div>';
        document.body.appendChild(overlay);
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
                    const data = await app.api('/plugins');
                    plugins = data.plugins || data || [];
                    renderAll();
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
        if (btn) {
            btn.disabled = true;
            btn.textContent = 'Generating...';
        }
        try {
            const [data, JSZip] = await Promise.all([
                app.api('/export?plugin=' + encodeURIComponent(pluginId) + '&platform=' + encodeURIComponent(platform)),
                app.shared.loadJSZip()
            ]);
            const zip = new JSZip();
            const items = data.plugins || data.bundles || [];
            const bundle = items.find((b) => b.id === pluginId || b.plugin_id === pluginId);
            if (!bundle || !bundle.files) {
                throw new Error('No files found in export');
            }
            bundle.files.forEach((f) => {
                zip.file(f.path, f.content);
            });
            const blob = await zip.generateAsync({ type: 'blob' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = pluginId + '.zip';
            a.click();
            URL.revokeObjectURL(url);
            app.Toast.show('Plugin zip generated', 'success');
        } catch (err) {
            app.Toast.show(err.message || 'Export failed', 'error');
        } finally {
            if (btn) {
                btn.disabled = false;
                btn.textContent = 'Generate';
            }
            closeAllMenus();
        }
    }
    app.renderPlugins = async (selector) => {
        root = document.querySelector(selector);
        if (!root) return;
        app.shared.showLoading(root, 'Loading plugins...');
        try {
            const data = await app.api('/plugins');
            plugins = data.plugins || data || [];
            if (!plugins.length) {
                root.innerHTML = '<div class="empty-state"><p>No plugins assigned to your account.</p></div>';
                return;
            }
            renderAll();
        } catch (err) {
            root.innerHTML = '<div class="empty-state"><p>Failed to load plugins.</p></div>';
            app.Toast.show(err.message || 'Failed to load plugins', 'error');
        }
        root.addEventListener('click', (e) => {
            if (app.shared.handleMenuToggle(e)) return;
            // Dropdown item clicks should not toggle expand
            if (e.target.closest('.actions-dropdown')) {
                e.stopPropagation();
            }
            // Generate plugin (export)
            const generateBtn = e.target.closest('[data-generate-plugin]');
            if (generateBtn) {
                e.stopPropagation();
                const platform = generateBtn.getAttribute('data-platform') || 'unix';
                handleExport(generateBtn.getAttribute('data-generate-plugin'), generateBtn, platform);
                return;
            }
            // Delete plugin
            const deletePluginBtn = e.target.closest('[data-delete-plugin]');
            if (deletePluginBtn) {
                e.stopPropagation();
                closeAllMenus();
                showDeleteConfirm(deletePluginBtn.getAttribute('data-delete-plugin'));
                return;
            }
            // Toggle plugin expand/collapse
            const toggleBtn = e.target.closest('[data-toggle-plugin]');
            if (toggleBtn && !e.target.closest('.actions-menu') && !e.target.closest('[data-tab]')) {
                const id = toggleBtn.getAttribute('data-toggle-plugin');
                if (expandedPlugins[id]) {
                    delete expandedPlugins[id];
                } else {
                    expandedPlugins[id] = true;
                    activeTab = 'skills';
                }
                renderAll();
                restoreSearchFocus();
                return;
            }
            // Tab switching
            const tab = e.target.closest('[data-tab]');
            if (tab) {
                const newTab = tab.getAttribute('data-tab');
                if (activeTab !== newTab) {
                    activeTab = newTab;
                    renderAll();
                    restoreSearchFocus();
                }
                return;
            }
        });
        app.shared.createDebouncedSearch(root, 'plugin-search', (value) => {
            searchQuery = value;
            renderAll();
        });
        function restoreSearchFocus() {
            const input = document.getElementById('plugin-search');
            if (input) {
                input.focus();
                input.selectionStart = input.selectionEnd = input.value.length;
            }
        }
        app.shared.registerListPageListeners('plugins', {});
    };
})(window.AdminApp);

(function(app) {
    const escapeHtml = app.escapeHtml;
    const ROLES = app.constants.ROLES;
    function renderForm(plugin, allSkills, allAgents, allMcpServers) {
        const roles = (plugin.roles || []);
        const keywords = (plugin.keywords || []).join(', ');
        const authorName = (plugin.author && plugin.author.name) ? plugin.author.name : '';
        const rolesHtml = ROLES.map((r) => {
            const checked = roles.indexOf(r) >= 0 ? ' checked' : '';
            return '<label style="display:inline-flex;align-items:center;gap:var(--space-1);margin-right:var(--space-3);font-size:var(--text-sm);cursor:pointer">' +
                '<input type="checkbox" name="roles" value="' + escapeHtml(r) + '"' + checked + '>' +
                escapeHtml(r) +
            '</label>';
        }).join('');
        return '<div class="detail-header">' +
            '<a href="' + app.BASE + '/plugins/" class="btn btn-secondary btn-sm">&larr; Back to Plugins</a>' +
        '</div>' +
        '<div class="card">' +
            '<form id="plugin-edit-form">' +
                '<div class="form-group">' +
                    '<label class="field-label">Plugin ID</label>' +
                    '<input class="field-input" name="plugin_id" readonly value="' + escapeHtml(plugin.id) + '">' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label">Name</label>' +
                    '<input class="field-input" name="name" required value="' + escapeHtml(plugin.name) + '">' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label">Description</label>' +
                    '<textarea class="field-input" name="description" rows="3">' + escapeHtml(plugin.description || '') + '</textarea>' +
                '</div>' +
                '<div style="display:flex;gap:var(--space-3)">' +
                    '<div class="form-group" style="flex:1">' +
                        '<label class="field-label">Version</label>' +
                        '<input class="field-input" name="version" value="' + escapeHtml(plugin.version || '0.1.0') + '">' +
                    '</div>' +
                    '<div class="form-group" style="flex:1">' +
                        '<label class="field-label">Category</label>' +
                        '<input class="field-input" name="category" value="' + escapeHtml(plugin.category || '') + '">' +
                    '</div>' +
                '</div>' +
                '<div class="form-group">' +
                    '<label style="display:inline-flex;align-items:center;gap:var(--space-2);font-size:var(--text-sm);cursor:pointer">' +
                        '<input type="checkbox" name="enabled"' + (plugin.enabled !== false ? ' checked' : '') + '>' +
                        'Enabled' +
                    '</label>' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label">Keywords <span style="font-weight:400;color:var(--text-tertiary)">(comma-separated)</span></label>' +
                    '<input class="field-input" name="keywords" value="' + escapeHtml(keywords) + '">' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label">Author Name</label>' +
                    '<input class="field-input" name="author_name" value="' + escapeHtml(authorName) + '">' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label">Roles</label>' +
                    '<div style="display:flex;flex-wrap:wrap;gap:var(--space-1);padding:var(--space-2) 0">' + rolesHtml + '</div>' +
                '</div>' +
                app.formUtils.renderChecklist('skills', allSkills, plugin.skills || [], 'Skills') +
                app.formUtils.renderChecklist('agents', allAgents, plugin.agents || [], 'Agents') +
                app.formUtils.renderChecklist('mcp_servers', allMcpServers, plugin.mcp_servers || [], 'MCP Servers') +
                '<div style="display:flex;gap:var(--space-3);margin-top:var(--space-6)">' +
                    '<button type="submit" class="btn btn-primary">Save</button>' +
                    '<a href="' + app.BASE + '/plugins/" class="btn btn-secondary">Cancel</a>' +
                    '<button type="button" class="btn btn-danger" id="btn-delete-plugin" style="margin-left:auto">Delete</button>' +
                '</div>' +
            '</form>' +
        '</div>';
    }
    function attachFormHandler(root, pluginId) {
        const form = root.querySelector('#plugin-edit-form');
        if (!form) return;
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
                keywords,
                author: { name: formData.get('author_name') || '' },
                roles: app.formUtils.getCheckedValues(form, 'roles'),
                skills: app.formUtils.getCheckedValues(form, 'skills'),
                agents: app.formUtils.getCheckedValues(form, 'agents'),
                mcp_servers: app.formUtils.getCheckedValues(form, 'mcp_servers')
            };
            const submitBtn = form.querySelector('button[type="submit"]');
            submitBtn.disabled = true;
            submitBtn.textContent = 'Saving...';
            try {
                await app.api('/plugins/' + encodeURIComponent(pluginId), {
                    method: 'PUT',
                    body: JSON.stringify(body)
                });
                app.Toast.show('Plugin saved!', 'success');
                window.location.href = app.BASE + '/plugins/';
            } catch (err) {
                app.Toast.show(err.message || 'Failed to save plugin', 'error');
                submitBtn.disabled = false;
                submitBtn.textContent = 'Save';
            }
        });
        const deleteBtn = root.querySelector('#btn-delete-plugin');
        if (deleteBtn) {
            deleteBtn.addEventListener('click', () => {
                app.shared.showConfirmDialog('Delete Plugin?', 'Are you sure you want to delete this plugin? This cannot be undone.', 'Delete', async () => {
                    deleteBtn.disabled = true;
                    deleteBtn.textContent = 'Deleting...';
                    try {
                        await app.api('/plugins/' + encodeURIComponent(pluginId), {
                            method: 'DELETE'
                        });
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
    }
    app.renderPluginEditor = (selector) => {
        const root = document.querySelector(selector);
        if (!root) return;
        const pluginId = new URLSearchParams(window.location.search).get('id');
        if (!pluginId) {
            root.innerHTML = '<div class="detail-header"><a href="' + app.BASE + '/plugins/" class="btn btn-secondary btn-sm">&larr; Back to Plugins</a></div>' +
                '<div class="empty-state"><p>No plugin ID specified.</p></div>';
            return;
        }
        app.shared.showLoading(root, 'Loading plugin...');
        async function load() {
            try {
                const results = await Promise.all([
                    app.api('/plugins/' + encodeURIComponent(pluginId)),
                    app.api('/plugins/all-skills'),
                    app.api('/agents'),
                    app.api('/mcp-servers')
                ]);
                const plugin = results[0];
                const allSkills = results[1] || [];
                const allAgents = results[2] || [];
                const allMcpServers = results[3] || [];
                if (!plugin) {
                    root.innerHTML = '<div class="detail-header"><a href="' + app.BASE + '/plugins/" class="btn btn-secondary btn-sm">&larr; Back to Plugins</a></div>' +
                        '<div class="empty-state"><p>Plugin not found.</p></div>';
                    return;
                }
                root.innerHTML = renderForm(plugin, allSkills, allAgents, allMcpServers);
                attachFormHandler(root, pluginId);
                app.formUtils.attachFilterHandlers(root);
            } catch (err) {
                root.innerHTML = '<div class="detail-header"><a href="' + app.BASE + '/plugins/" class="btn btn-secondary btn-sm">&larr; Back to Plugins</a></div>' +
                    '<div class="empty-state"><p>Failed to load plugin.</p></div>';
                app.Toast.show(err.message || 'Failed to load plugin', 'error');
            }
        }
        load();
    };
})(window.AdminApp);

(function(app) {
    const escapeHtml = app.escapeHtml;
    const ROLES = app.constants.ROLES;
    const HOOK_EVENTS = app.constants.HOOK_EVENTS;
    var CHECKLIST_STEPS = {
        2: { title: 'Select Skills', id: 'wizard-skills', dataKey: 'allSkills', selectedKey: 'selectedSkills', label: 'Available Skills' },
        3: { title: 'Select Agents', id: 'wizard-agents', dataKey: 'allAgents', selectedKey: 'selectedAgents', label: 'Available Agents' },
        4: { title: 'MCP Servers', id: 'wizard-mcp', dataKey: 'allMcpServers', selectedKey: 'selectedMcpServers', label: 'Available MCP Servers' },
    };
    function renderChecklistStep(state, stepNum) {
        var cfg = CHECKLIST_STEPS[stepNum];
        return '<h3 style="margin:0 0 var(--space-4);font-size:var(--text-lg);font-weight:600">' + escapeHtml(cfg.title) + '</h3>' +
            app.formUtils.renderChecklist(cfg.id, state[cfg.dataKey], state[cfg.selectedKey], cfg.label, { hasSelectAll: true });
    }
    function renderStep1(state) {
        const f = state.form;
        return '<h3 style="margin:0 0 var(--space-4);font-size:var(--text-lg);font-weight:600">Basic Info</h3>' +
            '<div class="form-group">' +
                '<label class="field-label">Plugin ID <span style="font-weight:400;color:var(--text-tertiary)">(kebab-case)</span></label>' +
                '<input class="field-input" name="plugin_id" placeholder="my-plugin" required value="' + escapeHtml(f.plugin_id) + '">' +
            '</div>' +
            '<div class="form-group">' +
                '<label class="field-label">Name</label>' +
                '<input class="field-input" name="name" placeholder="My Plugin" required value="' + escapeHtml(f.name) + '">' +
            '</div>' +
            '<div class="form-group">' +
                '<label class="field-label">Description</label>' +
                '<textarea class="field-input" name="description" rows="3" placeholder="What this plugin does...">' + escapeHtml(f.description) + '</textarea>' +
            '</div>' +
            '<div style="display:flex;gap:var(--space-3)">' +
                '<div class="form-group" style="flex:1">' +
                    '<label class="field-label">Version</label>' +
                    '<input class="field-input" name="version" value="' + escapeHtml(f.version) + '">' +
                '</div>' +
                '<div class="form-group" style="flex:1">' +
                    '<label class="field-label">Category</label>' +
                    '<input class="field-input" name="category" placeholder="productivity" value="' + escapeHtml(f.category) + '">' +
                '</div>' +
            '</div>';
    }
    function renderStep5(state) {
        const hooksHtml = state.hooks.map((hook, idx) => {
            const eventOptions = HOOK_EVENTS.map((evt) => {
                const selected = hook.event === evt ? ' selected' : '';
                return '<option value="' + escapeHtml(evt) + '"' + selected + '>' + escapeHtml(evt) + '</option>';
            }).join('');
            return '<div class="hook-entry card" style="padding:var(--space-3);margin-bottom:var(--space-3)" data-hook-index="' + idx + '">' +
                '<div style="display:flex;gap:var(--space-3);flex-wrap:wrap;align-items:flex-start">' +
                    '<div class="form-group" style="flex:1;min-width:150px">' +
                        '<label class="field-label">Event</label>' +
                        '<select class="field-input" name="hook_event" data-hook-idx="' + idx + '">' + eventOptions + '</select>' +
                    '</div>' +
                    '<div class="form-group" style="flex:1;min-width:150px">' +
                        '<label class="field-label">Matcher</label>' +
                        '<input class="field-input" name="hook_matcher" value="' + escapeHtml(hook.matcher || '') + '" placeholder="*" data-hook-idx="' + idx + '">' +
                    '</div>' +
                    '<div class="form-group" style="flex:2;min-width:200px">' +
                        '<label class="field-label">Command</label>' +
                        '<input class="field-input" name="hook_command" value="' + escapeHtml(hook.command || '') + '" placeholder="do-something" data-hook-idx="' + idx + '">' +
                    '</div>' +
                    '<div class="form-group" style="display:flex;flex-direction:column;align-items:center">' +
                        '<label class="field-label">Async</label>' +
                        '<input type="checkbox" name="hook_async" data-hook-idx="' + idx + '"' + (hook.async ? ' checked' : '') + ' style="margin-top:var(--space-2)">' +
                    '</div>' +
                    '<div style="display:flex;align-items:flex-end;padding-bottom:var(--space-1)">' +
                        '<button type="button" class="btn btn-danger btn-sm" data-remove-hook="' + idx + '">Remove</button>' +
                    '</div>' +
                '</div>' +
            '</div>';
        }).join('');
        return '<h3 style="margin:0 0 var(--space-4);font-size:var(--text-lg);font-weight:600">Hooks</h3>' +
            '<p style="font-size:var(--text-sm);color:var(--text-secondary);margin:0 0 var(--space-4)">Configure event hooks that trigger commands during plugin lifecycle.</p>' +
            '<div id="hooks-list">' + hooksHtml + '</div>' +
            '<button type="button" class="btn btn-secondary btn-sm" id="btn-add-hook">+ Add Hook</button>';
    }
    function renderStep6(state) {
        const f = state.form;
        const rolesHtml = ROLES.map((r) => {
            const checked = state.form.roles[r] ? ' checked' : '';
            return '<label style="display:inline-flex;align-items:center;gap:var(--space-1);margin-right:var(--space-3);font-size:var(--text-sm);cursor:pointer">' +
                '<input type="checkbox" name="wizard-roles" value="' + escapeHtml(r) + '"' + checked + '>' +
                escapeHtml(r) +
            '</label>';
        }).join('');
        return '<h3 style="margin:0 0 var(--space-4);font-size:var(--text-lg);font-weight:600">Roles & Access</h3>' +
            '<div class="form-group">' +
                '<label class="field-label">Roles</label>' +
                '<div style="display:flex;flex-wrap:wrap;gap:var(--space-1);padding:var(--space-2) 0">' + rolesHtml + '</div>' +
            '</div>' +
            '<div class="form-group">' +
                '<label class="field-label">Author Name</label>' +
                '<input class="field-input" name="author_name" placeholder="Your name" value="' + escapeHtml(f.author_name) + '">' +
            '</div>' +
            '<div class="form-group">' +
                '<label class="field-label">Keywords <span style="font-weight:400;color:var(--text-tertiary)">(comma-separated)</span></label>' +
                '<input class="field-input" name="keywords" placeholder="automation, workflow" value="' + escapeHtml(f.keywords) + '">' +
            '</div>';
    }
    function renderStep7(state) {
        const f = state.form;
        const selectedSkillsList = Object.keys(state.selectedSkills).filter((k) => state.selectedSkills[k]);
        const selectedAgentsList = Object.keys(state.selectedAgents).filter((k) => state.selectedAgents[k]);
        const selectedMcpList = Object.keys(state.selectedMcpServers).filter((k) => state.selectedMcpServers[k]);
        const selectedRoles = Object.keys(state.form.roles).filter((k) => state.form.roles[k]);
        function renderList(items, emptyMsg) {
            if (!items.length) return '<span style="color:var(--text-tertiary)">' + escapeHtml(emptyMsg) + '</span>';
            return items.map((i) => '<span class="badge badge-blue" style="margin:var(--space-1)">' + escapeHtml(i) + '</span>').join('');
        }
        return '<h3 style="margin:0 0 var(--space-4);font-size:var(--text-lg);font-weight:600">Review & Create</h3>' +
            '<div class="card" style="padding:var(--space-4)">' +
                '<div style="display:grid;grid-template-columns:140px 1fr;gap:var(--space-3);font-size:var(--text-sm)">' +
                    '<strong>Plugin ID:</strong><span>' + escapeHtml(f.plugin_id || '-') + '</span>' +
                    '<strong>Name:</strong><span>' + escapeHtml(f.name || '-') + '</span>' +
                    '<strong>Description:</strong><span>' + escapeHtml(f.description || '-') + '</span>' +
                    '<strong>Version:</strong><span>' + escapeHtml(f.version || '0.1.0') + '</span>' +
                    '<strong>Category:</strong><span>' + escapeHtml(f.category || '-') + '</span>' +
                    '<strong>Author:</strong><span>' + escapeHtml(f.author_name || '-') + '</span>' +
                    '<strong>Keywords:</strong><span>' + escapeHtml(f.keywords || '-') + '</span>' +
                    '<strong>Roles:</strong><div>' + renderList(selectedRoles, 'None selected') + '</div>' +
                    '<strong>Skills (' + selectedSkillsList.length + '):</strong><div style="display:flex;flex-wrap:wrap">' + renderList(selectedSkillsList, 'None selected') + '</div>' +
                    '<strong>Agents (' + selectedAgentsList.length + '):</strong><div style="display:flex;flex-wrap:wrap">' + renderList(selectedAgentsList, 'None selected') + '</div>' +
                    '<strong>MCP Servers (' + selectedMcpList.length + '):</strong><div style="display:flex;flex-wrap:wrap">' + renderList(selectedMcpList, 'None selected') + '</div>' +
                    '<strong>Hooks (' + state.hooks.length + '):</strong><span>' + (state.hooks.length > 0 ? state.hooks.map((h) => escapeHtml(h.event + ': ' + (h.command || '?'))).join(', ') : 'None') + '</span>' +
                '</div>' +
            '</div>';
    }
    function renderCurrentStep(state) {
        switch (state.step) {
            case 1: return renderStep1(state);
            case 2:
            case 3:
            case 4: return renderChecklistStep(state, state.step);
            case 5: return renderStep5(state);
            case 6: return renderStep6(state);
            case 7: return renderStep7(state);
            default: return '';
        }
    }
    app.pluginWizardSteps = {
        renderCurrentStep,
        HOOK_EVENTS: app.constants.HOOK_EVENTS
    };
})(window.AdminApp);

(function(app) {
    const escapeHtml = app.escapeHtml;
    const TOTAL_STEPS = 7;
    const state = {
        step: 1,
        allSkills: [],
        allAgents: [],
        allMcpServers: [],
        selectedSkills: {},
        selectedAgents: {},
        selectedMcpServers: {},
        hooks: [],
        form: {
            plugin_id: '',
            name: '',
            description: '',
            version: '0.1.0',
            category: '',
            author_name: '',
            keywords: '',
            roles: {}
        }
    };
    let root = null;
    function renderStepIndicator() {
        const labels = ['Basic Info', 'Skills', 'Agents', 'MCP Servers', 'Hooks', 'Roles & Access', 'Review'];
        let html = '<div class="wizard-steps" style="display:flex;gap:var(--space-1);margin-bottom:var(--space-6);flex-wrap:wrap">';
        for (let i = 1; i <= TOTAL_STEPS; i++) {
            const isActive = i === state.step;
            const isDone = i < state.step;
            const cls = isActive ? 'wizard-step active' : (isDone ? 'wizard-step done' : 'wizard-step');
            const bgColor = isActive ? 'var(--accent)' : (isDone ? 'var(--success)' : 'var(--bg-tertiary)');
            const textColor = (isActive || isDone) ? '#fff' : 'var(--text-tertiary)';
            html += '<div class="' + cls + '" style="display:flex;align-items:center;gap:var(--space-2);padding:var(--space-2) var(--space-3);border-radius:var(--radius-md);background:' + bgColor + ';color:' + textColor + ';font-size:var(--text-sm);font-weight:' + (isActive ? '600' : '400') + '">' +
                '<span style="width:20px;height:20px;border-radius:50%;background:rgba(255,255,255,0.2);display:inline-flex;align-items:center;justify-content:center;font-size:var(--text-xs)">' + i + '</span>' +
                '<span>' + escapeHtml(labels[i - 1]) + '</span>' +
            '</div>';
        }
        html += '</div>';
        return html;
    }
    function renderNavButtons() {
        let html = '<div style="display:flex;gap:var(--space-3);margin-top:var(--space-6)">';
        if (state.step > 1) {
            html += '<button type="button" class="btn btn-secondary" id="wizard-prev">Previous</button>';
        }
        if (state.step < TOTAL_STEPS) {
            html += '<button type="button" class="btn btn-primary" id="wizard-next">Next</button>';
        }
        if (state.step === TOTAL_STEPS) {
            html += '<button type="button" class="btn btn-primary" id="wizard-create">Create Plugin</button>';
        }
        html += '</div>';
        return html;
    }
    function saveCurrentStepState() {
        if (!root) return;
        if (state.step === 1) {
            const fields = ['plugin_id', 'name', 'description', 'version', 'category'];
            fields.forEach((name) => {
                const input = root.querySelector('[name="' + name + '"]');
                if (input) {
                    state.form[name] = input.value;
                }
            });
        }
        if (state.step === 2) {
            state.selectedSkills = {};
            const checked = root.querySelectorAll('input[name="wizard-skills"]:checked');
            Array.from(checked).forEach((cb) => { state.selectedSkills[cb.value] = true; });
        }
        if (state.step === 3) {
            state.selectedAgents = {};
            const checked = root.querySelectorAll('input[name="wizard-agents"]:checked');
            Array.from(checked).forEach((cb) => { state.selectedAgents[cb.value] = true; });
        }
        if (state.step === 4) {
            state.selectedMcpServers = {};
            const checked = root.querySelectorAll('input[name="wizard-mcp"]:checked');
            Array.from(checked).forEach((cb) => { state.selectedMcpServers[cb.value] = true; });
        }
        if (state.step === 5) {
            state.hooks.forEach((hook, idx) => {
                const eventEl = root.querySelector('select[name="hook_event"][data-hook-idx="' + idx + '"]');
                const matcherEl = root.querySelector('input[name="hook_matcher"][data-hook-idx="' + idx + '"]');
                const commandEl = root.querySelector('input[name="hook_command"][data-hook-idx="' + idx + '"]');
                const asyncEl = root.querySelector('input[name="hook_async"][data-hook-idx="' + idx + '"]');
                if (eventEl) hook.event = eventEl.value;
                if (matcherEl) hook.matcher = matcherEl.value;
                if (commandEl) hook.command = commandEl.value;
                if (asyncEl) hook.async = asyncEl.checked;
            });
        }
        if (state.step === 6) {
            state.form.roles = {};
            const checkedRoles = root.querySelectorAll('input[name="wizard-roles"]:checked');
            Array.from(checkedRoles).forEach((cb) => { state.form.roles[cb.value] = true; });
            const authorInput = root.querySelector('[name="author_name"]');
            if (authorInput) state.form.author_name = authorInput.value;
            const keywordsInput = root.querySelector('[name="keywords"]');
            if (keywordsInput) state.form.keywords = keywordsInput.value;
        }
    }
    function renderWizard() {
        const content = '<div class="detail-header">' +
            '<a href="' + app.BASE + '/plugins/" class="btn btn-secondary btn-sm">&larr; Back to Plugins</a>' +
        '</div>' +
        '<h2 style="margin:var(--space-4) 0 var(--space-5);font-size:var(--text-xl);font-weight:600">Create Plugin</h2>' +
        renderStepIndicator() +
        '<div class="card" style="max-width:800px;padding:var(--space-5)">' +
            app.pluginWizardSteps.renderCurrentStep(state) +
            renderNavButtons() +
        '</div>';
        root.innerHTML = content;
    }
    function validateStep1() {
        const pluginId = state.form.plugin_id;
        const name = state.form.name;
        if (!pluginId || !pluginId.trim()) {
            app.Toast.show('Plugin ID is required', 'error');
            return false;
        }
        if (!/^[a-z0-9]+(-[a-z0-9]+)*$/.test(pluginId.trim())) {
            app.Toast.show('Plugin ID must be kebab-case (e.g. my-plugin)', 'error');
            return false;
        }
        if (!name || !name.trim()) {
            app.Toast.show('Name is required', 'error');
            return false;
        }
        return true;
    }
    function buildPluginBody() {
        const f = state.form;
        const keywords = (f.keywords || '').split(',').map((t) => t.trim()).filter(Boolean);
        const roles = Object.keys(f.roles).filter((k) => f.roles[k]);
        const skills = Object.keys(state.selectedSkills).filter((k) => state.selectedSkills[k]);
        const agents = Object.keys(state.selectedAgents).filter((k) => state.selectedAgents[k]);
        const mcpServers = Object.keys(state.selectedMcpServers).filter((k) => state.selectedMcpServers[k]);
        const hooks = state.hooks.map((h) => ({
                event: h.event || 'PostToolUse',
                matcher: h.matcher || '*',
                command: h.command || '',
                async: !!h.async
            })).filter((h) => h.command);
        return {
            id: f.plugin_id.trim(),
            name: f.name.trim(),
            description: f.description || '',
            version: f.version || '0.1.0',
            category: f.category || '',
            enabled: true,
            keywords,
            author: { name: f.author_name || '' },
            roles,
            skills,
            agents,
            mcp_servers: mcpServers,
            hooks
        };
    }
    async function createPlugin() {
        const body = buildPluginBody();
        const createBtn = root.querySelector('#wizard-create');
        if (createBtn) {
            createBtn.disabled = true;
            createBtn.textContent = 'Creating...';
        }
        try {
            await app.api('/plugins', {
                method: 'POST',
                body: JSON.stringify(body)
            });
            app.Toast.show('Plugin created!', 'success');
            window.location.href = app.BASE + '/plugins/';
        } catch (err) {
            app.Toast.show(err.message || 'Failed to create plugin', 'error');
            if (createBtn) {
                createBtn.disabled = false;
                createBtn.textContent = 'Create Plugin';
            }
        }
    }
    function attachEventHandlers() {
        root.addEventListener('click', (e) => {
            if (e.target.closest('#wizard-next')) {
                saveCurrentStepState();
                if (state.step === 1 && !validateStep1()) return;
                if (state.step < TOTAL_STEPS) {
                    state.step++;
                    renderWizard();
                }
                return;
            }
            if (e.target.closest('#wizard-prev')) {
                saveCurrentStepState();
                if (state.step > 1) {
                    state.step--;
                    renderWizard();
                }
                return;
            }
            if (e.target.closest('#wizard-create')) {
                saveCurrentStepState();
                createPlugin();
                return;
            }
            if (e.target.closest('#btn-add-hook')) {
                saveCurrentStepState();
                state.hooks.push({ event: 'PostToolUse', matcher: '*', command: '', async: false });
                renderWizard();
                return;
            }
            const removeHookBtn = e.target.closest('[data-remove-hook]');
            if (removeHookBtn) {
                saveCurrentStepState();
                const idx = parseInt(removeHookBtn.getAttribute('data-remove-hook'), 10);
                state.hooks.splice(idx, 1);
                renderWizard();
                return;
            }
            const selectAllBtn = e.target.closest('[data-select-all]');
            if (selectAllBtn) {
                const listId = selectAllBtn.getAttribute('data-select-all');
                const container = root.querySelector('[data-checklist="' + listId + '"]');
                if (container) {
                    const checkboxes = container.querySelectorAll('input[type="checkbox"]');
                    checkboxes.forEach((cb) => { cb.checked = true; });
                }
                return;
            }
            const deselectAllBtn = e.target.closest('[data-deselect-all]');
            if (deselectAllBtn) {
                const listId = deselectAllBtn.getAttribute('data-deselect-all');
                const container = root.querySelector('[data-checklist="' + listId + '"]');
                if (container) {
                    const checkboxes = container.querySelectorAll('input[type="checkbox"]');
                    checkboxes.forEach((cb) => { cb.checked = false; });
                }
                return;
            }
        });
        app.formUtils.attachFilterHandlers(root);
    }
    app.renderPluginWizard = async (selector) => {
        root = document.querySelector(selector);
        if (!root) return;
        app.shared.showLoading(root, 'Loading...');
        try {
            const results = await Promise.all([
                app.api('/plugins/all-skills'),
                app.api('/agents'),
                app.api('/mcp-servers')
            ]);
            state.allSkills = results[0] || [];
            state.allAgents = results[1] || [];
            state.allMcpServers = results[2] || [];
            renderWizard();
            attachEventHandlers();
        } catch (err) {
            root.innerHTML = '<div class="detail-header"><a href="' + app.BASE + '/plugins/" class="btn btn-secondary btn-sm">&larr; Back to Plugins</a></div>' +
                '<div class="empty-state"><p>Failed to load data.</p></div>';
            app.Toast.show(err.message || 'Failed to load data', 'error');
        }
    };
})(window.AdminApp);

(function(app) {
    var escapeHtml = app.escapeHtml;
    var truncate = app.shared.truncate;
    app.renderSkills = function(selector) {
        app.shared.createListPage(selector, {
            entityName:        'skill',
            pageKey:           'skills',
            searchInputId:     'skill-search',
            searchPlaceholder: 'Search skills...',
            newHref:           app.BASE + '/skills/edit/',
            newLabel:          '+ New Skill',
            apiPath:           '/skills',
            apiResponseKey:    'skills',
            columns:           ['Name', 'Skill ID', 'Description', 'Status', ''],
            idAttr:            'skill-id',
            filterFn: function(s, q) {
                return (s.name || '').toLowerCase().indexOf(q) >= 0 ||
                       (s.skill_id || '').toLowerCase().indexOf(q) >= 0 ||
                       (s.description || '').toLowerCase().indexOf(q) >= 0;
            },
            renderRow: function(s) {
                var checked = s.enabled ? ' checked' : '';
                return '<tr>' +
                    '<td style="font-weight:500">' + escapeHtml(s.name) + '</td>' +
                    '<td><code style="background:var(--bg-surface-raised);padding:2px 8px;border-radius:var(--radius-xs);font-size:var(--text-xs)">' + escapeHtml(s.skill_id) + '</code></td>' +
                    '<td title="' + escapeHtml(s.description || '') + '" style="color:var(--text-secondary)">' + escapeHtml(truncate(s.description)) + '</td>' +
                    '<td><label class="toggle-switch"><input type="checkbox"' + checked + ' data-skill-id="' + escapeHtml(s.skill_id) + '" data-action="toggle"><span class="toggle-slider"></span></label></td>' +
                    '<td class="col-actions"><div class="actions-menu" data-actions-for="' + escapeHtml(s.skill_id) + '">' +
                        '<button class="actions-trigger" data-action="menu" title="Actions">&#8942;</button>' +
                        '<div class="actions-dropdown">' +
                            '<a href="' + app.BASE + '/skills/edit/?id=' + encodeURIComponent(s.skill_id) + '" class="actions-item">Edit</a>' +
                            '<button class="actions-item actions-item-danger" data-action="delete" data-skill-id="' + escapeHtml(s.skill_id) + '">Delete</button>' +
                        '</div></div></td>' +
                '</tr>';
            },
            hasToggle:         true,
            toggleApiPath:     function(id) { return '/skills/' + encodeURIComponent(id); },
            toggleBody:        function(enabled) { return { enabled: enabled }; },
            toggleSuccessMsg:  function(enabled) { return 'Skill ' + (enabled ? 'enabled' : 'disabled'); },
            deleteDialogTitle: 'Delete Skill?',
            deleteApiPath:     function(id) { return '/skills/' + encodeURIComponent(id); },
            deleteSuccessMsg:  'Skill deleted'
        });
    };
})(window.AdminApp);

(function(app) {
    var escapeHtml = app.escapeHtml;
    function renderForm(skill) {
        var isEdit = !!skill;
        var title = isEdit ? 'Edit Skill' : 'Create Skill';
        return '<div class="detail-header">' +
            '<a href="' + app.BASE + '/skills/" class="btn btn-secondary btn-sm">&larr; Back to Skills</a>' +
        '</div>' +
        '<h2 style="margin:var(--space-4) 0 var(--space-5);font-size:var(--text-xl);font-weight:600">' + escapeHtml(title) + '</h2>' +
        '<div class="card" style="max-width:800px">' +
            '<form id="skill-form">' +
                '<div class="form-group">' +
                    '<label class="field-label">Skill ID <span style="font-weight:400;color:var(--text-tertiary)">(kebab-case)</span></label>' +
                    '<input class="field-input" name="skill_id" placeholder="my-custom-skill" required' +
                        (isEdit ? ' readonly value="' + escapeHtml(skill.skill_id) + '"' : '') + '>' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label">Name</label>' +
                    '<input class="field-input" name="name" placeholder="My Custom Skill" required' +
                        (isEdit ? ' value="' + escapeHtml(skill.name) + '"' : '') + '>' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label">Description</label>' +
                    '<input class="field-input" name="description" placeholder="What this skill does..."' +
                        (isEdit ? ' value="' + escapeHtml(skill.description || '') + '"' : '') + '>' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label">Content (Markdown)</label>' +
                    '<textarea class="field-input" name="content" rows="15" placeholder="# Skill Instructions...">' +
                        (isEdit ? escapeHtml(skill.content || '') : '') +
                    '</textarea>' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label">Tags (comma-separated)</label>' +
                    '<input class="field-input" name="tags" placeholder="writing, content, marketing"' +
                        (isEdit && skill.tags ? ' value="' + escapeHtml(skill.tags.join(', ')) + '"' : '') + '>' +
                '</div>' +
                '<div style="display:flex;gap:var(--space-3);margin-top:var(--space-6)">' +
                    '<button type="submit" class="btn btn-primary">Save</button>' +
                    '<a href="' + app.BASE + '/skills/" class="btn btn-secondary">Cancel</a>' +
                '</div>' +
            '</form>' +
        '</div>';
    }
    function buildBody(form, formData) {
        var tagsRaw = formData.get('tags') || '';
        var tags = tagsRaw.split(',').map(function(t) { return t.trim(); }).filter(Boolean);
        return {
            name: formData.get('name'),
            description: formData.get('description') || '',
            content: formData.get('content') || '',
            tags: tags
        };
    }
    app.renderSkillEditor = function(selector) {
        app.shared.createEditPage(selector, {
            entityName: 'skill',
            listPath:   '/skills/',
            apiPath:    '/skills/',
            idParam:    'id',
            idField:    'skill_id',
            formId:     'skill-form',
            renderForm: renderForm,
            buildBody:  buildBody,
            successMsg: 'Skill saved!'
        });
    };
})(window.AdminApp);

(function(app) {
    var escapeHtml = app.escapeHtml;
    var truncate = app.shared.truncate;
    app.renderAgents = function(selector) {
        app.shared.createListPage(selector, {
            entityName:        'agent',
            pageKey:           'agents',
            searchInputId:     'agent-search',
            searchPlaceholder: 'Search agents...',
            newHref:           app.BASE + '/agents/edit/',
            newLabel:          '+ New Agent',
            apiPath:           '/agents',
            apiResponseKey:    'agents',
            columns:           ['Name', 'Agent ID', 'Description', 'Primary', 'Status', ''],
            idAttr:            'agent-id',
            filterFn: function(a, q) {
                return (a.name || '').toLowerCase().indexOf(q) >= 0 ||
                       (a.id || '').toLowerCase().indexOf(q) >= 0 ||
                       (a.description || '').toLowerCase().indexOf(q) >= 0;
            },
            renderRow: function(a) {
                var checked = a.enabled ? ' checked' : '';
                var primaryBadge = a.is_primary
                    ? '<span style="background:var(--bg-surface-raised);padding:2px 8px;border-radius:var(--radius-xs);font-size:var(--text-xs);color:var(--text-secondary)">Primary</span>'
                    : '';
                return '<tr>' +
                    '<td style="font-weight:500">' + escapeHtml(a.name) + '</td>' +
                    '<td><code style="background:var(--bg-surface-raised);padding:2px 8px;border-radius:var(--radius-xs);font-size:var(--text-xs)">' + escapeHtml(a.id) + '</code></td>' +
                    '<td title="' + escapeHtml(a.description || '') + '" style="color:var(--text-secondary)">' + escapeHtml(truncate(a.description)) + '</td>' +
                    '<td>' + primaryBadge + '</td>' +
                    '<td><label class="toggle-switch"><input type="checkbox"' + checked + ' data-agent-id="' + escapeHtml(a.id) + '" data-action="toggle"><span class="toggle-slider"></span></label></td>' +
                    '<td class="col-actions"><div class="actions-menu" data-actions-for="' + escapeHtml(a.id) + '">' +
                        '<button class="actions-trigger" data-action="menu" title="Actions">&#8942;</button>' +
                        '<div class="actions-dropdown">' +
                            '<a href="' + app.BASE + '/agents/edit/?id=' + encodeURIComponent(a.id) + '" class="actions-item">Edit</a>' +
                            '<button class="actions-item actions-item-danger" data-action="delete" data-agent-id="' + escapeHtml(a.id) + '">Delete</button>' +
                        '</div></div></td>' +
                '</tr>';
            },
            hasToggle:         true,
            toggleApiPath:     function(id) { return '/agents/' + encodeURIComponent(id); },
            toggleBody:        function(enabled) { return { enabled: enabled }; },
            toggleSuccessMsg:  function(enabled) { return 'Agent ' + (enabled ? 'enabled' : 'disabled'); },
            deleteDialogTitle: 'Delete Agent?',
            deleteApiPath:     function(id) { return '/agents/' + encodeURIComponent(id); },
            deleteSuccessMsg:  'Agent deleted'
        });
    };
})(window.AdminApp);

(function(app) {
    var escapeHtml = app.escapeHtml;
    function renderForm(agent) {
        var isEdit = !!agent;
        var title = isEdit ? 'Edit Agent' : 'Create Agent';
        return '<div class="detail-header">' +
            '<a href="' + app.BASE + '/agents/" class="btn btn-secondary btn-sm">&larr; Back to Agents</a>' +
        '</div>' +
        '<h2 style="margin:var(--space-4) 0 var(--space-5);font-size:var(--text-xl);font-weight:600">' + escapeHtml(title) + '</h2>' +
        '<div class="card" style="max-width:800px">' +
            '<form id="agent-form">' +
                '<div class="form-group">' +
                    '<label class="field-label">Agent ID <span style="font-weight:400;color:var(--text-tertiary)">(kebab-case)</span></label>' +
                    '<input class="field-input" name="id" placeholder="my-custom-agent" required' +
                        (isEdit ? ' readonly value="' + escapeHtml(agent.id) + '"' : '') + '>' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label">Name</label>' +
                    '<input class="field-input" name="name" placeholder="My Custom Agent" required' +
                        (isEdit ? ' value="' + escapeHtml(agent.name) + '"' : '') + '>' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label">Description</label>' +
                    '<input class="field-input" name="description" placeholder="What this agent does..."' +
                        (isEdit ? ' value="' + escapeHtml(agent.description || '') + '"' : '') + '>' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label">System Prompt</label>' +
                    '<textarea class="field-input" name="system_prompt" rows="15" placeholder="You are a helpful assistant that...">' +
                        (isEdit ? escapeHtml(agent.system_prompt || '') : '') +
                    '</textarea>' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="toggle-switch" style="display:inline-flex;align-items:center;gap:var(--space-3)">' +
                        '<input type="checkbox" name="enabled"' + (isEdit ? (agent.enabled ? ' checked' : '') : ' checked') + '>' +
                        '<span class="toggle-slider"></span>' +
                        '<span class="field-label" style="margin:0">Enabled</span>' +
                    '</label>' +
                '</div>' +
                '<div style="display:flex;gap:var(--space-3);margin-top:var(--space-6)">' +
                    '<button type="submit" class="btn btn-primary">Save</button>' +
                    '<a href="' + app.BASE + '/agents/" class="btn btn-secondary">Cancel</a>' +
                '</div>' +
            '</form>' +
        '</div>';
    }
    function buildBody(form, formData) {
        return {
            name: formData.get('name'),
            description: formData.get('description') || '',
            system_prompt: formData.get('system_prompt') || '',
            enabled: form.querySelector('[name="enabled"]').checked
        };
    }
    app.renderAgentEditor = function(selector) {
        app.shared.createEditPage(selector, {
            entityName: 'agent',
            listPath:   '/agents/',
            apiPath:    '/agents/',
            idParam:    'id',
            idField:    'id',
            formId:     'agent-form',
            renderForm: renderForm,
            buildBody:  buildBody,
            successMsg: 'Agent saved!'
        });
    };
})(window.AdminApp);

(function(app) {
    var escapeHtml = app.escapeHtml;
    var truncate = app.shared.truncate;
    app.renderMcpServers = function(selector) {
        app.shared.createListPage(selector, {
            entityName:        'MCP server',
            pageKey:           'mcp-servers',
            searchInputId:     'mcp-search',
            searchPlaceholder: 'Search MCP servers...',
            newHref:           app.BASE + '/mcp-servers/edit/',
            newLabel:          '+ New MCP Server',
            apiPath:           '/mcp-servers',
            apiResponseKey:    'mcp_servers',
            columns:           ['Name', 'Server ID', 'Binary', 'Port', 'Description', 'Status', ''],
            idAttr:            'server-id',
            filterFn: function(s, q) {
                return (s.id || '').toLowerCase().indexOf(q) >= 0 ||
                       (s.description || '').toLowerCase().indexOf(q) >= 0 ||
                       (s.binary || '').toLowerCase().indexOf(q) >= 0;
            },
            renderRow: function(s) {
                var checked = s.enabled ? ' checked' : '';
                return '<tr>' +
                    '<td style="font-weight:500">' + escapeHtml(s.id) + '</td>' +
                    '<td><code style="background:var(--bg-surface-raised);padding:2px 8px;border-radius:var(--radius-xs);font-size:var(--text-xs)">' + escapeHtml(s.id) + '</code></td>' +
                    '<td><code style="background:var(--bg-surface-raised);padding:2px 8px;border-radius:var(--radius-xs);font-size:var(--text-xs)">' + escapeHtml(s.binary || '') + '</code></td>' +
                    '<td>' + escapeHtml(String(s.port || '')) + '</td>' +
                    '<td title="' + escapeHtml(s.description || '') + '" style="color:var(--text-secondary)">' + escapeHtml(truncate(s.description)) + '</td>' +
                    '<td><label class="toggle-switch"><input type="checkbox"' + checked + ' data-server-id="' + escapeHtml(s.id) + '" data-action="toggle"><span class="toggle-slider"></span></label></td>' +
                    '<td class="col-actions"><div class="actions-menu" data-actions-for="' + escapeHtml(s.id) + '">' +
                        '<button class="actions-trigger" data-action="menu" title="Actions">&#8942;</button>' +
                        '<div class="actions-dropdown">' +
                            '<a href="' + app.BASE + '/mcp-servers/edit/?id=' + encodeURIComponent(s.id) + '" class="actions-item">Edit</a>' +
                            '<button class="actions-item actions-item-danger" data-action="delete" data-server-id="' + escapeHtml(s.id) + '">Delete</button>' +
                        '</div></div></td>' +
                '</tr>';
            },
            hasToggle:         true,
            toggleApiPath:     function(id) { return '/mcp-servers/' + encodeURIComponent(id); },
            toggleBody:        function(enabled) { return { enabled: enabled }; },
            toggleSuccessMsg:  function(enabled) { return 'MCP server ' + (enabled ? 'enabled' : 'disabled'); },
            deleteDialogTitle: 'Delete MCP Server?',
            deleteApiPath:     function(id) { return '/mcp-servers/' + encodeURIComponent(id); },
            deleteSuccessMsg:  'MCP server deleted'
        });
    };
})(window.AdminApp);

(function(app) {
    var escapeHtml = app.escapeHtml;
    function renderForm(server) {
        var isEdit = !!server;
        var title = isEdit ? 'Edit MCP Server' : 'Create MCP Server';
        return '<div class="detail-header">' +
            '<a href="' + app.BASE + '/mcp-servers/" class="btn btn-secondary btn-sm">&larr; Back to MCP Servers</a>' +
        '</div>' +
        '<h2 style="margin:var(--space-4) 0 var(--space-5);font-size:var(--text-xl);font-weight:600">' + escapeHtml(title) + '</h2>' +
        '<div class="card" style="max-width:800px">' +
            '<form id="mcp-form">' +
                '<div class="form-group">' +
                    '<label class="field-label">Server ID <span style="font-weight:400;color:var(--text-tertiary)">(kebab-case)</span></label>' +
                    '<input class="field-input" name="id" placeholder="my-mcp-server" required' +
                        (isEdit ? ' readonly value="' + escapeHtml(server.id) + '"' : '') + '>' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label">Binary</label>' +
                    '<input class="field-input" name="binary" placeholder="systemprompt-mcp-agent"' +
                        (isEdit ? ' value="' + escapeHtml(server.binary || '') + '"' : '') + '>' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label">Package</label>' +
                    '<input class="field-input" name="package_name" placeholder="systemprompt"' +
                        (isEdit ? ' value="' + escapeHtml(server['package'] || server.package_name || '') + '"' : '') + '>' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label">Port</label>' +
                    '<input class="field-input" name="port" type="number" placeholder="3100"' +
                        (isEdit ? ' value="' + escapeHtml(String(server.port || '')) + '"' : '') + '>' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label">Endpoint URL</label>' +
                    '<input class="field-input" name="endpoint" placeholder="https://mcp.example.com/sse"' +
                        (isEdit ? ' value="' + escapeHtml(server.endpoint || '') + '"' : '') + '>' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label">Description</label>' +
                    '<input class="field-input" name="description" placeholder="What this MCP server does..."' +
                        (isEdit ? ' value="' + escapeHtml(server.description || '') + '"' : '') + '>' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="toggle-switch" style="display:inline-flex;align-items:center;gap:var(--space-2)">' +
                        '<input type="checkbox" name="enabled"' + (isEdit && server.enabled ? ' checked' : (!isEdit ? ' checked' : '')) + '>' +
                        '<span class="toggle-slider"></span>' +
                        '<span class="field-label" style="margin:0">Enabled</span>' +
                    '</label>' +
                '</div>' +
                '<hr style="border:none;border-top:1px solid var(--border-subtle);margin:var(--space-5) 0">' +
                '<h3 style="font-size:var(--text-base);font-weight:600;margin:0 0 var(--space-4)">OAuth Settings</h3>' +
                '<div class="form-group">' +
                    '<label class="toggle-switch" style="display:inline-flex;align-items:center;gap:var(--space-2)">' +
                        '<input type="checkbox" name="oauth_required"' + (isEdit && server.oauth_required ? ' checked' : '') + '>' +
                        '<span class="toggle-slider"></span>' +
                        '<span class="field-label" style="margin:0">OAuth Required</span>' +
                    '</label>' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label">OAuth Scopes <span style="font-weight:400;color:var(--text-tertiary)">(comma-separated)</span></label>' +
                    '<input class="field-input" name="oauth_scopes" placeholder="read, write, admin"' +
                        (isEdit && server.oauth_scopes ? ' value="' + escapeHtml(server.oauth_scopes) + '"' : '') + '>' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label">OAuth Audience</label>' +
                    '<input class="field-input" name="oauth_audience" placeholder="https://api.example.com"' +
                        (isEdit ? ' value="' + escapeHtml(server.oauth_audience || '') + '"' : '') + '>' +
                '</div>' +
                '<div style="display:flex;gap:var(--space-3);margin-top:var(--space-6)">' +
                    '<button type="submit" class="btn btn-primary">Save</button>' +
                    '<a href="' + app.BASE + '/mcp-servers/" class="btn btn-secondary">Cancel</a>' +
                '</div>' +
            '</form>' +
        '</div>';
    }
    function buildBody(form, formData) {
        var portVal = formData.get('port');
        return {
            binary: formData.get('binary') || '',
            package_name: formData.get('package_name') || '',
            port: portVal ? parseInt(portVal, 10) : null,
            endpoint: formData.get('endpoint') || '',
            description: formData.get('description') || '',
            enabled: form.querySelector('[name="enabled"]').checked,
            oauth_required: form.querySelector('[name="oauth_required"]').checked,
            oauth_scopes: formData.get('oauth_scopes') || '',
            oauth_audience: formData.get('oauth_audience') || ''
        };
    }
    app.renderMcpEditor = function(selector) {
        app.shared.createEditPage(selector, {
            entityName: 'MCP server',
            listPath:   '/mcp-servers/',
            apiPath:    '/mcp-servers/',
            idParam:    'id',
            idField:    'id',
            formId:     'mcp-form',
            renderForm: renderForm,
            buildBody:  buildBody,
            successMsg: 'MCP server saved!'
        });
    };
})(window.AdminApp);

(function(app) {
    var escapeHtml = app.escapeHtml;
    var truncate = app.shared.truncate;
    app.renderHooks = function(selector) {
        app.shared.createListPage(selector, {
            entityName:        'hook',
            pageKey:           'hooks',
            searchInputId:     'hook-search',
            searchPlaceholder: 'Search hooks...',
            newHref:           app.BASE + '/hooks/edit/',
            newLabel:          '+ New Hook',
            apiPath:           '/hooks',
            apiResponseKey:    'hooks',
            columns:           ['Plugin', 'Event', 'Matcher', 'Command', 'Async', ''],
            idAttr:            'hook-id',
            filterFn: function(h, q) {
                return (h.plugin_id || '').toLowerCase().indexOf(q) >= 0 ||
                       (h.event || '').toLowerCase().indexOf(q) >= 0 ||
                       (h.command || '').toLowerCase().indexOf(q) >= 0;
            },
            renderRow: function(h) {
                var asyncBadge = h.is_async
                    ? '<span class="badge badge-info">Async</span>'
                    : '<span class="badge badge-secondary">Sync</span>';
                return '<tr>' +
                    '<td style="font-weight:500">' + escapeHtml(h.plugin_id) + '</td>' +
                    '<td><code style="background:var(--bg-surface-raised);padding:2px 8px;border-radius:var(--radius-xs);font-size:var(--text-xs)">' + escapeHtml(h.event) + '</code></td>' +
                    '<td>' + escapeHtml(h.matcher || '') + '</td>' +
                    '<td title="' + escapeHtml(h.command || '') + '" style="color:var(--text-secondary)">' + escapeHtml(truncate(h.command)) + '</td>' +
                    '<td>' + asyncBadge + '</td>' +
                    '<td class="col-actions"><div class="actions-menu" data-actions-for="' + escapeHtml(h.id) + '">' +
                        '<button class="actions-trigger" data-action="menu" title="Actions">&#8942;</button>' +
                        '<div class="actions-dropdown">' +
                            '<a href="' + app.BASE + '/hooks/edit/?id=' + encodeURIComponent(h.id) + '" class="actions-item">Edit</a>' +
                            '<button class="actions-item actions-item-danger" data-action="delete" data-hook-id="' + escapeHtml(h.id) + '">Delete</button>' +
                        '</div></div></td>' +
                '</tr>';
            },
            hasToggle:         false,
            deleteDialogTitle: 'Delete Hook?',
            deleteApiPath:     function(id) { return '/hooks/' + encodeURIComponent(id); },
            deleteSuccessMsg:  'Hook deleted'
        });
    };
})(window.AdminApp);

(function(app) {
    var escapeHtml = app.escapeHtml;
    function renderPluginOptions(plugins, selectedId) {
        var options = '<option value="">-- Select Plugin --</option>';
        plugins.forEach(function(p) {
            var id = p.id || p.plugin_id || '';
            var name = p.name || id;
            var selected = (id === selectedId) ? ' selected' : '';
            options += '<option value="' + escapeHtml(id) + '"' + selected + '>' + escapeHtml(name) + '</option>';
        });
        return options;
    }
    function renderEventOptions(selectedEvent) {
        return app.constants.HOOK_EVENTS.map(function(evt) {
            var selected = (evt === selectedEvent) ? ' selected' : '';
            return '<option value="' + escapeHtml(evt) + '"' + selected + '>' + escapeHtml(evt) + '</option>';
        }).join('');
    }
    function renderForm(hook, pluginsData) {
        var isEdit = !!hook;
        var title = isEdit ? 'Edit Hook' : 'Create Hook';
        var pluginsList = pluginsData ? (pluginsData.plugins || pluginsData) : [];
        if (!Array.isArray(pluginsList)) pluginsList = [];
        return '<div class="detail-header">' +
            '<a href="' + app.BASE + '/hooks/" class="btn btn-secondary btn-sm">&larr; Back to Hooks</a>' +
        '</div>' +
        '<h2 style="margin:var(--space-4) 0 var(--space-5);font-size:var(--text-xl);font-weight:600">' + escapeHtml(title) + '</h2>' +
        '<div class="card" style="max-width:800px">' +
            '<form id="hook-form">' +
                '<div class="form-group">' +
                    '<label class="field-label">Plugin</label>' +
                    '<select class="field-input" name="plugin_id" required>' +
                        renderPluginOptions(pluginsList, isEdit ? hook.plugin_id : '') +
                    '</select>' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label">Event Type</label>' +
                    '<select class="field-input" name="event" required>' +
                        renderEventOptions(isEdit ? hook.event : '') +
                    '</select>' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label">Matcher</label>' +
                    '<input class="field-input" name="matcher" placeholder="e.g. Skill, startup"' +
                        (isEdit ? ' value="' + escapeHtml(hook.matcher || '') + '"' : '') + '>' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label">Command</label>' +
                    '<textarea class="field-input" name="command" rows="3" placeholder="e.g. ${CLAUDE_PLUGIN_ROOT}/scripts/track-usage.sh" required>' +
                        (isEdit ? escapeHtml(hook.command || '') : '') +
                    '</textarea>' +
                '</div>' +
                '<div class="form-group">' +
                    '<label class="field-label" style="display:flex;align-items:center;gap:var(--space-2)">' +
                        '<input type="checkbox" name="is_async"' + (isEdit && hook.is_async ? ' checked' : '') + '>' +
                        ' Run Async' +
                    '</label>' +
                '</div>' +
                '<div style="display:flex;gap:var(--space-3);margin-top:var(--space-6)">' +
                    '<button type="submit" class="btn btn-primary">Save</button>' +
                    '<a href="' + app.BASE + '/hooks/" class="btn btn-secondary">Cancel</a>' +
                '</div>' +
            '</form>' +
        '</div>';
    }
    function buildBody(form, formData) {
        return {
            plugin_id: formData.get('plugin_id'),
            event: formData.get('event'),
            matcher: formData.get('matcher') || '',
            command: formData.get('command') || '',
            is_async: form.querySelector('[name="is_async"]').checked
        };
    }
    app.renderHookEditor = function(selector) {
        app.shared.createEditPage(selector, {
            entityName: 'hook',
            listPath:   '/hooks/',
            apiPath:    '/hooks/',
            idParam:    'id',
            idField:    null,
            formId:     'hook-form',
            renderForm: renderForm,
            buildBody:  buildBody,
            successMsg: 'Hook saved!',
            preload:    function() { return app.api('/plugins'); }
        });
    };
})(window.AdminApp);

(function(app) {
    const escapeHtml = app.escapeHtml;
    let exportData = null;
    // --- Helpers ---
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
        const lines = ['#!/bin/bash', '# Install script for systemprompt.io plugins', 'set -e', ''];
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
                lines.push('');
            }
        }
        if (data.marketplace) {
            const mktPath = sanitizePath(data.marketplace.path);
            const delim = safeDelimiter(delimIdx++);
            lines.push('# Marketplace manifest');
            lines.push('mkdir -p ~/.claude/plugins/.claude-plugin');
            lines.push("cat > ~/.claude/plugins/" + mktPath + " << '" + delim + "'");
            lines.push(data.marketplace.content);
            lines.push(delim);
        }
        lines.push('');
        lines.push('echo "All plugins installed successfully."');
        return lines.join('\n');
    }
    // --- Toggle ---
    function toggleBundle(idx) {
        const details = document.getElementById('bundle-details-' + idx);
        const icon = document.getElementById('bundle-icon-' + idx);
        if (!details) return;
        const open = details.style.display !== 'none';
        details.style.display = open ? 'none' : 'block';
        if (icon) icon.innerHTML = open ? '&#9654;' : '&#9660;';
    }
    function copyContent(pluginIdx, fileIdx, btn) {
        if (!exportData) return;
        const plugin = exportData.plugins[pluginIdx];
        if (!plugin) return;
        const file = plugin.files[fileIdx];
        if (!file) return;
        copyToClipboard(file.content, btn);
    }
    function copyScript(btn) {
        if (!exportData) return;
        const script = generateInstallScript(exportData);
        copyToClipboard(script, btn);
    }
    // --- Rendering ---
    function renderSummary(data) {
        const t = data.totals || {};
        const plugins = data.plugins || [];
        const totalFiles = t.files || plugins.reduce((sum, p) => sum + (p.files || []).length, 0);
        const stats = [
            { label: 'Plugins', value: t.plugins || plugins.length },
            { label: 'Total Files', value: totalFiles },
            { label: 'Skills', value: t.skills || 0 },
            { label: 'Agents', value: t.agents || 0 }
        ];
        return '<div class="stats-grid" style="margin-bottom:var(--space-6)">' +
            stats.map((s) => '<div class="stat-card">' +
                    '<div class="stat-value">' + s.value + '</div>' +
                    '<div class="stat-label">' + s.label + '</div>' +
                '</div>').join('') +
        '</div>';
    }
    function renderBundleCard(plugin, idx) {
        const files = plugin.files || [];
        const fileCount = files.length;
        const fileEntries = files.map((file, fileIdx) => '<div style="display:flex;justify-content:space-between;align-items:center;padding:var(--space-2) var(--space-4);border-bottom:1px solid var(--border-subtle)">' +
                '<code style="font-size:var(--text-sm);color:var(--text-secondary)">' + escapeHtml(file.path) + '</code>' +
                '<button class="btn btn-sm btn-secondary" id="copy-btn-' + idx + '-' + fileIdx + '" data-action="copy-content" data-plugin-idx="' + idx + '" data-file-idx="' + fileIdx + '">Copy</button>' +
            '</div>').join('');
        return '<div class="card" style="border-left:3px solid var(--accent);margin-bottom:var(--space-4);padding:0;overflow:hidden">' +
            '<div class="plugin-header" data-action="toggle-bundle" data-idx="' + idx + '" style="cursor:pointer;display:flex;justify-content:space-between;align-items:center;padding:var(--space-4)">' +
                '<div>' +
                    '<h3 style="margin:0;font-size:var(--text-base)">' + escapeHtml(plugin.name || plugin.id) + '</h3>' +
                    '<span class="badge badge-gray" style="margin-top:var(--space-1)">' + fileCount + ' file' + (fileCount !== 1 ? 's' : '') + '</span>' +
                '</div>' +
                '<span class="expand-icon" id="bundle-icon-' + idx + '" style="color:var(--text-tertiary);font-size:var(--text-sm)">&#9654;</span>' +
            '</div>' +
            '<div class="plugin-details" id="bundle-details-' + idx + '" style="display:none">' +
                fileEntries +
            '</div>' +
        '</div>';
    }
    // --- Zip download ---
    async function downloadZip(data) {
        const btn = document.getElementById('btn-download-zip');
        const origHtml = btn.innerHTML;
        btn.innerHTML = 'Generating...';
        btn.disabled = true;
        try {
            const JSZip = await app.shared.loadJSZip();
            const zip = new JSZip();
            const plugins = data.plugins || [];
            plugins.forEach((plugin) => {
                const folder = zip.folder(plugin.id);
                (plugin.files || []).forEach((file) => {
                    folder.file(file.path, file.content);
                });
            });
            if (data.marketplace) {
                zip.file(data.marketplace.path, data.marketplace.content);
            }
            const blob = await zip.generateAsync({ type: 'blob' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = 'systemprompt-plugins.zip';
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
    // --- Main render ---
    app.renderExport = async (selector) => {
        const root = document.querySelector(selector);
        if (!root) return;
        root.setAttribute('data-export-root', '');
        app.shared.showLoading(root, 'Generating export bundle...');
        try {
            const data = await app.api('/export');
            const plugins = data.plugins || [];
            if (!plugins.length) {
                root.innerHTML = '<div class="empty-state"><p>No plugins to export.</p></div>';
                return;
            }
            let html = '';
            // Instructions card
            html += '<div class="alert-section" style="border-left-color:var(--accent);margin-bottom:var(--space-6)">' +
                '<h3 style="color:var(--accent);margin:0">Export to Claude</h3>' +
                '<p style="color:var(--text-secondary);margin:var(--space-2) 0 0;font-size:var(--text-sm)">' +
                    'Generate installation files for your plugins. Copy the install script below to set up Claude with your configuration.' +
                '</p>' +
            '</div>';
            // Summary stats
            html += renderSummary(data);
            // Actions
            html += '<div style="display:flex;gap:var(--space-3);margin-bottom:var(--space-6)">' +
                '<button class="btn btn-primary" id="btn-download-zip">Download Plugin ZIP</button>' +
            '</div>';
            // Section title
            html += '<div class="section-title">Plugin Bundles</div>';
            // Bundle cards
            html += plugins.map((plugin, idx) => renderBundleCard(plugin, idx)).join('');
            // Marketplace entry
            if (data.marketplace) {
                html += '<div class="card" style="border-left:3px solid var(--accent);margin-bottom:var(--space-4);padding:0;overflow:hidden">' +
                    '<div class="plugin-header" data-action="toggle-bundle" data-idx="mkt" style="cursor:pointer;display:flex;justify-content:space-between;align-items:center;padding:var(--space-4)">' +
                        '<div>' +
                            '<h3 style="margin:0;font-size:var(--text-base)">Marketplace Manifest</h3>' +
                            '<span class="badge badge-gray" style="margin-top:var(--space-1)">1 file</span>' +
                        '</div>' +
                        '<span class="expand-icon" id="bundle-icon-mkt" style="color:var(--text-tertiary);font-size:var(--text-sm)">&#9654;</span>' +
                    '</div>' +
                    '<div class="plugin-details" id="bundle-details-mkt" style="display:none">' +
                        '<div style="display:flex;justify-content:space-between;align-items:center;padding:var(--space-2) var(--space-4);border-bottom:1px solid var(--border-subtle)">' +
                            '<code style="font-size:var(--text-sm);color:var(--text-secondary)">' + escapeHtml(data.marketplace.path) + '</code>' +
                        '</div>' +
                        '<pre style="background:var(--bg-surface-raised);border-top:1px solid var(--border-subtle);padding:var(--space-4);font-size:var(--text-xs);overflow:auto;max-height:300px;margin:0">' + escapeHtml(data.marketplace.content) + '</pre>' +
                    '</div>' +
                '</div>';
            }
            // Install script section
            const installScript = generateInstallScript(data);
            html += '<div class="section-title" style="margin-top:var(--space-8)">Install Script</div>';
            html += '<div class="card" style="padding:0;overflow:hidden">' +
                '<div style="padding:var(--space-4);display:flex;justify-content:space-between;align-items:center;border-bottom:1px solid var(--border-subtle)">' +
                    '<span style="font-weight:600;font-size:var(--text-sm)">Bash Install Script</span>' +
                    '<button class="btn btn-sm btn-primary" id="copy-script-btn" data-action="copy-script">Copy Script</button>' +
                '</div>' +
                '<pre style="background:var(--bg-surface-raised);padding:var(--space-4);font-size:var(--text-xs);overflow:auto;max-height:400px;margin:0;border:none">' + escapeHtml(installScript) + '</pre>' +
            '</div>';
            root.innerHTML = html;
            exportData = data;
            // Event delegation
            root.addEventListener('click', (e) => {
                if (e.target.closest('#btn-download-zip')) {
                    downloadZip(exportData);
                    return;
                }
                const toggleEl = e.target.closest('[data-action="toggle-bundle"]');
                if (toggleEl) {
                    toggleBundle(toggleEl.getAttribute('data-idx'));
                    return;
                }
                const copyContentEl = e.target.closest('[data-action="copy-content"]');
                if (copyContentEl) {
                    copyContent(parseInt(copyContentEl.getAttribute('data-plugin-idx'), 10), parseInt(copyContentEl.getAttribute('data-file-idx'), 10), copyContentEl);
                    return;
                }
                const copyScriptEl = e.target.closest('[data-action="copy-script"]');
                if (copyScriptEl) {
                    copyScript(copyScriptEl);
                    return;
                }
            });
        } catch (err) {
            root.innerHTML = '<div class="empty-state"><p>Failed to generate export.</p></div>';
            app.Toast.show(err.message || 'Failed to generate export', 'error');
        }
    };
})(window.AdminApp);

(function(app) {
    const escapeHtml = app.escapeHtml;
    let allJobs = [];
    let searchQuery = '';
    function getFilteredJobs() {
        if (!searchQuery) return allJobs;
        const q = searchQuery.toLowerCase();
        return allJobs.filter((j) => (j.job_name || '').toLowerCase().indexOf(q) >= 0 ||
                   (j.schedule || '').toLowerCase().indexOf(q) >= 0 ||
                   (j.last_status || '').toLowerCase().indexOf(q) >= 0);
    }
    function renderToolbar() {
        return '<div class="toolbar">' +
            '<div class="search-group">' +
                '<input type="text" class="search-input" placeholder="Search jobs..." id="job-search" value="' + escapeHtml(searchQuery) + '">' +
            '</div>' +
        '</div>';
    }
    function renderStatusBadge(status) {
        if (!status) return '<span class="badge badge-secondary">Never run</span>';
        switch (status.toLowerCase()) {
            case 'success':
                return '<span class="badge badge-success">Success</span>';
            case 'failed':
                return '<span class="badge badge-danger">Failed</span>';
            case 'running':
                return '<span class="badge badge-info">Running</span>';
            default:
                return '<span class="badge badge-secondary">' + escapeHtml(status) + '</span>';
        }
    }
    function renderEnabledBadge(enabled) {
        if (enabled) {
            return '<span class="badge badge-success">Enabled</span>';
        }
        return '<span class="badge badge-secondary">Disabled</span>';
    }
    function formatJobName(name) {
        return name.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
    }
    function renderJobsTable(jobs) {
        if (!jobs.length) {
            return '<div class="empty-state"><p>No jobs match your search.</p></div>';
        }
        const rows = jobs.map((j) => {
            const lastRun = j.last_run ? app.formatRelativeTime(j.last_run) : 'Never';
            const nextRun = j.next_run ? app.formatRelativeTime(j.next_run) : '\u2014';
            const errorTitle = j.last_error ? ' title="' + escapeHtml(j.last_error) + '"' : '';
            return '<tr>' +
                '<td style="font-weight:500">' + escapeHtml(formatJobName(j.job_name)) +
                    '<div style="color:var(--text-secondary);font-size:var(--text-xs);font-weight:400">' + escapeHtml(j.job_name) + '</div>' +
                '</td>' +
                '<td><code style="background:var(--bg-surface-raised);padding:2px 8px;border-radius:var(--radius-xs);font-size:var(--text-xs)">' + escapeHtml(j.schedule) + '</code></td>' +
                '<td>' + renderEnabledBadge(j.enabled) + '</td>' +
                '<td>' + escapeHtml(lastRun) + '</td>' +
                '<td' + errorTitle + '>' + renderStatusBadge(j.last_status) + '</td>' +
                '<td>' + escapeHtml(nextRun) + '</td>' +
                '<td style="text-align:right">' + j.run_count + '</td>' +
            '</tr>';
        }).join('');
        return '<div class="table-container"><div class="table-scroll">' +
            '<table class="data-table">' +
                '<thead><tr>' +
                    '<th>Job</th>' +
                    '<th>Schedule</th>' +
                    '<th>Enabled</th>' +
                    '<th>Last Run</th>' +
                    '<th>Status</th>' +
                    '<th>Next Run</th>' +
                    '<th style="text-align:right">Runs</th>' +
                '</tr></thead>' +
                '<tbody>' + rows + '</tbody>' +
            '</table>' +
        '</div></div>';
    }
    function renderPage(root) {
        const filtered = getFilteredJobs();
        root.innerHTML = renderToolbar() + renderJobsTable(filtered);
    }
    async function loadJobs(root) {
        app.shared.showLoading(root, 'Loading jobs...');
        try {
            const data = await app.api('/jobs');
            allJobs = data || [];
            renderPage(root);
        } catch (err) {
            root.innerHTML = '<div class="empty-state"><p>Failed to load jobs.</p></div>';
            app.Toast.show(err.message || 'Failed to load jobs', 'error');
        }
    }
    app.renderJobs = (selector) => {
        const root = document.querySelector(selector);
        if (!root) return;
        loadJobs(root);
        app.shared.createDebouncedSearch(root, 'job-search', (value) => {
            searchQuery = value;
            renderPage(root);
        });
    };
})(window.AdminApp);

(function(app) {
    const escapeHtml = app.escapeHtml;
    function todayStr() {
        return new Date().toISOString().split('T')[0];
    }
    function thirtyDaysAgoStr() {
        const d = new Date();
        d.setDate(d.getDate() - 30);
        return d.toISOString().split('T')[0];
    }
    function renderToolbar() {
        return '<div class="toolbar">' +
            '<div class="filter-group">' +
                '<div class="filter-label">From</div>' +
                '<input type="date" class="search-input" style="width:auto" id="audit-from" value="' + thirtyDaysAgoStr() + '">' +
            '</div>' +
            '<div class="filter-group">' +
                '<div class="filter-label">To</div>' +
                '<input type="date" class="search-input" style="width:auto" id="audit-to" value="' + todayStr() + '">' +
            '</div>' +
            '<div class="filter-group">' +
                '<div class="filter-label">Department</div>' +
                '<input type="text" class="search-input" style="width:150px" id="audit-dept" placeholder="All">' +
            '</div>' +
            '<div class="filter-group">' +
                '<div class="filter-label">User</div>' +
                '<input type="text" class="search-input" style="width:180px" id="audit-user" placeholder="All">' +
            '</div>' +
            '<div class="filter-group">' +
                '<div class="filter-label">Skill/Tool</div>' +
                '<input type="text" class="search-input" style="width:150px" id="audit-skill" placeholder="All">' +
            '</div>' +
            '<div class="filter-group">' +
                '<div class="filter-label">Format</div>' +
                '<select class="filter-select" id="audit-format">' +
                    '<option value="json">JSON</option>' +
                    '<option value="csv">CSV</option>' +
                '</select>' +
            '</div>' +
            '<button class="btn btn-secondary" id="btn-preview">Preview</button>' +
            '<button class="btn btn-primary" id="btn-download">Download</button>' +
        '</div>';
    }
    function buildQueryString() {
        const params = new URLSearchParams();
        const from = document.getElementById('audit-from').value;
        const to = document.getElementById('audit-to').value;
        const dept = document.getElementById('audit-dept').value.trim();
        const user = document.getElementById('audit-user').value.trim();
        const skill = document.getElementById('audit-skill').value.trim();
        const format = document.getElementById('audit-format').value;
        if (from) params.set('from', from + 'T00:00:00Z');
        if (to) params.set('to', to + 'T23:59:59Z');
        if (dept) params.set('department', dept);
        if (user) params.set('user_id', user);
        if (skill) params.set('skill', skill);
        params.set('format', format);
        return params.toString();
    }
    function eventBadgeClass(eventType) {
        if (!eventType) return 'badge-gray';
        const lower = eventType.toLowerCase();
        if (lower.indexOf('error') !== -1 || lower.indexOf('fail') !== -1) return 'badge-red';
        if (lower.indexOf('warn') !== -1) return 'badge-yellow';
        if (lower.indexOf('create') !== -1 || lower.indexOf('success') !== -1) return 'badge-green';
        return 'badge-blue';
    }
    function renderPreviewTable(rows) {
        if (!rows.length) {
            return '<div class="empty-state"><p>No events match the filters.</p></div>';
        }
        const tableRows = rows.slice(0, 100).map((r) => '<tr>' +
                '<td><span class="date">' + escapeHtml(app.formatDate(r.created_at)) + '</span></td>' +
                '<td>' + escapeHtml(r.display_name || r.user_id) + '</td>' +
                '<td>' + escapeHtml(r.department || '-') + '</td>' +
                '<td><span class="badge ' + eventBadgeClass(r.event_type) + '">' + escapeHtml(r.event_type) + '</span></td>' +
                '<td><code style="font-size:var(--text-xs);background:var(--bg-surface-raised);padding:2px 6px;border-radius:var(--radius-xs)">' + escapeHtml(r.tool_name || '-') + '</code></td>' +
                '<td>' + escapeHtml(r.plugin_id || '-') + '</td>' +
            '</tr>').join('');
        const info = '<div style="margin-top:var(--space-3);font-size:var(--text-sm);color:var(--text-tertiary)">' +
            rows.length + ' event' + (rows.length !== 1 ? 's' : '') + ' found' +
            (rows.length > 100 ? ' &mdash; showing first 100. Download for full data.' : '') +
        '</div>';
        return '<div class="table-container" style="margin-top:var(--space-4)"><div class="table-scroll">' +
            '<table class="data-table">' +
                '<thead><tr>' +
                    '<th>Timestamp</th><th>User</th><th>Department</th><th>Event Type</th><th>Tool</th><th>Plugin</th>' +
                '</tr></thead>' +
                '<tbody>' + tableRows + '</tbody>' +
            '</table>' +
        '</div></div>' + info;
    }
    app.renderAudit = (selector) => {
        const root = document.querySelector(selector);
        if (!root) return;
        root.innerHTML = renderToolbar() + '<div id="preview-area"></div>';
        root.addEventListener('click', async (e) => {
            if (e.target.closest('#btn-preview')) {
                const preview = document.getElementById('preview-area');
                app.shared.showLoading(preview, 'Loading...');
                const qs = buildQueryString().replace('format=csv', 'format=json');
                try {
                    let rows = await app.api('/export/usage?' + qs);
                    rows = rows || [];
                    preview.innerHTML = renderPreviewTable(rows);
                } catch (err) {
                    preview.innerHTML = '<div class="empty-state"><p>Failed to load data.</p></div>';
                    app.Toast.show(err.message || 'Export failed', 'error');
                }
                return;
            }
            if (e.target.closest('#btn-download')) {
                const qs = buildQueryString();
                const format = document.getElementById('audit-format').value;
                if (format === 'csv') {
                    window.open(app.API_BASE + '/export/usage?' + qs, '_blank');
                } else {
                    try {
                        const rows = await app.api('/export/usage?' + qs);
                        const blob = new Blob([JSON.stringify(rows, null, 2)], { type: 'application/json' });
                        const url = URL.createObjectURL(blob);
                        const a = document.createElement('a');
                        a.href = url;
                        a.download = 'usage-export.json';
                        a.click();
                        URL.revokeObjectURL(url);
                    } catch (err) {
                        app.Toast.show(err.message || 'Download failed', 'error');
                    }
                }
            }
        });
    };
})(window.AdminApp);

(function(app) {
    const escapeHtml = app.escapeHtml;
    function calcProgress(xp) {
        const rank = app.constants.getUserRank(xp);
        const next = app.constants.getNextUserRank(rank);
        if (!next) return 100;
        const range = next.xp - rank.xp;
        if (range <= 0) return 100;
        return Math.min(100, Math.round(((xp - rank.xp) / range) * 100));
    }
    function renderRankBadge(xp) {
        const rank = app.constants.getUserRank(xp);
        return '<span class="rank-badge rank-' + rank.level + '">' + escapeHtml(rank.name) + '</span>';
    }
    function renderXpBar(xp) {
        const pct = calcProgress(xp);
        const rank = app.constants.getUserRank(xp);
        return '<span class="xp-value">' + xp + '</span>' +
            '<div class="xp-progress">' +
                '<div class="xp-progress-fill" style="width:' + pct + '%;background:' + rank.color + '"></div>' +
            '</div>';
    }
    function renderStreakBadge(streak) {
        if (!streak || streak <= 0) return '<span class="text-tertiary">-</span>';
        const cls = streak >= 7 ? ' hot' : '';
        return '<span class="streak-badge' + cls + '">&#128293; ' + streak + 'd</span>';
    }
    function updateStats(data) {
        const users = data || [];
        const ranked = users.length;
        let totalXp = 0;
        let totalAch = 0;
        let activeStreaks = 0;
        users.forEach((u) => {
            totalXp += (u.xp || 0);
            totalAch += (u.achievement_count || 0);
            if (u.current_streak && u.current_streak > 0) activeStreaks++;
        });
        const avgXp = ranked > 0 ? Math.round(totalXp / ranked) : 0;
        const statRanked = document.getElementById('stat-ranked');
        const statAvgXp = document.getElementById('stat-avg-xp');
        const statAch = document.getElementById('stat-achievements');
        const statStreaks = document.getElementById('stat-streaks');
        if (statRanked) statRanked.querySelector('.value').textContent = ranked;
        if (statAvgXp) statAvgXp.querySelector('.value').textContent = avgXp;
        if (statAch) statAch.querySelector('.value').textContent = totalAch;
        if (statStreaks) statStreaks.querySelector('.value').textContent = activeStreaks;
    }
    function renderTable(users) {
        if (!users || !users.length) {
            return '<div class="table-container"><div class="empty-state"><p>No leaderboard data yet.</p></div></div>';
        }
        const rows = users.map((u, i) => {
            const pos = i + 1;
            const initials = app.getUserInitials(u.display_name || u.user_id);
            const name = escapeHtml(u.display_name || ('User-' + u.user_id.slice(0, 8)));
            const dept = u.department ? '<div class="stat-label">' + escapeHtml(u.department) + '</div>' : '';
            const topClass = pos <= 3 ? ' leaderboard-row top-' + pos : '';
            const relTime = u.last_active ? app.formatRelativeTime(u.last_active) : '-';
            const userId = encodeURIComponent(u.user_id);
            return '<tr class="clickable-row' + topClass + '" data-user-href="/admin/user/?id=' + userId + '">' +
                '<td><span class="leaderboard-rank' + (pos <= 3 ? ' leaderboard-rank-' + pos : '') + '">' + pos + '</span></td>' +
                '<td><div class="user-cell"><div class="user-avatar">' + escapeHtml(initials) + '</div><div><span>' + name + '</span>' + dept + '</div></div></td>' +
                '<td>' + renderRankBadge(u.xp || 0) + '</td>' +
                '<td class="numeric"><div>' + renderXpBar(u.xp || 0) + '</div></td>' +
                '<td>' + renderStreakBadge(u.current_streak) + '</td>' +
                '<td class="numeric">' + (u.achievement_count || 0) + '</td>' +
                '<td><span title="' + escapeHtml(app.formatDate(u.last_active)) + '">' + escapeHtml(relTime) + '</span></td>' +
            '</tr>';
        }).join('');
        return '<div class="table-container"><div class="table-scroll">' +
            '<table class="data-table">' +
                '<thead><tr>' +
                    '<th class="col-rank">#</th>' +
                    '<th>User</th>' +
                    '<th>Rank</th>' +
                    '<th class="numeric">XP</th>' +
                    '<th>Streak</th>' +
                    '<th class="numeric">Achievements</th>' +
                    '<th>Last Active</th>' +
                '</tr></thead>' +
                '<tbody>' + rows + '</tbody>' +
            '</table>' +
        '</div></div>';
    }
    function renderDepartments(departments) {
        if (!departments || !departments.length) {
            return '<div class="empty-state"><p>No department data available.</p></div>';
        }
        const cards = departments.map((d) => {
            const topUser = d.top_user
                ? '<div class="mt-3 text-sm text-muted">Top: <strong>' + escapeHtml(d.top_user.display_name || d.top_user.user_id) + '</strong> (' + (d.top_user.xp || 0) + ' XP)</div>'
                : '';
            return '<div class="dept-card">' +
                '<div class="card-title">' + escapeHtml(d.department) + '</div>' +
                '<div class="gamification-stats grid-3col">' +
                    '<div><div class="stat-label">Total XP</div><div class="stat-value">' + (d.total_xp || 0) + '</div></div>' +
                    '<div><div class="stat-label">Avg XP</div><div class="stat-value">' + (d.avg_xp || 0) + '</div></div>' +
                    '<div><div class="stat-label">Users</div><div class="stat-value">' + (d.user_count || 0) + '</div></div>' +
                '</div>' +
                topUser +
            '</div>';
        }).join('');
        return '<div class="grid-auto-fill">' + cards + '</div>';
    }
    async function loadLeaderboard() {
        const root = document.getElementById('leaderboard-content');
        if (!root) return;
        root.innerHTML = '<div class="loading-center"><div class="loading-spinner"></div></div>';
        try {
            const data = await app.api('/gamification/leaderboard?limit=50&offset=0');
            const users = data.leaderboard || data || [];
            updateStats(users);
            root.innerHTML = renderTable(users);
            root.addEventListener('click', function(e) {
                const tr = e.target.closest('[data-user-href]');
                if (tr) window.location.href = tr.getAttribute('data-user-href');
            });
        } catch (err) {
            root.innerHTML = '<div class="empty-state"><p>Failed to load leaderboard.</p></div>';
            app.Toast.show(err.message || 'Failed to load leaderboard', 'error');
        }
    }
    async function loadDepartments() {
        const root = document.getElementById('department-content');
        if (!root) return;
        root.innerHTML = '<div class="loading-center"><div class="loading-spinner"></div></div>';
        try {
            const data = await app.api('/gamification/departments');
            const departments = data.departments || data || [];
            root.innerHTML = renderDepartments(departments);
        } catch (err) {
            root.innerHTML = '<div class="empty-state"><p>Failed to load department data.</p></div>';
            app.Toast.show(err.message || 'Failed to load departments', 'error');
        }
    }
    function initTabs() {
        const buttons = document.querySelectorAll('.tab-btn');
        const leaderboardEl = document.getElementById('leaderboard-content');
        const departmentEl = document.getElementById('department-content');
        buttons.forEach((btn) => {
            btn.addEventListener('click', () => {
                buttons.forEach((b) => { b.classList.remove('active'); });
                btn.classList.add('active');
                const tab = btn.getAttribute('data-tab');
                if (tab === 'all') {
                    leaderboardEl.classList.remove('hidden');
                    departmentEl.classList.add('hidden');
                    loadLeaderboard();
                } else {
                    leaderboardEl.classList.add('hidden');
                    departmentEl.classList.remove('hidden');
                    loadDepartments();
                }
            });
        });
    }
    app.renderLeaderboard = () => {
        initTabs();
        loadLeaderboard();
    };
})(window.AdminApp);

(function(app) {
    const escapeHtml = app.escapeHtml;
    const CATEGORY_ORDER = ['First Steps', 'Volume', 'Exploration', 'Mastery', 'Streaks', 'Social'];
    function groupByCategory(achievements) {
        const groups = {};
        CATEGORY_ORDER.forEach((cat) => { groups[cat] = []; });
        Object.keys(app.constants.ACHIEVEMENT_DEFS).forEach((key) => {
            const def = app.constants.ACHIEVEMENT_DEFS[key];
            const earned = achievements[key] || null;
            const item = {
                key,
                name: def.name,
                description: def.description,
                icon: def.icon,
                category: def.category,
                unlocked: !!earned,
                unlock_pct: earned ? (earned.unlock_pct || 0) : 0,
            };
            if (groups[def.category]) {
                groups[def.category].push(item);
            }
        });
        return groups;
    }
    function renderAchievementCard(item) {
        const cls = item.unlocked ? 'achievement-card unlocked' : 'achievement-card locked';
        const pct = item.unlocked ? 100 : (item.unlock_pct || 0);
        const bar = '<div class="unlock-bar"><div class="unlock-bar-fill" style="width:' + pct + '%"></div></div>';
        return '<div class="' + cls + '">' +
            '<div class="achievement-icon">' + item.icon + '</div>' +
            '<div style="font-weight:600;font-size:var(--text-sm);color:var(--text-primary)">' + escapeHtml(item.name) + '</div>' +
            '<div style="font-size:var(--text-xs);color:var(--text-tertiary);margin-top:var(--space-1)">' + escapeHtml(item.description) + '</div>' +
            bar +
        '</div>';
    }
    function renderAchievementsContent(data) {
        const achievements = data.achievements || data || {};
        const groups = groupByCategory(achievements);
        let html = '';
        CATEGORY_ORDER.forEach((cat) => {
            const items = groups[cat];
            if (!items || !items.length) return;
            const cards = items.map(renderAchievementCard).join('');
            html += '<div style="margin-bottom:var(--space-6)">' +
                '<div class="section-title">' + escapeHtml(cat) + '</div>' +
                '<div class="achievement-grid">' + cards + '</div>' +
            '</div>';
        });
        return html || '<div class="empty-state"><p>No achievements defined.</p></div>';
    }
    app.renderAchievements = () => {
        const root = document.getElementById('achievements-content');
        if (!root) return;
        root.innerHTML = '<div class="loading-center"><div class="loading-spinner"></div></div>';
        async function load() {
            try {
                const data = await app.api('/gamification/achievements');
                root.innerHTML = renderAchievementsContent(data);
            } catch (err) {
                root.innerHTML = '<div class="empty-state"><p>Failed to load achievements.</p></div>';
                app.Toast.show(err.message || 'Failed to load achievements', 'error');
            }
        }
        load();
    };
})(window.AdminApp);

