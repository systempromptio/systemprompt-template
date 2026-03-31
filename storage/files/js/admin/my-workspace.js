(function(app) {
    'use strict';

    const MyCommon = {

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

                MyCommon.handleRowClick(row, detailRow);

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
                setBodyText: (text) => {
                    const body = panel.querySelector('[data-panel-body]');
                    if (!body) return;
                    body.replaceChildren();
                    const p = document.createElement('p');
                    p.style.cssText = 'color:var(--sp-text-tertiary);text-align:center;padding:var(--sp-space-4)';
                    p.textContent = text;
                    body.append(p);
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
                    if (el) footer.append(el);
                },
                panel: panel
            };

            if (closeBtn) closeBtn.addEventListener('click', api.close);
            if (overlay) overlay.addEventListener('click', api.close);

            return api;
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
                getSelected: () => { return Object.keys(selected); },
                clear: () => {
                    selected = {};
                    table.querySelectorAll('.bulk-checkbox, .bulk-select-all').forEach((cb) => {
                        cb.checked = false;
                    });
                    updateCount();
                }
            };
        },

        initSearch: (inputId, tableSelector) => {
            const input = document.getElementById(inputId);
            const table = document.querySelector(tableSelector);
            if (!input || !table) return;

            let timer = null;
            input.addEventListener('input', () => {
                clearTimeout(timer);
                timer = setTimeout(() => {
                    const query = input.value.toLowerCase().trim();
                    const rows = table.querySelectorAll('tbody tr.clickable-row');
                    rows.forEach((row) => {
                        const text = row.textContent.toLowerCase();
                        const matches = !query || text.includes(query);
                        row.style.display = matches ? '' : 'none';
                        const detail = row.nextElementSibling;
                        if (detail && detail.classList.contains('detail-row')) {
                            detail.style.display = matches ? '' : 'none';
                        }
                    });
                }, 200);
            });
        },

        initFilterSelect: (selectId, tableSelector, dataAttr) => {
            const select = document.getElementById(selectId);
            const table = document.querySelector(tableSelector);
            if (!select || !table) return;

            select.addEventListener('change', () => {
                const value = select.value;
                const rows = table.querySelectorAll('tbody tr.clickable-row');
                rows.forEach((row) => {
                    const attrVal = row.getAttribute(dataAttr) || '';
                    const matches = !value || attrVal === value;
                    row.style.display = matches ? '' : 'none';
                    const detail = row.nextElementSibling;
                    if (detail && detail.classList.contains('detail-row')) {
                        detail.style.display = matches ? '' : 'none';
                    }
                });
            });
        },

        initForkPanel: (config) => {
            const panelApi = MyCommon.initSidePanel(config.panelId);
            if (!panelApi) return null;

            return {
                open: () => {
                    panelApi.setTitle('Fork from Org: ' + (config.entityLabel || config.entityType));
                    panelApi.setBodyText('Loading...');
                    panelApi.setFooterDom(null);
                    panelApi.open();

                    fetch(app.API_BASE + '/user/forkable/' + config.entityType)
                        .then((res) => { return res.json(); })
                        .then((data) => {
                            const items = data[config.entityType] || data.plugins || data.skills || data.agents || data.mcp_servers || data.hooks || [];
                            if (items.length === 0) {
                                panelApi.setBodyText('No org entities available to fork.');
                                return;
                            }

                            const checklist = document.createElement('div');
                            checklist.className = 'add-checklist';
                            items.forEach((item) => {
                                const label = document.createElement('label');
                                label.className = 'acl-checkbox-row';
                                const input = document.createElement('input');
                                input.type = 'checkbox';
                                input.name = 'fork_id';
                                input.value = item.id;
                                if (item.already_forked) input.disabled = true;
                                const span = document.createElement('span');
                                span.className = 'acl-checkbox-label';
                                span.textContent = (item.name || item.id) + (item.already_forked ? ' (already forked)' : '');
                                label.append(input, span);
                                checklist.append(label);
                            });
                            panelApi.setBodyDom(checklist);

                            const footerFrag = document.createDocumentFragment();
                            const cancelBtn = document.createElement('button');
                            cancelBtn.className = 'btn btn-secondary';
                            cancelBtn.setAttribute('data-panel-close', '');
                            cancelBtn.textContent = 'Cancel';
                            const saveBtn = document.createElement('button');
                            saveBtn.className = 'btn btn-primary';
                            saveBtn.setAttribute('data-fork-save', '');
                            saveBtn.textContent = 'Fork Selected';
                            footerFrag.append(cancelBtn, document.createTextNode(' '), saveBtn);
                            panelApi.setFooterDom(footerFrag);

                            cancelBtn.addEventListener('click', panelApi.close);

                            saveBtn.addEventListener('click', () => {
                                const checked = panelApi.panel.querySelectorAll('input[name="fork_id"]:checked');
                                if (checked.length === 0) {
                                    app.Toast.show('Select at least one entity to fork', 'warning');
                                    return;
                                }
                                saveBtn.disabled = true;
                                saveBtn.textContent = 'Forking...';

                                const promises = [];
                                const typeKey = config.entityType.replace(/s$/, '');
                                checked.forEach((cb) => {
                                    const body = {};
                                    body['org_' + typeKey + '_id'] = cb.value;
                                    promises.push(
                                        fetch(app.API_BASE + '/user/fork/' + typeKey.replace('_', '-'), {
                                            method: 'POST',
                                            headers: { 'Content-Type': 'application/json' },
                                            body: JSON.stringify(body)
                                        })
                                    );
                                });

                                Promise.all(promises).then((results) => {
                                    const ok = results.filter((r) => { return r.ok; }).length;
                                    app.Toast.show('Forked ' + ok + ' ' + config.entityLabel + '(s)', 'success');
                                    panelApi.close();
                                    if (config.onForked) config.onForked();
                                    else setTimeout(() => { window.location.reload(); }, 500);
                                }).catch(() => {
                                    app.Toast.show('Fork failed', 'error');
                                    saveBtn.disabled = false;
                                    saveBtn.textContent = 'Fork Selected';
                                });
                            });
                        })
                        .catch(() => {
                            const errP = document.createElement('p');
                            errP.style.cssText = 'color:var(--sp-danger);text-align:center;padding:var(--sp-space-4)';
                            errP.textContent = 'Failed to load forkable entities.';
                            panelApi.setBodyDom(errP);
                        });
                },
                close: panelApi.close,
                panel: panelApi
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

        renderSourceBadge: (baseId) => {
            const container = document.createElement('span');
            const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
            svg.setAttribute('class', 'fork-icon');
            svg.setAttribute('viewBox', '0 0 16 16');
            svg.setAttribute('fill', 'currentColor');
            const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
            if (baseId) {
                container.className = 'fork-indicator forked';
                path.setAttribute('d', 'M5 3.25a.75.75 0 11-1.5 0 .75.75 0 011.5 0zm0 2.122a2.25 2.25 0 10-1.5 0v.878A2.25 2.25 0 005.75 8.5h1.5v2.128a2.251 2.251 0 101.5 0V8.5h1.5a2.25 2.25 0 002.25-2.25v-.878a2.25 2.25 0 10-1.5 0v.878a.75.75 0 01-.75.75h-4.5A.75.75 0 015 6.25v-.878z');
                svg.append(path);
                container.append(svg, 'forked');
            } else {
                container.className = 'fork-indicator custom';
                path.setAttribute('d', 'M8 2a6 6 0 100 12A6 6 0 008 2zm.75 3.75v2.5h2.5v1.5h-2.5v2.5h-1.5v-2.5h-2.5v-1.5h2.5v-2.5h1.5z');
                svg.append(path);
                container.append(svg, 'custom');
            }
            return container;
        }
    };

    app.MyCommon = MyCommon;

    app.initMyPlugins = () => {
        MyCommon.initExpandRows('#my-plugins-table');
        MyCommon.initSearch('my-plugins-search', '#my-plugins-table');
        MyCommon.initFilterSelect('my-plugins-category-filter', '#my-plugins-table', 'data-category');
        MyCommon.initBulkActions('#my-plugins-table', 'my-plugins-bulk-bar');

        const forkBtn = document.getElementById('my-plugins-fork-btn');
        if (forkBtn) {
            const forkPanel = MyCommon.initForkPanel({
                panelId: 'fork-panel',
                entityType: 'plugins',
                entityLabel: 'plugin'
            });
            if (forkPanel) {
                forkBtn.addEventListener('click', forkPanel.open);
            }
        }
    };

    app.initMySkills = () => {
        MyCommon.initExpandRows('#my-skills-table');
        MyCommon.initSearch('my-skills-search', '#my-skills-table');
        MyCommon.initFilterSelect('my-skills-tag-filter', '#my-skills-table', 'data-tags');
        MyCommon.initBulkActions('#my-skills-table', 'my-skills-bulk-bar');

        const forkBtn = document.getElementById('my-skills-fork-btn');
        if (forkBtn) {
            const forkPanel = MyCommon.initForkPanel({
                panelId: 'fork-panel',
                entityType: 'skills',
                entityLabel: 'skill'
            });
            if (forkPanel) {
                forkBtn.addEventListener('click', forkPanel.open);
            }
        }
    };

    app.initMyAgents = () => {
        MyCommon.initExpandRows('#my-agents-table');
        MyCommon.initSearch('my-agents-search', '#my-agents-table');
        MyCommon.initBulkActions('#my-agents-table', 'my-agents-bulk-bar');

        const forkBtn = document.getElementById('my-agents-fork-btn');
        if (forkBtn) {
            const forkPanel = MyCommon.initForkPanel({
                panelId: 'fork-panel',
                entityType: 'agents',
                entityLabel: 'agent'
            });
            if (forkPanel) {
                forkBtn.addEventListener('click', forkPanel.open);
            }
        }
    };

    app.initMyMcpServers = () => {
        MyCommon.initExpandRows('#my-mcp-table');
        MyCommon.initSearch('my-mcp-search', '#my-mcp-table');
        MyCommon.initBulkActions('#my-mcp-table', 'my-mcp-bulk-bar');

        const forkBtn = document.getElementById('my-mcp-fork-btn');
        if (forkBtn) {
            const forkPanel = MyCommon.initForkPanel({
                panelId: 'fork-panel',
                entityType: 'mcp-servers',
                entityLabel: 'MCP server'
            });
            if (forkPanel) {
                forkBtn.addEventListener('click', forkPanel.open);
            }
        }
    };

    app.initMyHooks = () => {
        MyCommon.initExpandRows('#my-hooks-table');
        MyCommon.initSearch('my-hooks-search', '#my-hooks-table');
        MyCommon.initBulkActions('#my-hooks-table', 'my-hooks-bulk-bar');

        const forkBtn = document.getElementById('my-hooks-fork-btn');
        if (forkBtn) {
            const forkPanel = MyCommon.initForkPanel({
                panelId: 'fork-panel',
                entityType: 'hooks',
                entityLabel: 'hook'
            });
            if (forkPanel) {
                forkBtn.addEventListener('click', forkPanel.open);
            }
        }
    };

    app.initMyMarketplace = () => {
        MyCommon.initExpandRows('#my-marketplace-table');
        MyCommon.initSearch('my-marketplace-search', '#my-marketplace-table');
        MyCommon.initFilterSelect('my-marketplace-source-filter', '#my-marketplace-table', 'data-source');
        MyCommon.initFilterSelect('my-marketplace-category-filter', '#my-marketplace-table', 'data-category');

        app.events.on('click', '[data-customize-plugin]', (e, btn) => {
            const pluginId = btn.getAttribute('data-customize-plugin');
            btn.disabled = true;
            btn.textContent = 'Customizing...';

            fetch(app.API_BASE + '/user/fork/plugin', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ org_plugin_id: pluginId })
            }).then((res) => {
                if (res.ok) {
                    return res.json().then((data) => {
                        app.Toast.show('Plugin customized successfully', 'success');
                        if (data.plugin && data.plugin.plugin_id) {
                            setTimeout(() => {
                                window.location.href = '/admin/my/plugins/edit?id=' + encodeURIComponent(data.plugin.plugin_id);
                            }, 500);
                        } else {
                            setTimeout(() => { window.location.reload(); }, 500);
                        }
                    });
                } else {
                    app.Toast.show('Failed to customize plugin', 'error');
                    btn.disabled = false;
                    btn.textContent = 'Customize';
                }
            }).catch(() => {
                app.Toast.show('Failed to customize plugin', 'error');
                btn.disabled = false;
                btn.textContent = 'Customize';
            });
        });
    };

})(window.AdminApp || (window.AdminApp = {}));
