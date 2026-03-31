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
