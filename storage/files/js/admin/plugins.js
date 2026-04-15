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
