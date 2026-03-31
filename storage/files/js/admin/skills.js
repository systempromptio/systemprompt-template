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
        if (!data) {
            const noData = document.createElement('p');
            noData.className = 'text-muted';
            noData.textContent = 'No detail data available.';
            return noData;
        }

        const frag = document.createDocumentFragment();

        const descSection = document.createElement('div');
        descSection.className = 'detail-section';
        const descLabel = document.createElement('strong');
        descLabel.textContent = 'Description';
        const descP = document.createElement('p');
        descP.style.cssText = 'margin:var(--sp-space-1) 0;color:var(--sp-text-secondary);font-size:var(--sp-text-sm)';
        descP.textContent = data.description || 'No description';
        descSection.append(descLabel, descP);
        frag.append(descSection);

        if (data.command) {
            const cmdSection = document.createElement('div');
            cmdSection.className = 'detail-section';
            const cmdLabel = document.createElement('strong');
            cmdLabel.textContent = 'Command';
            const cmdPre = document.createElement('pre');
            cmdPre.style.cssText = 'margin:var(--sp-space-1) 0;font-size:var(--sp-text-xs);background:var(--sp-bg-surface-raised);padding:var(--sp-space-2);border-radius:var(--sp-radius-sm);overflow-x:auto';
            cmdPre.textContent = data.command;
            cmdSection.append(cmdLabel, cmdPre);
            frag.append(cmdSection);
        }

        if (data.tags && data.tags.length) {
            const tagSection = document.createElement('div');
            tagSection.className = 'detail-section';
            const tagLabel = document.createElement('strong');
            tagLabel.textContent = 'Tags';
            tagSection.append(tagLabel, document.createElement('br'));
            const badgeRow = document.createElement('div');
            badgeRow.className = 'badge-row';
            badgeRow.style.marginTop = 'var(--sp-space-1)';
            data.tags.forEach((tag) => {
                const badge = document.createElement('span');
                badge.className = 'badge badge-gray';
                badge.textContent = tag;
                badgeRow.append(badge);
            });
            tagSection.append(badgeRow);
            frag.append(tagSection);
        }

        const jsonSection = document.createElement('div');
        jsonSection.className = 'detail-section';
        const details = document.createElement('details');
        const summary = document.createElement('summary');
        summary.style.cssText = 'cursor:pointer;font-size:var(--sp-text-sm);color:var(--sp-text-secondary)';
        summary.textContent = 'JSON Config';
        details.append(summary);
        details.append(app.OrgCommon.formatJson(data));
        jsonSection.append(details);
        frag.append(jsonSection);

        return frag;
    }

    function initExpandRows() {
        app.OrgCommon.initExpandRows('.data-table', (row, detailRow) => {
            const content = detailRow.querySelector('[data-skill-expand]');
            if (content && !content.hasAttribute('data-loaded')) {
                const skillId = content.getAttribute('data-skill-expand');
                content.replaceChildren();
                content.append(renderSkillExpand(skillId));
                content.setAttribute('data-loaded', 'true');
            }
        });
    }

    function initDeleteHandlers() {
        app.events.on('click', '[data-delete-skill]', (e, el) => {
            const skillId = el.getAttribute('data-delete-skill');
            if (!confirm('Are you sure you want to delete skill "' + skillId + '"? This cannot be undone.')) return;

            fetch('/api/admin/skills/' + encodeURIComponent(skillId), { method: 'DELETE' })
                .then((res) => {
                    if (res.ok) {
                        app.Toast.show('Skill deleted', 'success');
                        setTimeout(() => { window.location.reload(); }, 500);
                    } else {
                        app.Toast.show('Failed to delete skill', 'error');
                    }
                })
                .catch(() => {
                    app.Toast.show('Failed to delete skill', 'error');
                });
        });
    }

    function initForkHandlers() {
        app.events.on('click', '[data-fork-skill]', (e, el) => {
            const skillId = el.getAttribute('data-fork-skill');
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
            .then((res) => {
                if (res.ok) {
                    app.Toast.show('Skill customized', 'success');
                    setTimeout(() => { window.location.reload(); }, 500);
                } else {
                    app.Toast.show('Failed to customize skill', 'error');
                }
            })
            .catch(() => {
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

        app.events.on('click', '[data-assign-skill]', (e, el) => {
            const skillId = el.getAttribute('data-assign-skill');
            const skillName = el.getAttribute('data-skill-name') || skillId;
            const data = getSkillDetail(skillId);
            const currentPluginIds = data && data.assigned_plugin_ids ? data.assigned_plugin_ids : [];
            assignApi.open(skillId, skillName, currentPluginIds);
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
                return fetch('/api/admin/plugins/' + encodeURIComponent(plugin.id) + '/skills')
                    .then((res) => { return res.json(); })
                    .then((currentSkills) => {
                        let skillIds = (currentSkills || []).slice();
                        const shouldInclude = selectedPlugins.includes(plugin.id);
                        const hasSkill = skillIds.includes(entityId);

                        if (shouldInclude && !hasSkill) {
                            skillIds.push(entityId);
                        } else if (!shouldInclude && hasSkill) {
                            skillIds = skillIds.filter((s) => { return s !== entityId; });
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

        app.events.on('click', '[data-edit-skill]', (e, el) => {
            const skillId = el.getAttribute('data-edit-skill');
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
            deleteBtn.addEventListener('click', () => {
                const ids = bulk.getSelected();
                if (!ids.length) return;
                if (!confirm('Delete ' + ids.length + ' skill(s)? This action cannot be undone.')) return;
                Promise.all(ids.map((id) => {
                    return fetch('/api/admin/skills/' + encodeURIComponent(id), { method: 'DELETE' });
                })).then(() => {
                    app.Toast.show(ids.length + ' skills deleted', 'success');
                    setTimeout(() => { window.location.reload(); }, 500);
                }).catch(() => {
                    app.Toast.show('Failed to delete some skills', 'error');
                });
            });
        }

        const assignBtn = document.getElementById('bulk-assign-btn');
        if (assignBtn && assignApi) {
            assignBtn.addEventListener('click', () => {
                const ids = bulk.getSelected();
                if (!ids.length) return;
                assignApi.open(ids.join(','), ids.length + ' skills', []);
            });
        }

        const categoryBtn = document.getElementById('bulk-category-btn');
        if (categoryBtn) {
            categoryBtn.addEventListener('click', () => {
                const ids = bulk.getSelected();
                if (!ids.length) return;
                const category = prompt('Enter category for ' + ids.length + ' skill(s):');
                if (category === null) return;
                Promise.all(ids.map((id) => {
                    return fetch('/api/public/skills/' + encodeURIComponent(id), {
                        method: 'PUT',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ category_id: category })
                    });
                })).then(() => {
                    app.Toast.show('Category updated for ' + ids.length + ' skills', 'success');
                    setTimeout(() => { window.location.reload(); }, 500);
                }).catch(() => {
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
