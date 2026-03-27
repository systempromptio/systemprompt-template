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
        if (!data) return '<p class="text-muted">No detail data available.</p>';

        let html = '';

        if (data.system_prompt) {
            html += '<div class="detail-section">';
            html += '<strong>System Prompt</strong>';
            html += '<pre style="margin:var(--sp-space-1) 0;max-height:200px;overflow:auto;font-size:var(--sp-text-xs);background:var(--sp-bg-surface-raised);padding:var(--sp-space-2);border-radius:var(--sp-radius-sm);white-space:pre-wrap;word-break:break-word">' + app.escapeHtml(data.system_prompt) + '</pre>';
            html += '</div>';
        }

        if (data.port || data.endpoint) {
            html += '<div class="detail-section">';
            html += '<strong>Connection</strong>';
            html += '<div style="margin:var(--sp-space-1) 0;font-size:var(--sp-text-sm);color:var(--sp-text-secondary)">';
            if (data.port) html += '<div>Port: <code class="code-inline">' + app.escapeHtml(String(data.port)) + '</code></div>';
            if (data.endpoint) html += '<div>Endpoint: <code class="code-inline">' + app.escapeHtml(data.endpoint) + '</code></div>';
            html += '</div></div>';
        }

        if ((data.skill_count && data.skill_count > 0) || (data.mcp_count && data.mcp_count > 0)) {
            html += '<div class="detail-section">';
            html += '<strong>Capabilities</strong>';
            html += '<div class="badge-row" style="margin-top:var(--sp-space-1)">';
            if (data.skill_count > 0) {
                html += '<span class="badge badge-green">' + data.skill_count + ' skill' + (data.skill_count !== 1 ? 's' : '') + '</span>';
            }
            if (data.mcp_count > 0) {
                html += '<span class="badge badge-yellow">' + data.mcp_count + ' MCP server' + (data.mcp_count !== 1 ? 's' : '') + '</span>';
            }
            html += '</div></div>';
        }

        html += '<div class="detail-section">';
        html += '<details><summary style="cursor:pointer;font-size:var(--sp-text-sm);color:var(--sp-text-secondary)">JSON Config</summary>';
        html += app.OrgCommon.formatJson(data);
        html += '</details></div>';

        return html;
    }

    function initExpandRows() {
        app.OrgCommon.initExpandRows('.data-table', (row, detailRow) => {
            const content = detailRow.querySelector('[data-agent-expand]');
            if (content && !content.hasAttribute('data-loaded')) {
                const agentId = content.getAttribute('data-agent-expand');
                content.innerHTML = renderAgentExpand(agentId);
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
