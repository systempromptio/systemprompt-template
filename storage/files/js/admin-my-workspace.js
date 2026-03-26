(function(app) {
    'use strict';

    var MyCommon = {

        initExpandRows: function(tableSelector, renderCallback) {
            var table = document.querySelector(tableSelector);
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

                var row = e.target.closest('tr.clickable-row');
                if (!row) return;

                var detailRow = row.nextElementSibling;
                if (!detailRow || !detailRow.classList.contains('detail-row')) return;

                MyCommon.handleRowClick(row, detailRow);

                if (renderCallback && detailRow.classList.contains('visible')) {
                    renderCallback(row, detailRow);
                }
            });
        },

        handleRowClick: function(row, detailRow) {
            var isVisible = detailRow.classList.contains('visible');

            var table = row.closest('table');
            if (table) {
                table.querySelectorAll('tr.detail-row.visible').forEach(function(r) {
                    if (r !== detailRow) {
                        r.classList.remove('visible');
                        var prevRow = r.previousElementSibling;
                        if (prevRow) {
                            var indicator = prevRow.querySelector('.expand-indicator');
                            if (indicator) indicator.classList.remove('expanded');
                        }
                    }
                });
            }

            if (!isVisible) {
                detailRow.classList.add('visible');
                var expandIndicator = row.querySelector('.expand-indicator');
                if (expandIndicator) expandIndicator.classList.add('expanded');
            } else {
                detailRow.classList.remove('visible');
                var collapseIndicator = row.querySelector('.expand-indicator');
                if (collapseIndicator) collapseIndicator.classList.remove('expanded');
            }
        },

        initSidePanel: function(panelId) {
            var panel = document.getElementById(panelId);
            if (!panel) return null;

            var overlayId = panel.getAttribute('data-overlay') || (panelId + '-overlay');
            var overlay = document.getElementById(overlayId);
            var closeBtn = panel.querySelector('[data-panel-close]');

            var api = {
                open: function() {
                    panel.classList.add('open');
                    if (overlay) overlay.classList.add('active');
                },
                close: function() {
                    panel.classList.remove('open');
                    if (overlay) overlay.classList.remove('active');
                },
                setTitle: function(text) {
                    var title = panel.querySelector('[data-panel-title]');
                    if (title) title.textContent = text;
                },
                setBody: function(html) {
                    var body = panel.querySelector('[data-panel-body]');
                    if (body) body.innerHTML = html;
                },
                setFooter: function(html) {
                    var footer = panel.querySelector('[data-panel-footer]');
                    if (footer) footer.innerHTML = html;
                },
                panel: panel
            };

            if (closeBtn) closeBtn.addEventListener('click', api.close);
            if (overlay) overlay.addEventListener('click', api.close);

            return api;
        },

        initBulkActions: function(tableSelector, barId) {
            var table = document.querySelector(tableSelector);
            if (!table) return null;

            var selected = {};

            function updateCount() {
                var count = Object.keys(selected).length;
                var countEl = document.querySelector('[data-bulk-count]');
                if (countEl) countEl.textContent = count;
                var bar = document.getElementById(barId);
                if (bar) bar.style.display = count > 0 ? 'flex' : 'none';
            }

            table.addEventListener('change', function(e) {
                if (e.target.classList.contains('bulk-select-all')) {
                    var checked = e.target.checked;
                    table.querySelectorAll('.bulk-checkbox').forEach(function(cb) {
                        cb.checked = checked;
                        var id = cb.getAttribute('data-entity-id');
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
                    var id = e.target.getAttribute('data-entity-id');
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

        initSearch: function(inputId, tableSelector) {
            var input = document.getElementById(inputId);
            var table = document.querySelector(tableSelector);
            if (!input || !table) return;

            var timer = null;
            input.addEventListener('input', function() {
                clearTimeout(timer);
                timer = setTimeout(function() {
                    var query = input.value.toLowerCase().trim();
                    var rows = table.querySelectorAll('tbody tr.clickable-row');
                    rows.forEach(function(row) {
                        var text = row.textContent.toLowerCase();
                        var matches = !query || text.indexOf(query) !== -1;
                        row.style.display = matches ? '' : 'none';
                        var detail = row.nextElementSibling;
                        if (detail && detail.classList.contains('detail-row')) {
                            detail.style.display = matches ? '' : 'none';
                        }
                    });
                }, 200);
            });
        },

        initFilterSelect: function(selectId, tableSelector, dataAttr) {
            var select = document.getElementById(selectId);
            var table = document.querySelector(tableSelector);
            if (!select || !table) return;

            select.addEventListener('change', function() {
                var value = select.value;
                var rows = table.querySelectorAll('tbody tr.clickable-row');
                rows.forEach(function(row) {
                    var attrVal = row.getAttribute(dataAttr) || '';
                    var matches = !value || attrVal === value;
                    row.style.display = matches ? '' : 'none';
                    var detail = row.nextElementSibling;
                    if (detail && detail.classList.contains('detail-row')) {
                        detail.style.display = matches ? '' : 'none';
                    }
                });
            });
        },

        initForkPanel: function(config) {
            // config: { panelId, entityType, entityLabel, onForked }
            var panelApi = MyCommon.initSidePanel(config.panelId);
            if (!panelApi) return null;

            return {
                open: function() {
                    panelApi.setTitle('Fork from Org: ' + (config.entityLabel || config.entityType));
                    panelApi.setBody('<p style="color:var(--text-tertiary);text-align:center;padding:var(--space-4)">Loading...</p>');
                    panelApi.setFooter('');
                    panelApi.open();

                    fetch(app.API_BASE + '/user/forkable/' + config.entityType)
                        .then(function(res) { return res.json(); })
                        .then(function(data) {
                            var items = data[config.entityType] || data.plugins || data.skills || data.agents || data.mcp_servers || data.hooks || [];
                            if (items.length === 0) {
                                panelApi.setBody('<p style="color:var(--text-tertiary);text-align:center;padding:var(--space-4)">No org entities available to fork.</p>');
                                return;
                            }

                            var html = '<div class="add-checklist">';
                            items.forEach(function(item) {
                                var disabled = item.already_forked ? ' disabled' : '';
                                var label = item.already_forked ? ' (already forked)' : '';
                                html += '<label class="acl-checkbox-row">' +
                                    '<input type="checkbox" name="fork_id" value="' + app.escapeHtml(item.id) + '"' + disabled + '>' +
                                    '<span class="acl-checkbox-label">' + app.escapeHtml(item.name || item.id) + label + '</span>' +
                                    '</label>';
                            });
                            html += '</div>';
                            panelApi.setBody(html);

                            panelApi.setFooter(
                                '<button class="btn btn-secondary" data-panel-close>Cancel</button> ' +
                                '<button class="btn btn-primary" data-fork-save>Fork Selected</button>'
                            );

                            var footer = panelApi.panel.querySelector('[data-panel-footer]');
                            if (footer) {
                                var cancelBtn = footer.querySelector('[data-panel-close]');
                                if (cancelBtn) cancelBtn.addEventListener('click', panelApi.close);

                                var saveBtn = footer.querySelector('[data-fork-save]');
                                if (saveBtn) {
                                    saveBtn.addEventListener('click', function() {
                                        var checked = panelApi.panel.querySelectorAll('input[name="fork_id"]:checked');
                                        if (checked.length === 0) {
                                            app.Toast.show('Select at least one entity to fork', 'warning');
                                            return;
                                        }
                                        saveBtn.disabled = true;
                                        saveBtn.textContent = 'Forking...';

                                        var promises = [];
                                        var typeKey = config.entityType.replace(/s$/, '');
                                        checked.forEach(function(cb) {
                                            var body = {};
                                            body['org_' + typeKey + '_id'] = cb.value;
                                            promises.push(
                                                fetch(app.API_BASE + '/user/fork/' + typeKey.replace('_', '-'), {
                                                    method: 'POST',
                                                    headers: { 'Content-Type': 'application/json' },
                                                    body: JSON.stringify(body)
                                                })
                                            );
                                        });

                                        Promise.all(promises).then(function(results) {
                                            var ok = results.filter(function(r) { return r.ok; }).length;
                                            app.Toast.show('Forked ' + ok + ' ' + config.entityLabel + '(s)', 'success');
                                            panelApi.close();
                                            if (config.onForked) config.onForked();
                                            else setTimeout(function() { window.location.reload(); }, 500);
                                        }).catch(function() {
                                            app.Toast.show('Fork failed', 'error');
                                            saveBtn.disabled = false;
                                            saveBtn.textContent = 'Fork Selected';
                                        });
                                    });
                                }
                            }
                        })
                        .catch(function() {
                            panelApi.setBody('<p style="color:var(--danger);text-align:center;padding:var(--space-4)">Failed to load forkable entities.</p>');
                        });
                },
                close: panelApi.close,
                panel: panelApi
            };
        },

        formatJson: function(data) {
            if (typeof data === 'string') {
                try { data = JSON.parse(data); } catch (e) { return app.escapeHtml(data); }
            }
            return '<pre class="json-view">' + app.escapeHtml(JSON.stringify(data, null, 2)) + '</pre>';
        },

        renderSourceBadge: function(baseId) {
            if (baseId) {
                return '<span class="fork-indicator forked">' +
                    '<svg class="fork-icon" viewBox="0 0 16 16" fill="currentColor"><path d="M5 3.25a.75.75 0 11-1.5 0 .75.75 0 011.5 0zm0 2.122a2.25 2.25 0 10-1.5 0v.878A2.25 2.25 0 005.75 8.5h1.5v2.128a2.251 2.251 0 101.5 0V8.5h1.5a2.25 2.25 0 002.25-2.25v-.878a2.25 2.25 0 10-1.5 0v.878a.75.75 0 01-.75.75h-4.5A.75.75 0 015 6.25v-.878z"/></svg>' +
                    'forked</span>';
            }
            return '<span class="fork-indicator custom">' +
                '<svg class="fork-icon" viewBox="0 0 16 16" fill="currentColor"><path d="M8 2a6 6 0 100 12A6 6 0 008 2zm.75 3.75v2.5h2.5v1.5h-2.5v2.5h-1.5v-2.5h-2.5v-1.5h2.5v-2.5h1.5z"/></svg>' +
                'custom</span>';
        }
    };

    // Expose MyCommon on app
    app.MyCommon = MyCommon;

    // ---- Page initializers ----

    app.initMyPlugins = function() {
        MyCommon.initExpandRows('#my-plugins-table');
        MyCommon.initSearch('my-plugins-search', '#my-plugins-table');
        MyCommon.initFilterSelect('my-plugins-category-filter', '#my-plugins-table', 'data-category');
        MyCommon.initBulkActions('#my-plugins-table', 'my-plugins-bulk-bar');

        var forkBtn = document.getElementById('my-plugins-fork-btn');
        if (forkBtn) {
            var forkPanel = MyCommon.initForkPanel({
                panelId: 'fork-panel',
                entityType: 'plugins',
                entityLabel: 'plugin'
            });
            if (forkPanel) {
                forkBtn.addEventListener('click', forkPanel.open);
            }
        }
    };

    app.initMySkills = function() {
        MyCommon.initExpandRows('#my-skills-table');
        MyCommon.initSearch('my-skills-search', '#my-skills-table');
        MyCommon.initFilterSelect('my-skills-tag-filter', '#my-skills-table', 'data-tags');
        MyCommon.initBulkActions('#my-skills-table', 'my-skills-bulk-bar');

        var forkBtn = document.getElementById('my-skills-fork-btn');
        if (forkBtn) {
            var forkPanel = MyCommon.initForkPanel({
                panelId: 'fork-panel',
                entityType: 'skills',
                entityLabel: 'skill'
            });
            if (forkPanel) {
                forkBtn.addEventListener('click', forkPanel.open);
            }
        }
    };

    app.initMyAgents = function() {
        MyCommon.initExpandRows('#my-agents-table');
        MyCommon.initSearch('my-agents-search', '#my-agents-table');
        MyCommon.initBulkActions('#my-agents-table', 'my-agents-bulk-bar');

        var forkBtn = document.getElementById('my-agents-fork-btn');
        if (forkBtn) {
            var forkPanel = MyCommon.initForkPanel({
                panelId: 'fork-panel',
                entityType: 'agents',
                entityLabel: 'agent'
            });
            if (forkPanel) {
                forkBtn.addEventListener('click', forkPanel.open);
            }
        }
    };

    app.initMyMcpServers = function() {
        MyCommon.initExpandRows('#my-mcp-table');
        MyCommon.initSearch('my-mcp-search', '#my-mcp-table');
        MyCommon.initBulkActions('#my-mcp-table', 'my-mcp-bulk-bar');

        var forkBtn = document.getElementById('my-mcp-fork-btn');
        if (forkBtn) {
            var forkPanel = MyCommon.initForkPanel({
                panelId: 'fork-panel',
                entityType: 'mcp-servers',
                entityLabel: 'MCP server'
            });
            if (forkPanel) {
                forkBtn.addEventListener('click', forkPanel.open);
            }
        }
    };

    app.initMyHooks = function() {
        MyCommon.initExpandRows('#my-hooks-table');
        MyCommon.initSearch('my-hooks-search', '#my-hooks-table');
        MyCommon.initBulkActions('#my-hooks-table', 'my-hooks-bulk-bar');

        var forkBtn = document.getElementById('my-hooks-fork-btn');
        if (forkBtn) {
            var forkPanel = MyCommon.initForkPanel({
                panelId: 'fork-panel',
                entityType: 'hooks',
                entityLabel: 'hook'
            });
            if (forkPanel) {
                forkBtn.addEventListener('click', forkPanel.open);
            }
        }
    };

    app.initMyMarketplace = function() {
        MyCommon.initExpandRows('#my-marketplace-table');
        MyCommon.initSearch('my-marketplace-search', '#my-marketplace-table');
        MyCommon.initFilterSelect('my-marketplace-source-filter', '#my-marketplace-table', 'data-source');
        MyCommon.initFilterSelect('my-marketplace-category-filter', '#my-marketplace-table', 'data-category');

        // Customize button handler
        document.addEventListener('click', function(e) {
            var btn = e.target.closest('[data-customize-plugin]');
            if (!btn) return;
            var pluginId = btn.getAttribute('data-customize-plugin');
            btn.disabled = true;
            btn.textContent = 'Customizing...';

            fetch(app.API_BASE + '/user/fork/plugin', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ org_plugin_id: pluginId })
            }).then(function(res) {
                if (res.ok) {
                    return res.json().then(function(data) {
                        app.Toast.show('Plugin customized successfully', 'success');
                        if (data.plugin && data.plugin.plugin_id) {
                            setTimeout(function() {
                                window.location.href = '/admin/my/plugins/edit?id=' + encodeURIComponent(data.plugin.plugin_id);
                            }, 500);
                        } else {
                            setTimeout(function() { window.location.reload(); }, 500);
                        }
                    });
                } else {
                    app.Toast.show('Failed to customize plugin', 'error');
                    btn.disabled = false;
                    btn.textContent = 'Customize';
                }
            }).catch(function() {
                app.Toast.show('Failed to customize plugin', 'error');
                btn.disabled = false;
                btn.textContent = 'Customize';
            });
        });
    };

})(window.AdminApp || (window.AdminApp = {}));
