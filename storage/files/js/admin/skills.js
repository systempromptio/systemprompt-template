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
