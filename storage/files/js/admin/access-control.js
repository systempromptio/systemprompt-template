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
