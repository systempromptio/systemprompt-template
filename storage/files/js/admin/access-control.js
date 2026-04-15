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
