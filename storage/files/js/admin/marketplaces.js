(function(app) {
    'use strict';

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
            fetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id) + '/sync', {
                method: 'POST',
                credentials: 'include'
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
            overlay.innerHTML = '<div class="confirm-dialog">' +
                '<h3 style="margin:0 0 var(--sp-space-3)">Publish to GitHub?</h3>' +
                '<p style="margin:0 0 var(--sp-space-4);color:var(--sp-text-secondary);font-size:var(--sp-text-sm)">This will push the current marketplace plugins to the linked GitHub repository. Any remote changes will be overwritten.</p>' +
                '<div style="display:flex;gap:var(--sp-space-3);justify-content:flex-end">' +
                    '<button class="btn btn-secondary" data-confirm-cancel>Cancel</button>' +
                    '<button class="btn btn-primary" data-confirm-publish>Publish</button>' +
                '</div>' +
            '</div>';
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
                    fetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id) + '/publish', {
                        method: 'POST',
                        credentials: 'include'
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
        overlay.innerHTML = '<div class="confirm-dialog">' +
            '<h3 style="margin:0 0 var(--sp-space-3)">Delete Marketplace?</h3>' +
            '<p style="margin:0 0 var(--sp-space-2);color:var(--sp-text-secondary);font-size:var(--sp-text-sm)">You are about to delete <strong>' + app.escapeHtml(marketplaceId) + '</strong>.</p>' +
            '<p style="margin:0 0 var(--sp-space-5);color:var(--sp-text-secondary);font-size:var(--sp-text-sm)">This will remove the marketplace and all plugin associations. This action cannot be undone.</p>' +
            '<div style="display:flex;gap:var(--sp-space-3);justify-content:flex-end">' +
                '<button class="btn btn-secondary" data-confirm-cancel>Cancel</button>' +
                '<button class="btn btn-danger" data-confirm-delete="' + app.escapeHtml(marketplaceId) + '">Delete Marketplace</button>' +
            '</div>' +
        '</div>';
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
                    const resp = await fetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id), {
                        method: 'DELETE',
                        credentials: 'include'
                    });
                    if (resp.ok) {
                        app.Toast.show('Marketplace deleted', 'success');
                        setTimeout(() => { window.location.reload(); }, 500);
                    } else {
                        const data = await resp.json().catch(() => ({}));
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

            fetch(app.API_BASE + '/plugins', { credentials: 'include' })
                .then((r) => r.json())
                .then((allPlugins) => {
                    const currentIds = {};
                    (mktData.plugin_ids || []).forEach((pid) => { currentIds[pid] = true; });

                    let html = '<div class="assign-panel-checklist">';
                    if (!allPlugins.length) {
                        html += '<p style="color:var(--sp-text-tertiary);font-size:var(--sp-text-sm)">No plugins available.</p>';
                    } else {
                        allPlugins.forEach((p) => {
                            const pid = p.id || p.plugin_id;
                            const pname = p.name || pid;
                            const checked = currentIds[pid] ? ' checked' : '';
                            html += '<label class="acl-checkbox-row" style="display:flex;align-items:center;gap:var(--sp-space-2);padding:var(--sp-space-1) 0;cursor:pointer">' +
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
                        saveBtn.addEventListener('click', async () => {
                            const checked = panelApi.panel.querySelectorAll('input[name="plugin_id"]:checked');
                            const ids = [];
                            checked.forEach((cb) => { ids.push(cb.value); });
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
                                    setTimeout(() => { window.location.reload(); }, 500);
                                } else {
                                    const data = await resp.json().catch(() => ({}));
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

            fetch(app.API_BASE + '/plugins', { credentials: 'include' })
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

                    html += '<div class="form-group">' +
                        '<label class="field-label">Roles</label>' +
                        '<div style="display:flex;flex-wrap:wrap;gap:var(--sp-space-1);padding:var(--sp-space-2) 0">';
                    allRoles.forEach((r) => {
                        const val = r.value || r;
                        const checked = currentRoles[val] ? ' checked' : '';
                        html += '<label style="display:inline-flex;align-items:center;gap:var(--sp-space-2);margin-right:var(--sp-space-3);font-size:var(--sp-text-sm);cursor:pointer">' +
                            '<input type="checkbox" name="roles" value="' + app.escapeHtml(val) + '"' + checked + '> ' +
                            app.escapeHtml(val) + '</label>';
                    });
                    html += '</div></div>';

                    html += '<div class="form-group">' +
                        '<label class="field-label">Departments</label>' +
                        '<div class="checklist-container" style="max-height:300px;overflow-y:auto;border:1px solid var(--sp-border-subtle);border-radius:var(--sp-radius-md);padding:var(--sp-space-2)">' +
                        '<div class="checklist-item" style="display:flex;align-items:center;gap:var(--sp-space-2);padding:var(--sp-space-2);border-bottom:1px solid var(--sp-border-subtle)">' +
                        '<input type="checkbox" id="panel-dept-check-all">' +
                        '<label for="panel-dept-check-all" style="flex:1;font-size:var(--sp-text-sm);cursor:pointer;color:var(--sp-text-primary);font-weight:600">Check all</label>' +
                        '</div>';
                    allDepts.forEach((d, i) => {
                        const val = d.value || d.name || d;
                        const checked = currentDepts[val] ? ' checked' : '';
                        const defaultChecked = deptDefaults[val] ? ' checked' : '';
                        html += '<div class="checklist-item" style="display:flex;align-items:center;gap:var(--sp-space-2);padding:var(--sp-space-2)">' +
                            '<input type="checkbox" name="departments" value="' + app.escapeHtml(val) + '"' + checked + ' id="panel-dept-' + i + '">' +
                            '<label for="panel-dept-' + i + '" style="flex:1;font-size:var(--sp-text-sm);cursor:pointer;color:var(--sp-text-primary)">' + app.escapeHtml(val) + '</label>' +
                            '<span class="badge badge-gray" style="font-size:var(--sp-text-xs)">' + (d.user_count || 0) + ' users</span>' +
                            '<label style="display:inline-flex;align-items:center;gap:4px;font-size:var(--sp-text-xs);color:var(--sp-text-secondary);cursor:pointer;white-space:nowrap">' +
                            '<input type="checkbox" name="dept_default_' + val + '"' + defaultChecked + '> Default</label>' +
                            '</div>';
                    });
                    html += '</div>' +
                        '<span class="field-hint" style="margin-top:var(--sp-space-2);display:block">At least one department is required.</span>' +
                        '</div>';

                    html += '<div class="form-group">' +
                        '<label class="field-label">Plugins</label>' +
                        '<input type="text" class="field-input" placeholder="Filter plugins..." id="panel-plugin-filter" style="margin-bottom:var(--sp-space-2)">' +
                        '<div class="checklist-container" style="max-height:200px;overflow-y:auto;border:1px solid var(--sp-border-subtle);border-radius:var(--sp-radius-md);padding:var(--sp-space-2)">';
                    allPlugins.forEach((p, i) => {
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
                    setTimeout(() => { window.location.reload(); }, 500);
                } else {
                    const data = await resp.json().catch(() => ({}));
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
                    const created = await resp.json().catch(() => ({}));
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
                    setTimeout(() => { window.location.reload(); }, 500);
                } else {
                    const data = await resp.json().catch(() => ({}));
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
                        setTimeout(() => { window.location.href = '/admin/org/marketplaces/'; }, 500);
                    } else {
                        const data = await resp.json().catch(() => ({}));
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
                        const created = await resp.json().catch(() => ({}));
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
                        setTimeout(() => { window.location.href = '/admin/org/marketplaces/'; }, 500);
                    } else {
                        const data = await resp.json().catch(() => ({}));
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
