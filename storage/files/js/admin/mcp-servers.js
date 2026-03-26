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
                rows.forEach(function(row) {
                    const name = (row.getAttribute('data-name') || '').toLowerCase();
                    const match = !query || name.indexOf(query) !== -1;
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

        document.addEventListener('click', function(e) {
            const toggle = e.target.closest('[data-mcp-json]');
            if (!toggle) return;
            const mcpId = toggle.getAttribute('data-mcp-json');
            const container = document.querySelector('[data-mcp-json-container="' + mcpId + '"]');
            if (!container) return;

            if (container.style.display === 'none') {
                const script = document.querySelector('script[data-mcp-detail="' + mcpId + '"]');
                if (script) {
                    try {
                        const data = JSON.parse(script.textContent);
                        container.innerHTML = OrgCommon.formatJson(data);
                    } catch (err) {
                        container.innerHTML = '<span class="text-muted">Failed to parse JSON</span>';
                    }
                }
                container.style.display = 'block';
                toggle.textContent = 'Hide JSON';
            } else {
                container.style.display = 'none';
                toggle.textContent = 'Show JSON';
            }
        });

        document.addEventListener('click', function(e) {
            const deleteBtn = e.target.closest('[data-action="delete"][data-entity-type="mcp-server"]');
            if (!deleteBtn) return;
            const id = deleteBtn.getAttribute('data-entity-id');
            if (!confirm('Delete MCP server "' + id + '"? This cannot be undone.')) return;

            app.api('/mcp-servers/' + encodeURIComponent(id), {
                method: 'DELETE'
            }).then(function() {
                app.Toast.show('MCP server deleted', 'success');
                const row = document.querySelector('tr[data-entity-id="' + id + '"].clickable-row');
                if (row) {
                    const detail = row.nextElementSibling;
                    if (detail && detail.classList.contains('detail-row')) {
                        detail.remove();
                    }
                    row.remove();
                }
            }).catch(function(err) {
                app.Toast.show(err.message || 'Failed to delete MCP server', 'error');
            });
        });

        let allPlugins = [];
        document.querySelectorAll('script[data-mcp-detail]').forEach(function(script) {
            try {
                const data = JSON.parse(script.textContent);
                if (data.assigned_plugins) {
                    data.assigned_plugins.forEach(function(p) {
                        if (!allPlugins.some(function(existing) { return existing.id === p.id; })) {
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

        document.addEventListener('click', function(e) {
            const assignBtn = e.target.closest('[data-assign-mcp]');
            if (!assignBtn) return;
            const mcpId = assignBtn.getAttribute('data-assign-mcp');
            const mcpName = assignBtn.getAttribute('data-mcp-name') || mcpId;

            let currentPluginIds = [];
            const script = document.querySelector('script[data-mcp-detail="' + mcpId + '"]');
            if (script) {
                try {
                    const data = JSON.parse(script.textContent);
                    if (data.assigned_plugins) {
                        currentPluginIds = data.assigned_plugins.map(function(p) { return p.id; });
                    }
                } catch (e) {}
            }

            if (assignPanel) {
                assignPanel.open(mcpId, mcpName, currentPluginIds);
            }
        });
    };

})(window.AdminApp = window.AdminApp || {});
