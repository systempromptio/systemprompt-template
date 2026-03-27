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
                        container.innerHTML = OrgCommon.formatJson(data);
                    } catch (err) {
                        container.innerHTML = '<span class="text-muted">Failed to parse JSON</span>';
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
