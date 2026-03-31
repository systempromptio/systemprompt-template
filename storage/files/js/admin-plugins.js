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
                setBody: function(content) {
                    var body = panel.querySelector('[data-panel-body]');
                    if (body) body.replaceChildren(content);
                },
                setFooter: function(content) {
                    var footer = panel.querySelector('[data-panel-footer]');
                    if (footer) footer.replaceChildren(content);
                },
                panel: panel
            };

            if (closeBtn) closeBtn.addEventListener('click', api.close);
            if (overlay) overlay.addEventListener('click', api.close);

            return api;
        },

        initAssignPanel: function(config) {
            var panelApi = OrgCommon.initSidePanel(config.panelId);
            if (!panelApi) return null;

            return {
                open: function(entityId, entityName, currentPluginIds) {
                    panelApi.setTitle('Assign ' + (entityName || entityId));

                    var allPlugins = config.allPlugins || [];
                    var currentSet = {};
                    (currentPluginIds || []).forEach(function(id) { currentSet[id] = true; });

                    var checklist = document.createElement('div');
                    checklist.className = 'assign-panel-checklist';
                    if (allPlugins.length === 0) {
                        var noPlugins = document.createElement('p');
                        noPlugins.style.cssText = 'color:var(--sp-text-tertiary);font-size:var(--sp-text-sm)';
                        noPlugins.textContent = 'No plugins available.';
                        checklist.append(noPlugins);
                    } else {
                        allPlugins.forEach(function(p) {
                            var label = document.createElement('label');
                            label.className = 'acl-checkbox-row';
                            var input = document.createElement('input');
                            input.type = 'checkbox';
                            input.name = 'plugin_id';
                            input.value = p.id;
                            if (currentSet[p.id]) input.checked = true;
                            var span = document.createElement('span');
                            span.className = 'acl-checkbox-label';
                            span.textContent = p.name || p.id;
                            label.append(input, span);
                            checklist.append(label);
                        });
                    }
                    panelApi.setBody(checklist);

                    var footerFrag = document.createDocumentFragment();
                    var cancelBtn = document.createElement('button');
                    cancelBtn.className = 'btn btn-secondary';
                    cancelBtn.setAttribute('data-panel-close', '');
                    cancelBtn.textContent = 'Cancel';
                    cancelBtn.addEventListener('click', panelApi.close);
                    var saveBtn = document.createElement('button');
                    saveBtn.className = 'btn btn-primary';
                    saveBtn.setAttribute('data-assign-save', '');
                    saveBtn.setAttribute('data-entity-id', entityId);
                    saveBtn.textContent = 'Save';
                    footerFrag.append(cancelBtn, document.createTextNode(' '), saveBtn);
                    panelApi.setFooter(footerFrag);

                    panelApi.open();
                },
                close: panelApi.close,
                panel: panelApi
            };
        },

        initEditPanel: function(config) {
            var panelApi = OrgCommon.initSidePanel(config.panelId);
            if (!panelApi) return null;
            var currentEntityId = null;

            function buildForm(entityData) {
                var form = document.createElement('form');
                form.className = 'edit-panel-form';
                (config.fields || []).forEach(function(f) {
                    var val = entityData[f.name] || '';
                    if (Array.isArray(val)) val = val.join(', ');
                    var group = document.createElement('div');
                    group.className = 'form-group';
                    var label = document.createElement('label');
                    label.className = 'form-label';
                    label.textContent = f.label;
                    group.append(label);
                    if (f.type === 'textarea') {
                        var textarea = document.createElement('textarea');
                        textarea.className = 'form-control';
                        textarea.name = f.name;
                        textarea.rows = f.rows || 10;
                        textarea.value = val;
                        group.append(textarea);
                    } else {
                        var input = document.createElement('input');
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
            }

            function collectFormData() {
                const form = panelApi.panel.querySelector('.edit-panel-form');
                if (!form) return {};
                const body = {};
                (config.fields || []).forEach(function(f) {
                    const el = form.querySelector('[name="' + f.name + '"]');
                    if (!el) return;
                    const val = el.value;
                    if (f.name === 'tags') {
                        body[f.name] = val.split(',').map(function(t) { return t.trim(); }).filter(Boolean);
                    } else {
                        body[f.name] = val;
                    }
                });
                return body;
            }

            document.addEventListener('click', function(e) {
                const btn = e.target.closest('[data-edit-save]');
                if (!btn) return;
                btn.disabled = true;
                btn.textContent = 'Saving...';
                const body = collectFormData();
                const url = config.apiBasePath + encodeURIComponent(currentEntityId);
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
                    panelApi.setTitle('Edit ' + (entityData.name || entityId));
                    panelApi.setBody(buildForm(entityData));
                    var footerFrag = document.createDocumentFragment();
                    var cancelBtn = document.createElement('button');
                    cancelBtn.className = 'btn btn-secondary';
                    cancelBtn.setAttribute('data-panel-close', '');
                    cancelBtn.textContent = 'Cancel';
                    cancelBtn.addEventListener('click', panelApi.close);
                    var saveBtn = document.createElement('button');
                    saveBtn.className = 'btn btn-primary';
                    saveBtn.setAttribute('data-edit-save', '');
                    saveBtn.textContent = 'Save';
                    footerFrag.append(cancelBtn, document.createTextNode(' '), saveBtn);
                    panelApi.setFooter(footerFrag);
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
                const count = Object.keys(selected).length;
                const countEl = document.querySelector('[data-bulk-count]');
                if (countEl) countEl.textContent = count;
                const bar = document.getElementById(barId);
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
                try { data = JSON.parse(data); } catch (e) {
                    var fallback = document.createElement('pre');
                    fallback.className = 'json-view';
                    fallback.textContent = data;
                    return fallback;
                }
            }
            var pre = document.createElement('pre');
            pre.className = 'json-view';
            pre.textContent = JSON.stringify(data, null, 2);
            return pre;
        },

        renderRoleBadges: function(roles) {
            var frag = document.createDocumentFragment();
            if (!roles || !roles.length) {
                var b = document.createElement('span');
                b.className = 'badge badge-gray';
                b.textContent = 'All';
                frag.append(b);
                return frag;
            }
            var assigned = roles.filter(function(r) { return r.assigned; });
            if (!assigned.length) {
                var b2 = document.createElement('span');
                b2.className = 'badge badge-gray';
                b2.textContent = 'All';
                frag.append(b2);
                return frag;
            }
            assigned.forEach(function(r) {
                var badge = document.createElement('span');
                badge.className = 'badge badge-blue';
                badge.textContent = r.name;
                frag.append(badge, document.createTextNode(' '));
            });
            return frag;
        },

        renderDeptBadges: function(departments) {
            var frag = document.createDocumentFragment();
            if (!departments || !departments.length) {
                var b = document.createElement('span');
                b.className = 'badge badge-gray';
                b.textContent = 'None';
                frag.append(b);
                return frag;
            }
            var assigned = departments.filter(function(d) { return d.assigned; });
            if (!assigned.length) {
                var b2 = document.createElement('span');
                b2.className = 'badge badge-gray';
                b2.textContent = 'None';
                frag.append(b2);
                return frag;
            }
            assigned.forEach(function(d) {
                var badge = document.createElement('span');
                badge.className = 'badge ' + (d.default_included ? 'badge-yellow' : 'badge-green');
                badge.textContent = d.name;
                frag.append(badge, document.createTextNode(' '));
            });
            return frag;
        },

        renderPluginBadges: function(plugins) {
            var frag = document.createDocumentFragment();
            if (!plugins || !plugins.length) {
                var b = document.createElement('span');
                b.className = 'badge badge-gray';
                b.textContent = 'None';
                frag.append(b);
                return frag;
            }
            plugins.forEach(function(p) {
                var name = typeof p === 'string' ? p : (p.name || p.id || p);
                var badge = document.createElement('span');
                badge.className = 'badge badge-purple';
                badge.textContent = name;
                frag.append(badge, document.createTextNode(' '));
            });
            return frag;
        },

        initFilters: function(searchInputId, tableSelector, filters) {
            const table = document.querySelector(tableSelector);
            if (!table) return;

            function applyFilters() {
                const searchInput = document.getElementById(searchInputId);
                const q = (searchInput ? searchInput.value : '').toLowerCase().trim();
                const filterValues = filters.map(function(f) {
                    const sel = document.getElementById(f.selectId);
                    return { attr: f.dataAttr, value: sel ? sel.value : '' };
                });

                table.querySelectorAll('tbody tr.clickable-row').forEach(function(row) {
                    const matchSearch = !q ||
                        (row.getAttribute('data-name') || '').includes(q) ||
                        (row.getAttribute('data-skill-id') || row.getAttribute('data-agent-id') || '').toLowerCase().includes(q) ||
                        (row.getAttribute('data-description') || '').includes(q);

                    const matchFilters = filterValues.every(function(fv) {
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
            }

            filters.forEach(function(f) {
                const sel = document.getElementById(f.selectId);
                if (sel) sel.addEventListener('change', applyFilters);
            });

            let searchTimer = null;
            const searchInput = document.getElementById(searchInputId);
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

        initTimeAgo: function() {
            document.querySelectorAll('.metadata-timestamp').forEach(function(el) {
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

        var dialog = document.createElement('div');
        dialog.className = 'confirm-dialog';
        var h3 = document.createElement('h3');
        h3.style.cssText = 'margin:0 0 var(--sp-space-3)';
        h3.textContent = 'Delete Plugin?';
        var p1 = document.createElement('p');
        p1.style.cssText = 'margin:0 0 var(--sp-space-2);color:var(--sp-text-secondary);font-size:var(--sp-text-sm)';
        p1.append(document.createTextNode('You are about to delete '));
        var strong = document.createElement('strong');
        strong.textContent = pluginId;
        p1.append(strong);
        p1.append(document.createTextNode('.'));
        var p2 = document.createElement('p');
        p2.style.cssText = 'margin:0 0 var(--sp-space-5);color:var(--sp-text-secondary);font-size:var(--sp-text-sm)';
        p2.textContent = 'This will remove the plugin directory and all its configuration. This action cannot be undone.';
        var btnRow = document.createElement('div');
        btnRow.style.cssText = 'display:flex;gap:var(--sp-space-3);justify-content:flex-end';
        var cancelBtn = document.createElement('button');
        cancelBtn.className = 'btn btn-secondary';
        cancelBtn.setAttribute('data-confirm-cancel', '');
        cancelBtn.textContent = 'Cancel';
        var deleteBtn = document.createElement('button');
        deleteBtn.className = 'btn btn-danger';
        deleteBtn.setAttribute('data-confirm-delete', pluginId);
        deleteBtn.textContent = 'Delete Plugin';
        btnRow.append(cancelBtn, deleteBtn);
        dialog.append(h3, p1, p2, btnRow);
        overlay.append(dialog);

        document.body.append(overlay);
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

        var panelBody = document.createDocumentFragment();

        var overviewSection = document.createElement('div');
        overviewSection.className = 'config-panel-section';
        var overviewH4 = document.createElement('h4');
        overviewH4.textContent = 'Overview';
        var grid = document.createElement('div');
        grid.className = 'config-overview-grid';

        function addGridRow(labelText, valueContent) {
            var label = document.createElement('span');
            label.className = 'config-overview-label';
            label.textContent = labelText;
            var value = document.createElement('span');
            value.className = 'config-overview-value';
            if (typeof valueContent === 'string') {
                value.textContent = valueContent;
            } else {
                value.append(valueContent);
            }
            grid.append(label, value);
        }

        var idCode = document.createElement('code');
        idCode.textContent = data.id;
        addGridRow('ID', idCode);
        var statusBadge = document.createElement('span');
        statusBadge.className = data.enabled ? 'badge badge-green' : 'badge badge-gray';
        statusBadge.textContent = data.enabled ? 'Enabled' : 'Disabled';
        addGridRow('Status', statusBadge);
        addGridRow('Version', data.version || '\u2014');
        addGridRow('Category', data.category || '\u2014');
        addGridRow('Author', data.author_name || '\u2014');
        addGridRow('Description', data.description || '\u2014');

        overviewSection.append(overviewH4, grid);
        panelBody.append(overviewSection);

        var envSection = document.createElement('div');
        envSection.className = 'config-panel-section';
        var envH4 = document.createElement('h4');
        envH4.textContent = 'Environment';
        var envStatus = document.createElement('div');
        envStatus.id = 'panel-env-status';
        envStatus.textContent = 'Loading...';
        envSection.append(envH4, envStatus);
        panelBody.append(envSection);

        document.getElementById('panel-body').replaceChildren(panelBody);

        var panelFooter = document.getElementById('panel-footer');
        panelFooter.replaceChildren();
        if (data.id !== 'custom') {
            var editLink = document.createElement('a');
            editLink.href = '/admin/org/plugins/edit/?id=' + encodeURIComponent(data.id);
            editLink.className = 'btn btn-primary';
            editLink.textContent = 'Edit Plugin';
            var envBtn = document.createElement('button');
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
            var emptyDiv = document.createElement('div');
            emptyDiv.className = 'empty-state';
            var emptyP = document.createElement('p');
            emptyP.textContent = 'N/A';
            emptyDiv.append(emptyP);
            document.getElementById('panel-env-status').replaceChildren(emptyDiv);
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
                const row = btn.closest('tr');
                if (row) row.remove();
                const countEl = document.querySelector('[data-count="' + resourceType + '"][data-for-plugin="' + pluginId + '"]');
                if (countEl) countEl.textContent = updatedIds.length;
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

        app.events.on('click', '[data-add-to-plugin]', function(e, btn) {
            e.stopPropagation();
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

                var overlay = document.createElement('div');
                overlay.className = 'confirm-overlay';

                var addDialog = document.createElement('div');
                addDialog.className = 'confirm-dialog';
                var addH3 = document.createElement('h3');
                addH3.style.cssText = 'margin:0 0 var(--sp-space-3)';
                addH3.textContent = 'Add ' + resourceType.replace('_', ' ');

                var checklist = document.createElement('div');
                checklist.className = 'add-checklist';
                available.forEach(function(item) {
                    var id = typeof item === 'string' ? item : (item.id || item.skill_id || item.agent_id);
                    var itemName = typeof item === 'string' ? item : (item.name || item.id || item.skill_id);
                    var lbl = document.createElement('label');
                    var cb = document.createElement('input');
                    cb.type = 'checkbox';
                    cb.value = id;
                    lbl.append(cb, document.createTextNode(' ' + itemName));
                    checklist.append(lbl);
                });

                var addBtnRow = document.createElement('div');
                addBtnRow.style.cssText = 'display:flex;gap:var(--sp-space-3);justify-content:flex-end;margin-top:var(--sp-space-3)';
                var addCancelBtn = document.createElement('button');
                addCancelBtn.className = 'btn btn-secondary';
                addCancelBtn.setAttribute('data-add-cancel', '');
                addCancelBtn.textContent = 'Cancel';
                var addConfirmBtn = document.createElement('button');
                addConfirmBtn.className = 'btn btn-primary';
                addConfirmBtn.setAttribute('data-add-confirm', '');
                addConfirmBtn.textContent = 'Add Selected';
                addBtnRow.append(addCancelBtn, addConfirmBtn);
                addDialog.append(addH3, checklist, addBtnRow);
                overlay.append(addDialog);

                document.body.append(overlay);

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

    function createEmptyState(text) {
        var div = document.createElement('div');
        div.className = 'empty-state';
        var p = document.createElement('p');
        p.textContent = text;
        div.append(p);
        return div;
    }

    function loadEnvStatus(pluginId, container) {
        var loadingDiv = document.createElement('div');
        loadingDiv.style.cssText = 'padding:var(--sp-space-4);color:var(--sp-text-tertiary);font-size:var(--sp-text-sm)';
        loadingDiv.textContent = 'Loading variables...';
        container.replaceChildren(loadingDiv);

        app.api('/plugins/' + encodeURIComponent(pluginId) + '/env').then(function(data) {
            var defs = data.definitions || [];
            var stored = data.stored || [];
            if (!defs.length && !stored.length) {
                container.replaceChildren(createEmptyState('No environment variables defined for this plugin.'));
                return;
            }
            var storedMap = {};
            stored.forEach(function(v) { storedMap[v.var_name] = v; });
            var frag = document.createDocumentFragment();
            defs.forEach(function(def) {
                var s = storedMap[def.name];
                var hasValue = s && s.var_value && s.var_value !== '';

                var item = document.createElement('div');
                item.className = 'detail-item';
                var info = document.createElement('div');
                info.className = 'detail-item-info';
                var nameDiv = document.createElement('div');
                nameDiv.className = 'detail-item-name';
                var code = document.createElement('code');
                code.style.cssText = 'background:var(--sp-bg-surface-raised);padding:1px 6px;border-radius:var(--sp-radius-xs);font-size:var(--sp-text-sm)';
                code.textContent = def.name;
                nameDiv.append(code, document.createTextNode(' '));
                var valBadge = document.createElement('span');
                valBadge.className = hasValue ? 'badge badge-green' : 'badge badge-red';
                valBadge.textContent = hasValue ? 'configured' : 'not set';
                nameDiv.append(valBadge);
                if (def.required !== false && !hasValue) {
                    var reqBadge = document.createElement('span');
                    reqBadge.className = 'badge badge-yellow';
                    reqBadge.textContent = 'required';
                    nameDiv.append(document.createTextNode(' '), reqBadge);
                }
                if (def.secret) {
                    var secBadge = document.createElement('span');
                    secBadge.className = 'badge badge-gray';
                    secBadge.textContent = 'secret';
                    nameDiv.append(document.createTextNode(' '), secBadge);
                }
                var descDiv = document.createElement('div');
                descDiv.className = 'detail-item-desc';
                descDiv.style.cssText = 'font-size:var(--sp-text-sm);color:var(--sp-text-secondary);margin-top:var(--sp-space-1)';
                if (def.description) descDiv.textContent = def.description;
                if (hasValue) {
                    var maskedSpan = document.createElement('span');
                    maskedSpan.style.cssText = 'font-family:monospace;color:var(--sp-text-tertiary)';
                    maskedSpan.textContent = s.is_secret ? '--------' : s.var_value;
                    descDiv.append(document.createTextNode(' '), maskedSpan);
                }
                info.append(nameDiv, descDiv);
                item.append(info);
                frag.append(item);
            });
            var btnWrap = document.createElement('div');
            btnWrap.style.cssText = 'padding:var(--sp-space-3) 0';
            var configBtn = document.createElement('button');
            configBtn.className = 'btn btn-primary btn-sm';
            configBtn.setAttribute('data-open-env', pluginId);
            configBtn.setAttribute('data-plugin-name', pluginId);
            configBtn.textContent = 'Configure';
            btnWrap.append(configBtn);
            frag.append(btnWrap);
            container.replaceChildren(frag);
        }).catch(function() {
            container.replaceChildren(createEmptyState('Failed to load environment variables.'));
        });
    }
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
        var frag = document.createDocumentFragment();
        if (!vars.length) {
            var empty = document.createElement('div');
            empty.className = 'empty-state';
            empty.style.padding = 'var(--sp-space-6)';
            var emptyP = document.createElement('p');
            emptyP.textContent = 'No environment variables defined for this plugin.';
            empty.append(emptyP);
            frag.append(empty);
            return frag;
        }
        vars.forEach(function(v, i) {
            var group = document.createElement('div');
            group.className = 'form-group';
            var label = document.createElement('label');
            label.textContent = v.name;
            if (v.required) {
                var reqBadge = document.createElement('span');
                reqBadge.className = 'badge badge-red';
                reqBadge.textContent = 'required';
                label.append(document.createTextNode(' '), reqBadge);
            }
            if (v.secret) {
                var secBadge = document.createElement('span');
                secBadge.className = 'badge badge-gray';
                secBadge.textContent = 'secret';
                label.append(document.createTextNode(' '), secBadge);
            }
            group.append(label);
            if (v.description) {
                var desc = document.createElement('p');
                desc.style.cssText = 'margin:0 0 var(--sp-space-1);font-size:var(--sp-text-xs);color:var(--sp-text-tertiary)';
                desc.textContent = v.description;
                group.append(desc);
            }
            var input = document.createElement('input');
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

    function renderModal(vars) {
        var frag = document.createDocumentFragment();
        var h3 = document.createElement('h3');
        h3.style.cssText = 'margin:0 0 var(--sp-space-4)';
        h3.textContent = currentPluginName + ' \u2014 Environment Variables';
        var scrollDiv = document.createElement('div');
        scrollDiv.style.cssText = 'max-height:60vh;overflow-y:auto';
        scrollDiv.append(renderVarList(vars));
        var actions = document.createElement('div');
        actions.className = 'form-actions';
        actions.style.cssText = 'display:flex;gap:var(--sp-space-3);justify-content:flex-end;margin-top:var(--sp-space-4)';
        var closeBtn = document.createElement('button');
        closeBtn.className = 'btn btn-secondary';
        closeBtn.id = 'plugin-env-close';
        closeBtn.textContent = 'Close';
        var saveBtn = document.createElement('button');
        saveBtn.className = 'btn btn-primary';
        saveBtn.id = 'plugin-env-save';
        saveBtn.textContent = 'Save';
        actions.append(closeBtn, saveBtn);
        frag.append(h3, scrollDiv, actions);
        return frag;
    }

    function updatePanel(vars) {
        var panel = overlay && overlay.querySelector('.confirm-dialog');
        if (panel) panel.replaceChildren(renderModal(vars));
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
                saveBtn.style.background = 'var(--sp-success)';
                saveBtn.style.borderColor = 'var(--sp-success)';
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
        var envDialog = document.createElement('div');
        envDialog.className = 'confirm-dialog';
        envDialog.style.cssText = 'width:560px;max-width:90vw';
        var envLoading = document.createElement('div');
        envLoading.style.cssText = 'display:flex;align-items:center;justify-content:center;padding:var(--sp-space-6);color:var(--sp-text-tertiary)';
        envLoading.textContent = 'Loading...';
        envDialog.append(envLoading);
        overlay.append(envDialog);
        document.body.append(overlay);

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
        var frag = document.createDocumentFragment();
        if (!files.length) {
            var empty = document.createElement('div');
            empty.className = 'empty-state';
            empty.style.padding = 'var(--sp-space-6)';
            var p1 = document.createElement('p');
            p1.textContent = 'No files found for this skill.';
            var p2 = document.createElement('p');
            p2.style.cssText = 'font-size:var(--sp-text-sm);color:var(--sp-text-tertiary);margin-top:var(--sp-space-2)';
            p2.textContent = 'Click "Sync Files" to scan the filesystem.';
            empty.append(p1, p2);
            frag.append(empty);
            return frag;
        }
        var groups = groupByCategory(files);
        categoryOrder.forEach(function(cat) {
            var group = groups[cat];
            if (!group || !group.length) return;
            var wrapper = document.createElement('div');
            wrapper.style.marginBottom = 'var(--sp-space-3)';
            var catDiv = document.createElement('div');
            catDiv.className = 'skill-file-category';
            catDiv.textContent = (categoryLabels[cat] || cat) + ' (' + group.length + ')';
            wrapper.append(catDiv);
            group.forEach(function(f) {
                var item = document.createElement('div');
                item.className = 'skill-file-item' + (selectedFile && selectedFile.id === f.id ? ' selected' : '');
                item.setAttribute('data-file-id', f.id);
                var nameSpan = document.createElement('span');
                nameSpan.className = 'skill-file-name';
                nameSpan.textContent = f.file_path;
                item.append(nameSpan);
                if (f.language) {
                    var langSpan = document.createElement('span');
                    langSpan.className = 'skill-file-lang';
                    langSpan.textContent = f.language;
                    item.append(langSpan);
                }
                wrapper.append(item);
            });
            frag.append(wrapper);
        });
        return frag;
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
        var frag = document.createDocumentFragment();
        if (!selectedFile) {
            var placeholder = document.createElement('div');
            placeholder.style.cssText = 'display:flex;align-items:center;justify-content:center;height:100%;color:var(--sp-text-tertiary);font-size:var(--sp-text-sm)';
            placeholder.textContent = 'Select a file to view its contents';
            frag.append(placeholder);
            return frag;
        }
        var wrapper = document.createElement('div');
        wrapper.style.cssText = 'display:flex;flex-direction:column;height:100%';

        var header = document.createElement('div');
        header.style.cssText = 'display:flex;align-items:center;gap:var(--sp-space-2);padding:var(--sp-space-2) var(--sp-space-3);border-bottom:1px solid var(--sp-border-subtle);flex-shrink:0';
        var pathSpan = document.createElement('span');
        pathSpan.style.cssText = 'font-family:monospace;font-size:var(--sp-text-sm);font-weight:600';
        pathSpan.textContent = selectedFile.file_path;
        var langBadge = document.createElement('span');
        langBadge.className = 'badge badge-blue';
        langBadge.style.fontSize = 'var(--sp-text-xs)';
        langBadge.textContent = selectedFile.language || 'text';
        header.append(pathSpan, langBadge);
        if (selectedFile.executable) {
            var execBadge = document.createElement('span');
            execBadge.className = 'badge badge-green';
            execBadge.style.fontSize = 'var(--sp-text-xs)';
            execBadge.textContent = 'executable';
            header.append(execBadge);
        }
        var sizeSpan = document.createElement('span');
        sizeSpan.style.cssText = 'margin-left:auto;font-size:var(--sp-text-xs);color:var(--sp-text-tertiary)';
        sizeSpan.textContent = selectedFile.size_bytes + ' bytes';
        header.append(sizeSpan);

        var textarea = document.createElement('textarea');
        textarea.id = 'skill-file-editor';
        textarea.style.cssText = 'flex:1;width:100%;border:none;padding:var(--sp-space-3);font-family:monospace;font-size:var(--sp-text-sm);line-height:1.5;resize:none;background:var(--sp-bg-surface);color:var(--sp-text-primary);outline:none;box-sizing:border-box';
        textarea.value = selectedFile.content || '';

        var footer = document.createElement('div');
        footer.style.cssText = 'display:flex;align-items:center;padding:var(--sp-space-2) var(--sp-space-3);border-top:1px solid var(--sp-border-subtle);flex-shrink:0';
        var validationSpan = document.createElement('span');
        validationSpan.id = 'skill-file-validation';
        validationSpan.style.cssText = 'font-size:var(--sp-text-xs);flex:1';
        var saveBtn = document.createElement('button');
        saveBtn.className = 'btn btn-primary btn-sm';
        saveBtn.id = 'skill-file-save';
        saveBtn.style.fontSize = 'var(--sp-text-xs)';
        saveBtn.textContent = 'Save';
        footer.append(validationSpan, saveBtn);

        wrapper.append(header, textarea, footer);
        frag.append(wrapper);
        return frag;
    }

    function renderModal() {
        var outer = document.createElement('div');
        outer.style.cssText = 'display:flex;flex-direction:column;height:100%';

        var topBar = document.createElement('div');
        topBar.style.cssText = 'display:flex;align-items:center;padding:var(--sp-space-4);border-bottom:1px solid var(--sp-border-subtle);flex-shrink:0';
        var h2 = document.createElement('h2');
        h2.style.cssText = 'margin:0;font-size:var(--sp-text-lg);font-weight:600;color:var(--sp-text-primary)';
        h2.textContent = currentSkillName + ' - Files';
        var btnGroup = document.createElement('div');
        btnGroup.style.cssText = 'margin-left:auto;display:flex;gap:var(--sp-space-2)';
        var syncBtn = document.createElement('button');
        syncBtn.className = 'btn btn-secondary btn-sm';
        syncBtn.id = 'skill-files-sync';
        syncBtn.style.fontSize = 'var(--sp-text-xs)';
        syncBtn.textContent = 'Sync Files';
        var closeBtn = document.createElement('button');
        closeBtn.className = 'btn btn-secondary btn-sm';
        closeBtn.id = 'skill-files-close';
        closeBtn.style.fontSize = 'var(--sp-text-xs)';
        closeBtn.textContent = 'Close';
        btnGroup.append(syncBtn, closeBtn);
        topBar.append(h2, btnGroup);

        var body = document.createElement('div');
        body.style.cssText = 'display:flex;flex:1;min-height:0';
        var listDiv = document.createElement('div');
        listDiv.id = 'skill-files-list';
        listDiv.style.cssText = 'width:280px;overflow-y:auto;border-right:1px solid var(--sp-border-subtle);padding:var(--sp-space-2) 0';
        listDiv.append(renderFileList());
        var editorDiv = document.createElement('div');
        editorDiv.id = 'skill-files-editor';
        editorDiv.style.cssText = 'flex:1;min-width:0;overflow:hidden';
        editorDiv.append(renderEditor());
        body.append(listDiv, editorDiv);

        outer.append(topBar, body);
        return outer;
    }

    function updatePanel() {
        var panel = overlay && overlay.querySelector('.skill-files-panel');
        if (panel) panel.replaceChildren(renderModal());
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
            badge.style.color = 'var(--sp-danger)';
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
        var listEl = overlay.querySelector('#skill-files-list');
        var editorEl = overlay.querySelector('#skill-files-editor');
        if (listEl) listEl.replaceChildren(renderFileList());
        if (editorEl) editorEl.replaceChildren(renderEditor());
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
        var sfPanel = document.createElement('div');
        sfPanel.className = 'skill-files-panel';
        sfPanel.style.cssText = 'background:var(--sp-bg-surface);border-radius:var(--sp-radius-lg);width:90vw;max-width:1100px;height:80vh;overflow:hidden;box-shadow:var(--sp-shadow-lg);display:flex;flex-direction:column';
        var sfLoading = document.createElement('div');
        sfLoading.style.cssText = 'display:flex;align-items:center;justify-content:center;height:100%;color:var(--sp-text-tertiary)';
        sfLoading.textContent = 'Loading files...';
        sfPanel.append(sfLoading);
        overlay.append(sfPanel);
        document.body.append(overlay);

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
        var labels = ['Basic Info', 'Skills', 'Agents', 'MCP Servers', 'Hooks', 'Roles & Access', 'Review'];
        var container = document.getElementById('wizard-step-indicator');
        if (!container) return;
        var steps = document.createElement('div');
        steps.className = 'wizard-steps';
        steps.style.cssText = 'display:flex;gap:var(--sp-space-1);margin-bottom:var(--sp-space-6);flex-wrap:wrap';
        for (var i = 1; i <= TOTAL_STEPS; i++) {
            var isActive = i === state.step;
            var isDone = i < state.step;
            var bgColor = isActive ? 'var(--sp-accent)' : (isDone ? 'var(--sp-success)' : 'var(--sp-bg-tertiary)');
            var textColor = (isActive || isDone) ? '#fff' : 'var(--sp-text-tertiary)';
            var stepDiv = document.createElement('div');
            stepDiv.style.cssText = 'display:flex;align-items:center;gap:var(--sp-space-2);padding:var(--sp-space-2) var(--sp-space-3);border-radius:var(--sp-radius-md);font-size:var(--sp-text-sm)';
            stepDiv.style.background = bgColor;
            stepDiv.style.color = textColor;
            stepDiv.style.fontWeight = isActive ? '600' : '400';
            var numSpan = document.createElement('span');
            numSpan.style.cssText = 'width:20px;height:20px;border-radius:50%;background:rgba(255,255,255,0.2);display:inline-flex;align-items:center;justify-content:center;font-size:var(--sp-text-xs)';
            numSpan.textContent = i;
            var labelSpan = document.createElement('span');
            labelSpan.textContent = labels[i - 1];
            stepDiv.append(numSpan, labelSpan);
            steps.append(stepDiv);
        }
        container.replaceChildren(steps);
    }

    function renderNav() {
        var nav = document.getElementById('wizard-nav');
        if (!nav) return;
        var wrapper = document.createElement('div');
        wrapper.style.cssText = 'display:flex;gap:var(--sp-space-3);margin-top:var(--sp-space-6)';
        if (state.step > 1) {
            var prevBtn = document.createElement('button');
            prevBtn.type = 'button';
            prevBtn.className = 'btn btn-secondary';
            prevBtn.id = 'wizard-prev';
            prevBtn.textContent = 'Previous';
            wrapper.append(prevBtn);
        }
        if (state.step < TOTAL_STEPS) {
            var nextBtn = document.createElement('button');
            nextBtn.type = 'button';
            nextBtn.className = 'btn btn-primary';
            nextBtn.id = 'wizard-next';
            nextBtn.textContent = 'Next';
            wrapper.append(nextBtn);
        }
        if (state.step === TOTAL_STEPS) {
            var createBtn = document.createElement('button');
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
        list.replaceChildren();
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
            list.append(frag);
        });
    }

    function renderReview() {
        var el = document.getElementById('wizard-review');
        if (!el) return;
        var f = state.form;
        var selectedSkills = Object.keys(state.selectedSkills).filter(function(k) { return state.selectedSkills[k]; });
        var selectedAgents = Object.keys(state.selectedAgents).filter(function(k) { return state.selectedAgents[k]; });
        var selectedMcp = Object.keys(state.selectedMcpServers).filter(function(k) { return state.selectedMcpServers[k]; });
        var selectedRoles = Object.keys(f.roles).filter(function(k) { return f.roles[k]; });

        function buildBadgeList(items, emptyMsg) {
            var container = document.createElement('div');
            container.style.cssText = 'display:flex;flex-wrap:wrap';
            if (!items.length) {
                var empty = document.createElement('span');
                empty.style.color = 'var(--sp-text-tertiary)';
                empty.textContent = emptyMsg;
                container.append(empty);
            } else {
                items.forEach(function(item) {
                    var badge = document.createElement('span');
                    badge.className = 'badge badge-blue';
                    badge.style.margin = 'var(--sp-space-1)';
                    badge.textContent = item;
                    container.append(badge);
                });
            }
            return container;
        }

        function addRow(frag, labelText, valueText) {
            var strong = document.createElement('strong');
            strong.textContent = labelText;
            var span = document.createElement('span');
            span.textContent = valueText;
            frag.append(strong, span);
        }

        var frag = document.createDocumentFragment();
        addRow(frag, 'Plugin ID:', f.plugin_id || '-');
        addRow(frag, 'Name:', f.name || '-');
        addRow(frag, 'Description:', f.description || '-');
        addRow(frag, 'Version:', f.version || '0.1.0');
        addRow(frag, 'Category:', f.category || '-');
        addRow(frag, 'Author:', f.author_name || '-');
        addRow(frag, 'Keywords:', f.keywords || '-');

        var rolesLabel = document.createElement('strong');
        rolesLabel.textContent = 'Roles:';
        frag.append(rolesLabel, buildBadgeList(selectedRoles, 'None selected'));

        var skillsLabel = document.createElement('strong');
        skillsLabel.textContent = 'Skills (' + selectedSkills.length + '):';
        frag.append(skillsLabel, buildBadgeList(selectedSkills, 'None selected'));

        var agentsLabel = document.createElement('strong');
        agentsLabel.textContent = 'Agents (' + selectedAgents.length + '):';
        frag.append(agentsLabel, buildBadgeList(selectedAgents, 'None selected'));

        var mcpLabel = document.createElement('strong');
        mcpLabel.textContent = 'MCP (' + selectedMcp.length + '):';
        frag.append(mcpLabel, buildBadgeList(selectedMcp, 'None selected'));

        var hooksLabel = document.createElement('strong');
        hooksLabel.textContent = 'Hooks (' + state.hooks.length + '):';
        var hooksSpan = document.createElement('span');
        hooksSpan.textContent = state.hooks.length > 0
            ? state.hooks.map(function(h) { return h.event + ': ' + (h.command || '?'); }).join(', ')
            : 'None';
        frag.append(hooksLabel, hooksSpan);

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
        if (icon) icon.textContent = open ? '\u25b6' : '\u25bc';
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

    window.addEventListener('env-saved', function(e) {
        const pid = e.detail && e.detail.pluginId;
        if (!pid) return;
        const containerId = 'env-status-' + pid;
        const container = document.getElementById(containerId);
        if (container) {
            container.removeAttribute('data-loaded');
            var refreshDiv = document.createElement('div');
            refreshDiv.style.cssText = 'padding:var(--sp-space-4);color:var(--sp-text-tertiary);font-size:var(--sp-text-sm)';
            refreshDiv.textContent = 'Refreshing...';
            container.replaceChildren(refreshDiv);
        }
    });
    app.pluginDetails = { render: function() { return ''; } };
})(window.AdminApp);
