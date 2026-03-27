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
                setBody: function(html) {
                    const body = panel.querySelector('[data-panel-body]');
                    if (body) body.innerHTML = html;
                },
                setFooter: function(html) {
                    const footer = panel.querySelector('[data-panel-footer]');
                    if (footer) footer.innerHTML = html;
                },
                panel: panel
            };

            if (closeBtn) closeBtn.addEventListener('click', api.close);
            if (overlay) overlay.addEventListener('click', api.close);

            return api;
        },

        initAssignPanel: function(config) {
            const panelApi = OrgCommon.initSidePanel(config.panelId);
            if (!panelApi) return null;

            return {
                open: function(entityId, entityName, currentPluginIds) {
                    panelApi.setTitle('Assign ' + (entityName || entityId));

                    const allPlugins = config.allPlugins || [];
                    const currentSet = {};
                    (currentPluginIds || []).forEach(function(id) { currentSet[id] = true; });

                    let html = '<div class="assign-panel-checklist">';
                    if (allPlugins.length === 0) {
                        html += '<p style="color:var(--text-tertiary);font-size:var(--text-sm)">No plugins available.</p>';
                    } else {
                        allPlugins.forEach(function(p) {
                            const checked = currentSet[p.id] ? ' checked' : '';
                            html += '<label class="acl-checkbox-row">' +
                                '<input type="checkbox" name="plugin_id" value="' + app.escapeHtml(p.id) + '"' + checked + '>' +
                                '<span class="acl-checkbox-label">' + app.escapeHtml(p.name || p.id) + '</span>' +
                                '</label>';
                        });
                    }
                    html += '</div>';
                    panelApi.setBody(html);

                    panelApi.setFooter(
                        '<button class="btn btn-secondary" data-panel-close>Cancel</button> ' +
                        '<button class="btn btn-primary" data-assign-save data-entity-id="' + app.escapeHtml(entityId) + '">Save</button>'
                    );

                    const footer = panelApi.panel.querySelector('[data-panel-footer]');
                    if (footer) {
                        const cancelBtn = footer.querySelector('[data-panel-close]');
                        if (cancelBtn) cancelBtn.addEventListener('click', panelApi.close);
                    }

                    panelApi.open();
                },
                close: panelApi.close,
                panel: panelApi
            };
        },

        initEditPanel: function(config) {
            const panelApi = OrgCommon.initSidePanel(config.panelId);
            if (!panelApi) return null;
            let currentEntityId = null;

            function buildForm(entityData) {
                let html = '<form class="edit-panel-form">';
                (config.fields || []).forEach(function(f) {
                    let val = entityData[f.name] || '';
                    if (Array.isArray(val)) val = val.join(', ');
                    html += '<div class="form-group">';
                    html += '<label class="form-label">' + app.escapeHtml(f.label) + '</label>';
                    if (f.type === 'textarea') {
                        html += '<textarea class="form-control" name="' + f.name + '" rows="' + (f.rows || 10) + '">' + app.escapeHtml(val) + '</textarea>';
                    } else {
                        html += '<input type="text" class="form-control" name="' + f.name + '" value="' + app.escapeHtml(val) + '"' + (f.required ? ' required' : '') + '>';
                    }
                    html += '</div>';
                });
                html += '</form>';
                return html;
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
                    panelApi.setTitle('Edit ' + app.escapeHtml(entityData.name || entityId));
                    panelApi.setBody(buildForm(entityData));
                    panelApi.setFooter(
                        '<button class="btn btn-secondary" data-panel-close>Cancel</button> ' +
                        '<button class="btn btn-primary" data-edit-save>Save</button>'
                    );
                    const footer = panelApi.panel.querySelector('[data-panel-footer]');
                    if (footer) {
                        const cancelBtn = footer.querySelector('[data-panel-close]');
                        if (cancelBtn) cancelBtn.addEventListener('click', panelApi.close);
                    }
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
                try { data = JSON.parse(data); } catch (e) { return app.escapeHtml(data); }
            }
            return '<pre class="json-view">' + app.escapeHtml(JSON.stringify(data, null, 2)) + '</pre>';
        },

        renderRoleBadges: function(roles) {
            if (!roles || !roles.length) {
                return '<span class="badge badge-gray">All</span>';
            }
            const assigned = roles.filter(function(r) { return r.assigned; });
            if (!assigned.length) {
                return '<span class="badge badge-gray">All</span>';
            }
            return assigned.map(function(r) {
                return '<span class="badge badge-blue">' + app.escapeHtml(r.name) + '</span>';
            }).join(' ');
        },

        renderDeptBadges: function(departments) {
            if (!departments || !departments.length) {
                return '<span class="badge badge-gray">None</span>';
            }
            const assigned = departments.filter(function(d) { return d.assigned; });
            if (!assigned.length) {
                return '<span class="badge badge-gray">None</span>';
            }
            return assigned.map(function(d) {
                const cls = d.default_included ? 'badge-yellow' : 'badge-green';
                return '<span class="badge ' + cls + '">' + app.escapeHtml(d.name) + '</span>';
            }).join(' ');
        },

        renderPluginBadges: function(plugins) {
            if (!plugins || !plugins.length) {
                return '<span class="badge badge-gray">None</span>';
            }
            return plugins.map(function(p) {
                const name = typeof p === 'string' ? p : (p.name || p.id || p);
                return '<span class="badge badge-purple">' + app.escapeHtml(name) + '</span>';
            }).join(' ');
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
            const content = detailRow.querySelector('[data-agent-expand]');
            if (content && !content.hasAttribute('data-loaded')) {
                const agentId = content.getAttribute('data-agent-expand');
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
                        const shouldInclude = selectedPlugins.includes(plugin.id);
                        const hasAgent = agentIds.includes(entityId);

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

        document.addEventListener('click', function(e) {
            const btn = e.target.closest('[data-edit-agent]');
            if (!btn) return;
            const agentId = btn.getAttribute('data-edit-agent');
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
            deleteBtn.addEventListener('click', function() {
                const ids = bulk.getSelected();
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

        const assignBtn = document.getElementById('bulk-assign-btn');
        if (assignBtn && assignApi) {
            assignBtn.addEventListener('click', function() {
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
        if (!data) return '<p class="text-muted">No detail data available.</p>';

        let html = '<div class="detail-section">';
        html += '<strong>Description</strong>';
        html += '<p style="margin:var(--space-1) 0;color:var(--text-secondary);font-size:var(--text-sm)">' + app.escapeHtml(data.description || 'No description') + '</p>';
        html += '</div>';

        if (data.command) {
            html += '<div class="detail-section">';
            html += '<strong>Command</strong>';
            html += '<pre style="margin:var(--space-1) 0;font-size:var(--text-xs);background:var(--bg-surface-raised);padding:var(--space-2);border-radius:var(--radius-sm);overflow-x:auto">' + app.escapeHtml(data.command) + '</pre>';
            html += '</div>';
        }

        if (data.tags && data.tags.length) {
            html += '<div class="detail-section">';
            html += '<strong>Tags</strong><br>';
            html += '<div class="badge-row" style="margin-top:var(--space-1)">';
            data.tags.forEach(function(tag) {
                html += '<span class="badge badge-gray">' + app.escapeHtml(tag) + '</span>';
            });
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
            const content = detailRow.querySelector('[data-skill-expand]');
            if (content && !content.hasAttribute('data-loaded')) {
                const skillId = content.getAttribute('data-skill-expand');
                content.innerHTML = renderSkillExpand(skillId);
                content.setAttribute('data-loaded', 'true');
            }
        });
    }

    function initDeleteHandlers() {
        document.addEventListener('click', function(e) {
            const btn = e.target.closest('[data-delete-skill]');
            if (!btn) return;
            const skillId = btn.getAttribute('data-delete-skill');
            if (!confirm('Are you sure you want to delete skill "' + skillId + '"? This cannot be undone.')) return;

            fetch('/api/admin/skills/' + encodeURIComponent(skillId), { method: 'DELETE' })
                .then(function(res) {
                    if (res.ok) {
                        app.Toast.show('Skill deleted', 'success');
                        setTimeout(function() { window.location.reload(); }, 500);
                    } else {
                        app.Toast.show('Failed to delete skill', 'error');
                    }
                })
                .catch(function() {
                    app.Toast.show('Failed to delete skill', 'error');
                });
        });
    }

    function initForkHandlers() {
        document.addEventListener('click', function(e) {
            const btn = e.target.closest('[data-fork-skill]');
            if (!btn) return;
            const skillId = btn.getAttribute('data-fork-skill');
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
            .then(function(res) {
                if (res.ok) {
                    app.Toast.show('Skill customized', 'success');
                    setTimeout(function() { window.location.reload(); }, 500);
                } else {
                    app.Toast.show('Failed to customize skill', 'error');
                }
            })
            .catch(function() {
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

        document.addEventListener('click', function(e) {
            const btn = e.target.closest('[data-assign-skill]');
            if (!btn) return;
            const skillId = btn.getAttribute('data-assign-skill');
            const skillName = btn.getAttribute('data-skill-name') || skillId;
            const data = getSkillDetail(skillId);
            const currentPluginIds = data && data.assigned_plugin_ids ? data.assigned_plugin_ids : [];
            assignApi.open(skillId, skillName, currentPluginIds);
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
                return fetch('/api/admin/plugins/' + encodeURIComponent(plugin.id) + '/skills')
                    .then(function(res) { return res.json(); })
                    .then(function(currentSkills) {
                        let skillIds = (currentSkills || []).slice();
                        const shouldInclude = selectedPlugins.includes(plugin.id);
                        const hasSkill = skillIds.includes(entityId);

                        if (shouldInclude && !hasSkill) {
                            skillIds.push(entityId);
                        } else if (!shouldInclude && hasSkill) {
                            skillIds = skillIds.filter(function(s) { return s !== entityId; });
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

        document.addEventListener('click', function(e) {
            const btn = e.target.closest('[data-edit-skill]');
            if (!btn) return;
            const skillId = btn.getAttribute('data-edit-skill');
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
            deleteBtn.addEventListener('click', function() {
                const ids = bulk.getSelected();
                if (!ids.length) return;
                if (!confirm('Delete ' + ids.length + ' skill(s)? This action cannot be undone.')) return;
                Promise.all(ids.map(function(id) {
                    return fetch('/api/admin/skills/' + encodeURIComponent(id), { method: 'DELETE' });
                })).then(function() {
                    app.Toast.show(ids.length + ' skills deleted', 'success');
                    setTimeout(function() { window.location.reload(); }, 500);
                }).catch(function() {
                    app.Toast.show('Failed to delete some skills', 'error');
                });
            });
        }

        const assignBtn = document.getElementById('bulk-assign-btn');
        if (assignBtn && assignApi) {
            assignBtn.addEventListener('click', function() {
                const ids = bulk.getSelected();
                if (!ids.length) return;
                assignApi.open(ids.join(','), ids.length + ' skills', []);
            });
        }

        const categoryBtn = document.getElementById('bulk-category-btn');
        if (categoryBtn) {
            categoryBtn.addEventListener('click', function() {
                const ids = bulk.getSelected();
                if (!ids.length) return;
                const category = prompt('Enter category for ' + ids.length + ' skill(s):');
                if (category === null) return;
                Promise.all(ids.map(function(id) {
                    return fetch('/api/public/skills/' + encodeURIComponent(id), {
                        method: 'PUT',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ category_id: category })
                    });
                })).then(function() {
                    app.Toast.show('Category updated for ' + ids.length + ' skills', 'success');
                    setTimeout(function() { window.location.reload(); }, 500);
                }).catch(function() {
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
                rows.forEach(function(row) {
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
                rows.forEach(function(row) {
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

        document.addEventListener('click', function(e) {
            const toggle = e.target.closest('[data-hook-json]');
            if (!toggle) return;
            const hookId = toggle.getAttribute('data-hook-json');
            const container = document.querySelector('[data-hook-json-container="' + hookId + '"]');
            if (!container) return;

            if (container.style.display === 'none') {
                const script = document.querySelector('script[data-hook-detail="' + hookId + '"]');
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
            const deleteBtn = e.target.closest('[data-action="delete"][data-entity-type="hook"]');
            if (!deleteBtn) return;
            const id = deleteBtn.getAttribute('data-entity-id');
            if (!confirm('Delete this hook? This cannot be undone.')) return;

            app.api('/hooks/' + encodeURIComponent(id), {
                method: 'DELETE'
            }).then(function() {
                app.Toast.show('Hook deleted', 'success');
                const row = document.querySelector('tr[data-entity-id="' + id + '"].clickable-row');
                if (row) {
                    const detail = row.nextElementSibling;
                    if (detail && detail.classList.contains('detail-row')) {
                        detail.remove();
                    }
                    row.remove();
                }
            }).catch(function(err) {
                app.Toast.show(err.message || 'Failed to delete hook', 'error');
            });
        });

        document.addEventListener('click', function(e) {
            const detailsBtn = e.target.closest('[data-hook-details]');
            if (!detailsBtn) return;
            const hookId = detailsBtn.getAttribute('data-hook-details');
            const row = document.querySelector('tr[data-entity-id="' + hookId + '"].clickable-row');
            if (!row) return;
            const detailRow = row.nextElementSibling;
            if (!detailRow || !detailRow.classList.contains('detail-row')) return;
            OrgCommon.handleRowClick(row, detailRow);
        });
    };

})(window.AdminApp = window.AdminApp || {});
