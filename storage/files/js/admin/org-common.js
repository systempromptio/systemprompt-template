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
