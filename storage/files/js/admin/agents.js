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
            html += '<pre style="margin:var(--space-1) 0;max-height:200px;overflow:auto;font-size:var(--text-xs);background:var(--bg-surface-raised);padding:var(--space-2);border-radius:var(--radius-sm);white-space:pre-wrap;word-break:break-word">' + app.escapeHtml(data.system_prompt) + '</pre>';
            html += '</div>';
        }

        if (data.port || data.endpoint) {
            html += '<div class="detail-section">';
            html += '<strong>Connection</strong>';
            html += '<div style="margin:var(--space-1) 0;font-size:var(--text-sm);color:var(--text-secondary)">';
            if (data.port) html += '<div>Port: <code class="code-inline">' + app.escapeHtml(String(data.port)) + '</code></div>';
            if (data.endpoint) html += '<div>Endpoint: <code class="code-inline">' + app.escapeHtml(data.endpoint) + '</code></div>';
            html += '</div></div>';
        }

        if ((data.skill_count && data.skill_count > 0) || (data.mcp_count && data.mcp_count > 0)) {
            html += '<div class="detail-section">';
            html += '<strong>Capabilities</strong>';
            html += '<div class="badge-row" style="margin-top:var(--space-1)">';
            if (data.skill_count > 0) {
                html += '<span class="badge badge-green">' + data.skill_count + ' skill' + (data.skill_count !== 1 ? 's' : '') + '</span>';
            }
            if (data.mcp_count > 0) {
                html += '<span class="badge badge-yellow">' + data.mcp_count + ' MCP server' + (data.mcp_count !== 1 ? 's' : '') + '</span>';
            }
            html += '</div></div>';
        }

        html += '<div class="detail-section">';
        html += '<details><summary style="cursor:pointer;font-size:var(--text-sm);color:var(--text-secondary)">JSON Config</summary>';
        html += app.OrgCommon.formatJson(data);
        html += '</details></div>';

        return html;
    }

    function initExpandRows() {
        app.OrgCommon.initExpandRows('.data-table', function(row, detailRow) {
            var content = detailRow.querySelector('[data-agent-expand]');
            if (content && !content.hasAttribute('data-loaded')) {
                var agentId = content.getAttribute('data-agent-expand');
                content.innerHTML = renderAgentExpand(agentId);
                content.setAttribute('data-loaded', 'true');
            }
        });
    }

    function initDeleteHandlers() {
        document.addEventListener('click', function(e) {
            const btn = e.target.closest('[data-delete-agent]');
            if (!btn) return;
            const agentId = btn.getAttribute('data-delete-agent');
            if (!confirm('Are you sure you want to delete agent "' + agentId + '"? This cannot be undone.')) return;

            fetch('/api/admin/agents/' + encodeURIComponent(agentId), { method: 'DELETE' })
                .then(function(res) {
                    if (res.ok) {
                        app.Toast.show('Agent deleted', 'success');
                        setTimeout(function() { window.location.reload(); }, 500);
                    } else {
                        app.Toast.show('Failed to delete agent', 'error');
                    }
                })
                .catch(function() {
                    app.Toast.show('Failed to delete agent', 'error');
                });
        });
    }

    function initForkHandlers() {
        document.addEventListener('click', function(e) {
            const btn = e.target.closest('[data-fork-agent]');
            if (!btn) return;
            const agentId = btn.getAttribute('data-fork-agent');
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
            .then(function(res) {
                if (res.ok) {
                    app.Toast.show('Agent customized', 'success');
                    setTimeout(function() { window.location.reload(); }, 500);
                } else {
                    app.Toast.show('Failed to customize agent', 'error');
                }
            })
            .catch(function() {
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

        document.addEventListener('click', function(e) {
            const btn = e.target.closest('[data-assign-agent]');
            if (!btn) return;
            const agentId = btn.getAttribute('data-assign-agent');
            const agentName = btn.getAttribute('data-agent-name') || agentId;
            const data = getAgentDetail(agentId);
            const currentPluginIds = data && data.assigned_plugin_ids ? data.assigned_plugin_ids : [];
            assignApi.open(agentId, agentName, currentPluginIds);
        });

        document.addEventListener('click', function(e) {
            const btn = e.target.closest('[data-assign-save]');
            if (!btn) return;
            const entityId = btn.getAttribute('data-entity-id');
            const checkboxes = document.querySelectorAll('#assign-panel input[name="plugin_id"]');
            const selectedPlugins = [];
            checkboxes.forEach(function(cb) {
                if (cb.checked) selectedPlugins.push(cb.value);
            });

            btn.disabled = true;
            btn.textContent = 'Saving...';

            const promises = allPlugins.map(function(plugin) {
                return fetch('/api/admin/plugins/' + encodeURIComponent(plugin.id) + '/agents')
                    .then(function(res) { return res.json(); })
                    .then(function(currentAgents) {
                        let agentIds = (currentAgents || []).slice();
                        const shouldInclude = selectedPlugins.indexOf(plugin.id) !== -1;
                        const hasAgent = agentIds.indexOf(entityId) !== -1;

                        if (shouldInclude && !hasAgent) {
                            agentIds.push(entityId);
                        } else if (!shouldInclude && hasAgent) {
                            agentIds = agentIds.filter(function(a) { return a !== entityId; });
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
                .then(function() {
                    app.Toast.show('Plugin assignments updated', 'success');
                    assignApi.close();
                    setTimeout(function() { window.location.reload(); }, 500);
                })
                .catch(function() {
                    app.Toast.show('Failed to update assignments', 'error');
                    btn.disabled = false;
                    btn.textContent = 'Save';
                });
        });
    }

    function initEditPanel() {
        var editPanel = app.OrgCommon.initEditPanel({
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

        document.addEventListener('click', function(e) {
            var btn = e.target.closest('[data-edit-agent]');
            if (!btn) return;
            var agentId = btn.getAttribute('data-edit-agent');
            var data = getAgentDetail(agentId);
            if (data && editPanel) editPanel.open(agentId, data);
        });
    }

    function initBulkHandlers() {
        var bulk = app.OrgCommon.initBulkActions('.data-table', 'bulk-bar');
        if (!bulk) return;

        var allPlugins = getAllPlugins();
        var assignApi = app.OrgCommon.initAssignPanel({
            panelId: 'assign-panel',
            allPlugins: allPlugins
        });

        // Bulk delete
        var deleteBtn = document.getElementById('bulk-delete-btn');
        if (deleteBtn) {
            deleteBtn.addEventListener('click', function() {
                var ids = bulk.getSelected();
                if (!ids.length) return;
                if (!confirm('Delete ' + ids.length + ' agent(s)? This action cannot be undone.')) return;
                Promise.all(ids.map(function(id) {
                    return fetch('/api/admin/agents/' + encodeURIComponent(id), { method: 'DELETE' });
                })).then(function() {
                    app.Toast.show(ids.length + ' agents deleted', 'success');
                    setTimeout(function() { window.location.reload(); }, 500);
                }).catch(function() {
                    app.Toast.show('Failed to delete some agents', 'error');
                });
            });
        }

        // Bulk assign to plugin
        var assignBtn = document.getElementById('bulk-assign-btn');
        if (assignBtn && assignApi) {
            assignBtn.addEventListener('click', function() {
                var ids = bulk.getSelected();
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
