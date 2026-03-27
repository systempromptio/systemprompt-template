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
