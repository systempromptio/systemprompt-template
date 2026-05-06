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
