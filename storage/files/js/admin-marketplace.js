(function(app) {
    'use strict';

    const escapeHtml = app.escapeHtml;
    let plugins = [];

    function showVisibilityModal(pluginId) {
        const plugin = plugins.find(function(p) { return p.id === pluginId; });
        if (!plugin) return;
        const rules = plugin.visibility_rules || [];

        const overlay = document.createElement('div');
        overlay.className = 'confirm-overlay';
        overlay.id = 'visibility-modal';

        const rulesListHtml = renderRulesList(rules);

        overlay.innerHTML = '<div class="confirm-dialog" style="max-width:500px">' +
            '<h3 style="margin:0 0 var(--space-3)">Edit Visibility - ' + escapeHtml(plugin.name) + '</h3>' +
            '<div id="visibility-rules-list">' + rulesListHtml + '</div>' +
            '<div style="margin-top:var(--space-4);padding-top:var(--space-3);border-top:1px solid var(--border-primary)">' +
                '<strong style="font-size:var(--text-sm)">Add Rule</strong>' +
                '<div style="display:flex;gap:var(--space-2);margin-top:var(--space-2);flex-wrap:wrap">' +
                    '<select id="vis-rule-type" class="btn btn-secondary" style="cursor:pointer;font-size:var(--text-sm)">' +
                        '<option value="department">Department</option>' +
                        '<option value="user">User</option>' +
                    '</select>' +
                    '<input type="text" id="vis-rule-value" class="search-input" placeholder="Value..." style="flex:1;min-width:120px;font-size:var(--text-sm)">' +
                    '<select id="vis-rule-access" class="btn btn-secondary" style="cursor:pointer;font-size:var(--text-sm)">' +
                        '<option value="allow">Allow</option>' +
                        '<option value="deny">Deny</option>' +
                    '</select>' +
                    '<button class="btn btn-secondary" id="vis-add-rule" style="font-size:var(--text-sm)">Add</button>' +
                '</div>' +
            '</div>' +
            '<div style="display:flex;gap:var(--space-3);justify-content:flex-end;margin-top:var(--space-4)">' +
                '<button class="btn btn-secondary" data-confirm-cancel>Cancel</button>' +
                '<button class="btn btn-primary" id="vis-save">Save</button>' +
            '</div>' +
        '</div>';

        document.body.appendChild(overlay);

        const modalRules = rules.slice();

        function refreshRulesList() {
            const container = overlay.querySelector('#visibility-rules-list');
            if (container) container.innerHTML = renderRulesList(modalRules);
        }

        overlay.addEventListener('click', async function(e) {
            if (e.target === overlay || e.target.closest('[data-confirm-cancel]')) {
                overlay.remove();
                return;
            }
            const removeBtn = e.target.closest('[data-remove-rule]');
            if (removeBtn) {
                modalRules.splice(parseInt(removeBtn.getAttribute('data-remove-rule'), 10), 1);
                refreshRulesList();
                return;
            }
            if (e.target.closest('#vis-add-rule')) {
                const ruleValue = overlay.querySelector('#vis-rule-value').value.trim();
                if (!ruleValue) { app.Toast.show('Rule value is required', 'error'); return; }
                modalRules.push({
                    rule_type: overlay.querySelector('#vis-rule-type').value,
                    rule_value: ruleValue,
                    access: overlay.querySelector('#vis-rule-access').value
                });
                overlay.querySelector('#vis-rule-value').value = '';
                refreshRulesList();
                return;
            }
            if (e.target.closest('#vis-save')) {
                const saveBtn = overlay.querySelector('#vis-save');
                saveBtn.disabled = true;
                saveBtn.textContent = 'Saving...';
                try {
                    await app.api('/marketplace-plugins/' + encodeURIComponent(pluginId) + '/visibility', {
                        method: 'PUT',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ rules: modalRules })
                    });
                    app.Toast.show('Visibility updated', 'success');
                    overlay.remove();
                    window.location.reload();
                } catch (err) {
                    app.Toast.show(err.message || 'Failed to save visibility', 'error');
                    saveBtn.disabled = false;
                    saveBtn.textContent = 'Save';
                }
            }
        });
    }

    function renderRulesList(rules) {
        if (!rules.length) return '<p style="font-size:var(--text-sm);color:var(--text-tertiary)">No rules configured</p>';
        return rules.map(function(rule, idx) {
            return '<div style="display:flex;align-items:center;gap:var(--space-2);padding:var(--space-1) 0;font-size:var(--text-sm)">' +
                '<span class="badge ' + (rule.access === 'allow' ? 'badge-yellow' : 'badge-red') + '">' + escapeHtml(rule.rule_type) + ': ' + escapeHtml(rule.rule_value) + ' (' + escapeHtml(rule.access) + ')</span>' +
                '<button class="btn btn-danger" style="font-size:var(--text-xs);padding:2px 6px" data-remove-rule="' + idx + '">Remove</button>' +
            '</div>';
        }).join('');
    }

    app.initMarketplace = function(selector, pluginsData) {
        const root = document.querySelector(selector);
        if (!root) return;
        plugins = pluginsData || [];

        const searchInput = document.getElementById('mkt-search');
        if (searchInput) {
            const debounceTimer = null;
            searchInput.addEventListener('input', function() {
                clearTimeout(debounceTimer);
                debounceTimer = setTimeout(function() {
                    const q = searchInput.value.toLowerCase().trim();
                    const cards = root.querySelectorAll('.plugin-card[data-plugin-id]');
                    for (let i = 0; i < cards.length; i++) {
                        const name = cards[i].getAttribute('data-search-name') || '';
                        const desc = cards[i].getAttribute('data-search-desc') || '';
                        const cat = cards[i].getAttribute('data-search-cat') || '';
                        cards[i].style.display = (!q || name.indexOf(q) >= 0 || desc.indexOf(q) >= 0 || cat.indexOf(q) >= 0) ? '' : 'none';
                    }
                }, 200);
            });
        }

        const sortSelect = document.getElementById('mkt-sort');
        if (sortSelect) {
            sortSelect.addEventListener('change', function() {
                const url = new URL(window.location.href);
                url.searchParams.set('sort', sortSelect.value);
                window.location.href = url.toString();
            });
        }

        root.addEventListener('click', async function(e) {
            const toggleBtn = e.target.closest('[data-toggle-plugin]');
            if (toggleBtn) {
                const card = toggleBtn.closest('.plugin-card');
                const details = card && card.querySelector('.plugin-details');
                if (details) {
                    const isHidden = details.style.display === 'none';
                    details.style.display = isHidden ? '' : 'none';
                    const icon = toggleBtn.querySelector('.expand-icon');
                    if (icon) icon.style.transform = isHidden ? 'rotate(180deg)' : '';
                }
                return;
            }

            const visBtn = e.target.closest('[data-edit-visibility]');
            if (visBtn) {
                e.stopPropagation();
                showVisibilityModal(visBtn.getAttribute('data-edit-visibility'));
                return;
            }

            const loadUsersBtn = e.target.closest('[data-load-users]');
            if (loadUsersBtn) {
                e.stopPropagation();
                const pluginId = loadUsersBtn.getAttribute('data-load-users');
                loadUsersBtn.disabled = true;
                loadUsersBtn.textContent = 'Loading...';
                try {
                    const usersData = await app.api('/marketplace-plugins/' + encodeURIComponent(pluginId) + '/users');
                    const users = usersData.users || usersData || [];
                    const container = root.querySelector('[data-users-for="' + pluginId + '"]');
                    if (container) {
                        if (users.length === 0) {
                            container.innerHTML = '<div style="margin-top:var(--space-2);font-size:var(--text-xs);color:var(--text-tertiary)">No users found</div>';
                        } else {
                            container.innerHTML = '<div style="margin-top:var(--space-2);display:flex;flex-direction:column;gap:var(--space-1)">' +
                                users.map(function(u) {
                                    return '<div style="display:flex;align-items:center;gap:var(--space-2);font-size:var(--text-xs);padding:var(--space-1) 0;border-bottom:1px solid var(--border-primary)">' +
                                        '<span style="font-weight:600;color:var(--text-primary)">' + escapeHtml(u.display_name || 'Unknown') + '</span>' +
                                        (u.department ? '<span class="badge badge-blue">' + escapeHtml(u.department) + '</span>' : '') +
                                        '<span class="badge badge-gray">' + (u.event_count || 0) + ' events</span>' +
                                        (u.last_used ? '<span style="color:var(--text-tertiary)">' + new Date(u.last_used).toLocaleDateString() + '</span>' : '') +
                                    '</div>';
                                }).join('') +
                            '</div>';
                        }
                    }
                    loadUsersBtn.style.display = 'none';
                } catch (err) {
                    app.Toast.show(err.message || 'Failed to load users', 'error');
                    loadUsersBtn.disabled = false;
                    loadUsersBtn.textContent = 'Load Users';
                }
                return;
            }
        });
    };
})(window.AdminApp);

(function(app) {
    'use strict';

    const escapeHtml = app.escapeHtml;
    const versionDetails = {};
    const diffCache = {};
    let activeDiff = null;

    function marketplaceApi(userId, path) {
        const url = '/api/public/marketplace/' + encodeURIComponent(userId) + path;
        return fetch(url, { headers: { 'Content-Type': 'application/json' } })
            .then(function(resp) {
                if (!resp.ok) return resp.text().then(function(t) { throw new Error(t || resp.statusText); });
                return resp.json();
            });
    }

    function renderSkillRow(skill, versionId) {
        const hasBase = skill.base_skill_id && skill.base_skill_id !== 'null';
        const compareBtn = '';
        if (hasBase) {
            const isActive = activeDiff && activeDiff.versionId === versionId && activeDiff.skillId === skill.skill_id;
            compareBtn = '<button class="btn btn-secondary btn-sm" data-compare-skill="' + escapeHtml(skill.skill_id) +
                '" data-compare-version="' + escapeHtml(versionId) +
                '" data-base-skill="' + escapeHtml(skill.base_skill_id) +
                '" style="font-size:var(--text-xs);padding:2px 8px;white-space:nowrap"' +
                (isActive ? ' disabled' : '') + '>' +
                (isActive ? 'Viewing Diff' : 'Compare to Core') + '</button>';
        }
        const enabledBadge = skill.enabled === false
            ? '<span class="badge badge-red">disabled</span>'
            : '<span class="badge badge-green">enabled</span>';
        const baseBadge = hasBase
            ? '<span class="badge badge-yellow">customized</span>'
            : '<span class="badge badge-gray">custom</span>';

        return '<div class="detail-item">' +
            '<div class="detail-item-info">' +
                '<div class="detail-item-name">' +
                    escapeHtml(skill.name || skill.skill_id) +
                    ' ' + baseBadge + ' ' + enabledBadge +
                '</div>' +
                '<div style="font-size:var(--text-xs);color:var(--text-tertiary);margin-top:2px">' +
                    '<code style="background:var(--bg-surface-raised);padding:1px 6px;border-radius:var(--radius-xs)">' + escapeHtml(skill.skill_id) + '</code>' +
                    (skill.version ? ' <span>v' + escapeHtml(skill.version) + '</span>' : '') +
                    (skill.description ? ' &mdash; ' + escapeHtml(app.shared.truncate(skill.description, 80)) : '') +
                '</div>' +
            '</div>' +
            compareBtn +
        '</div>';
    }

    function renderDiffPanel(userSkill, coreSkill) {
        const userLines = (userSkill.content || '').split('\n');
        const coreLines = (coreSkill.content || '').split('\n');
        const maxLen = Math.max(userLines.length, coreLines.length);
        let diffHtml = '';
        for (let i = 0; i < maxLen; i++) {
            const coreLine = i < coreLines.length ? coreLines[i] : '';
            const userLine = i < userLines.length ? userLines[i] : '';
            const lineNum = i + 1;
            if (coreLine === userLine) {
                diffHtml += '<div class="diff-line diff-unchanged"><span class="diff-linenum">' + lineNum + '</span><span class="diff-text">' + escapeHtml(coreLine) + '</span></div>';
            } else {
                if (coreLine) diffHtml += '<div class="diff-line diff-removed"><span class="diff-linenum">' + lineNum + '</span><span class="diff-text">- ' + escapeHtml(coreLine) + '</span></div>';
                if (userLine) diffHtml += '<div class="diff-line diff-added"><span class="diff-linenum">' + lineNum + '</span><span class="diff-text">+ ' + escapeHtml(userLine) + '</span></div>';
            }
        }

        let metaDiff = '';
        if ((userSkill.name || '') !== (coreSkill.name || '')) {
            metaDiff += '<div style="margin-bottom:var(--space-2)"><strong>Name:</strong> <span class="diff-removed" style="padding:1px 4px">' + escapeHtml(coreSkill.name || '') + '</span> &rarr; <span class="diff-added" style="padding:1px 4px">' + escapeHtml(userSkill.name || '') + '</span></div>';
        }
        if ((userSkill.description || '') !== (coreSkill.description || '')) {
            metaDiff += '<div style="margin-bottom:var(--space-2)"><strong>Description:</strong> <span class="diff-removed" style="padding:1px 4px">' + escapeHtml(coreSkill.description || '') + '</span> &rarr; <span class="diff-added" style="padding:1px 4px">' + escapeHtml(userSkill.description || '') + '</span></div>';
        }

        return '<div class="diff-panel">' +
            '<div class="diff-panel-header">' +
                '<h4 style="margin:0;font-size:var(--text-sm);font-weight:600">Diff: ' + escapeHtml(userSkill.skill_id) + '</h4>' +
                '<div style="display:flex;gap:var(--space-3);font-size:var(--text-xs)">' +
                    '<span><span class="badge badge-blue">core</span> Base skill</span>' +
                    '<span><span class="badge badge-green">user</span> User version</span>' +
                '</div>' +
                '<button class="btn btn-secondary btn-sm" data-close-diff style="margin-left:auto;font-size:var(--text-xs);padding:2px 8px">Close</button>' +
            '</div>' +
            (metaDiff ? '<div style="padding:var(--space-3) var(--space-4);border-bottom:1px solid var(--border-subtle);font-size:var(--text-sm)">' + metaDiff + '</div>' : '') +
            '<div class="diff-content">' + (diffHtml || '<div style="padding:var(--space-4);color:var(--text-tertiary);text-align:center">Content is identical</div>') + '</div>' +
        '</div>';
    }

    function renderVersionDetails(detailsContainer, versionId) {
        const detail = versionDetails[versionId];
        if (!detail || detail === 'loading') return;
        if (detail === 'error') {
            detailsContainer.innerHTML = '<div style="padding:var(--space-4)"><div class="empty-state"><p>Failed to load version details.</p></div></div>';
            return;
        }
        let skills = [];
        if (Array.isArray(detail.skills_snapshot)) {
            skills = detail.skills_snapshot;
        } else if (typeof detail.skills_snapshot === 'string') {
            try { skills = JSON.parse(detail.skills_snapshot); } catch(e) { skills = []; }
        }
        const skillsHtml = skills.length
            ? skills.map(function(s) { return renderSkillRow(s, versionId); }).join('')
            : '<div class="empty-state" style="padding:var(--space-4)"><p>No skills in this snapshot.</p></div>';

        let diffHtml = '';
        if (activeDiff && activeDiff.versionId === versionId && diffCache[activeDiff.cacheKey]) {
            const userSkill = skills.find(function(s) { return s.skill_id === activeDiff.skillId; });
            if (userSkill) diffHtml = renderDiffPanel(userSkill, diffCache[activeDiff.cacheKey]);
        }

        detailsContainer.innerHTML =
            '<div style="padding:var(--space-4)">' +
                '<div style="font-size:var(--text-sm);font-weight:600;margin-bottom:var(--space-2);color:var(--text-secondary)">Skills Snapshot (' + skills.length + ')</div>' +
                skillsHtml +
            '</div>' +
            diffHtml;
    }

    async function loadVersionDetail(versionId, userId, detailsContainer) {
        if (versionDetails[versionId] && versionDetails[versionId] !== 'loading') {
            renderVersionDetails(detailsContainer, versionId);
            return;
        }
        versionDetails[versionId] = 'loading';
        try {
            const detail = await marketplaceApi(userId, '/versions/' + encodeURIComponent(versionId));
            versionDetails[versionId] = detail;
        } catch(err) {
            versionDetails[versionId] = 'error';
        }
        renderVersionDetails(detailsContainer, versionId);
    }

    async function loadCoreDiff(skillId, baseSkillId, versionId, detailsContainer) {
        const cacheKey = baseSkillId + ':' + skillId;
        if (!diffCache[cacheKey]) {
            try {
                const coreData = await app.api('/skills/' + encodeURIComponent(baseSkillId) + '/base-content');
                diffCache[cacheKey] = coreData;
            } catch(err) {
                app.Toast.show('Failed to load core skill: ' + (err.message || 'Unknown error'), 'error');
                return;
            }
        }
        activeDiff = { versionId: versionId, skillId: skillId, cacheKey: cacheKey };
        renderVersionDetails(detailsContainer, versionId);
    }

    async function handleRestore(versionId, versionNum, userId) {
        if (!confirm('Restore to version ' + versionNum + '? Current state will be saved as a new version.')) return;
        try {
            const result = await fetch('/api/public/marketplace/' + encodeURIComponent(userId) + '/restore/' + encodeURIComponent(versionId), {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' }
            });
            if (!result.ok) throw new Error(await result.text() || 'Restore failed');
            const data = await result.json();
            app.Toast.show('Restored to version ' + data.restored_version + '. ' + data.skills_restored + ' skills restored.', 'success');
            window.location.reload();
        } catch (err) {
            app.Toast.show(err.message || 'Restore failed', 'error');
        }
    }

    app.initMarketplaceVersions = function(selector) {
        const root = document.querySelector(selector);
        if (!root) return;

        let activeTab = 'versions';
        const changelogLoaded = {};

        root.addEventListener('click', async function(e) {
            const tabBtn = e.target.closest('[data-tab]');
            if (tabBtn) {
                const newTab = tabBtn.getAttribute('data-tab');
                if (activeTab === newTab) return;
                activeTab = newTab;
                root.querySelectorAll('[data-tab]').forEach(function(btn) {
                    btn.className = btn.getAttribute('data-tab') === activeTab ? 'btn btn-primary' : 'btn btn-secondary';
                });
                document.getElementById('mv-versions-tab').style.display = activeTab === 'versions' ? '' : 'none';
                document.getElementById('mv-changelog-tab').style.display = activeTab === 'changelog' ? '' : 'none';
                if (activeTab === 'changelog') {
                    const selectedUserId = document.getElementById('mv-user-select').value;
                    if (selectedUserId && !changelogLoaded[selectedUserId]) {
                        await loadChangelog(selectedUserId);
                    }
                }
                return;
            }

            const toggleBtn = e.target.closest('[data-toggle-version]');
            if (toggleBtn && !e.target.closest('[data-restore-version]') && !e.target.closest('[data-compare-skill]')) {
                const card = toggleBtn.closest('.plugin-card');
                const details = card && card.querySelector('.plugin-details');
                if (details) {
                    const isHidden = details.style.display === 'none';
                    details.style.display = isHidden ? '' : 'none';
                    const icon = toggleBtn.querySelector('.expand-icon');
                    if (icon) icon.style.transform = isHidden ? 'rotate(180deg)' : '';
                    if (isHidden) {
                        const vid = toggleBtn.getAttribute('data-toggle-version');
                        const vUserId = toggleBtn.getAttribute('data-version-user');
                        loadVersionDetail(vid, vUserId, details);
                    }
                }
                return;
            }

            const compareBtn = e.target.closest('[data-compare-skill]');
            if (compareBtn) {
                e.stopPropagation();
                const skillId = compareBtn.getAttribute('data-compare-skill');
                const baseSkillId = compareBtn.getAttribute('data-base-skill');
                const compareVersionId = compareBtn.getAttribute('data-compare-version');
                const versionCard = compareBtn.closest('.version-card');
                const detailsEl = versionCard && versionCard.querySelector('.plugin-details');
                if (detailsEl) await loadCoreDiff(skillId, baseSkillId, compareVersionId, detailsEl);
                return;
            }

            if (e.target.closest('[data-close-diff]')) {
                e.stopPropagation();
                const diffVersionCard = e.target.closest('.version-card');
                const diffDetails = diffVersionCard && diffVersionCard.querySelector('.plugin-details');
                activeDiff = null;
                if (diffDetails) {
                    const diffVid = diffVersionCard.querySelector('[data-toggle-version]');
                    if (diffVid) renderVersionDetails(diffDetails, diffVid.getAttribute('data-toggle-version'));
                }
                return;
            }

            const restoreBtn = e.target.closest('[data-restore-version]');
            if (restoreBtn) {
                e.stopPropagation();
                await handleRestore(
                    restoreBtn.getAttribute('data-restore-version'),
                    restoreBtn.getAttribute('data-restore-num'),
                    restoreBtn.getAttribute('data-restore-user')
                );
                return;
            }
        });

        root.addEventListener('change', function(e) {
            if (e.target.id === 'mv-user-select') {
                const userId = e.target.value;
                const groups = root.querySelectorAll('.version-user-group');
                groups.forEach(function(group) {
                    const versions = group.querySelectorAll('[data-version-user]');
                    let hasMatch = !userId;
                    versions.forEach(function(v) {
                        if (v.getAttribute('data-version-user') === userId) hasMatch = true;
                    });
                    group.style.display = hasMatch ? '' : 'none';
                });
                if (activeTab === 'changelog' && userId) {
                    loadChangelog(userId);
                }
            }
        });

        async function loadChangelog(userId) {
            const container = document.getElementById('mv-changelog-tab');
            if (!container) return;
            if (!userId) {
                container.innerHTML = '<div class="empty-state"><p>Select a user to view changelog.</p></div>';
                return;
            }
            container.innerHTML = '<div class="loading-center"><div class="loading-spinner" role="status"><span class="sr-only">Loading...</span></div></div>';
            try {
                const changelog = await marketplaceApi(userId, '/changelog');
                changelogLoaded[userId] = true;
                if (!changelog || !changelog.length) {
                    container.innerHTML = '<div class="empty-state"><p>No changelog entries found for this user.</p></div>';
                    return;
                }
                const rows = changelog.map(function(entry) {
                    let actionClass = '';
                    switch(entry.action) {
                        case 'added': actionClass = 'badge-green'; break;
                        case 'updated': actionClass = 'badge-yellow'; break;
                        case 'deleted': actionClass = 'badge-red'; break;
                        case 'restored': actionClass = 'badge-blue'; break;
                        default: actionClass = 'badge-gray';
                    }
                    return '<tr>' +
                        '<td><span class="badge ' + actionClass + '">' + escapeHtml(entry.action) + '</span></td>' +
                        '<td><code style="background:var(--bg-surface-raised);padding:1px 4px;border-radius:var(--radius-xs);font-size:var(--text-xs)">' + escapeHtml(entry.skill_id) + '</code></td>' +
                        '<td>' + escapeHtml(entry.skill_name) + '</td>' +
                        '<td style="color:var(--text-secondary)">' + escapeHtml(entry.detail) + '</td>' +
                        '<td><span title="' + escapeHtml(app.formatDate(entry.created_at)) + '">' + escapeHtml(app.formatRelativeTime(entry.created_at)) + '</span></td>' +
                    '</tr>';
                }).join('');
                container.innerHTML = '<div class="table-container"><div class="table-scroll">' +
                    '<table class="data-table">' +
                        '<thead><tr><th>Action</th><th>Skill ID</th><th>Name</th><th>Detail</th><th>Time</th></tr></thead>' +
                        '<tbody>' + rows + '</tbody>' +
                    '</table>' +
                '</div></div>';
            } catch(err) {
                container.innerHTML = '<div class="empty-state"><p>Failed to load changelog.</p></div>';
            }
        }

        const urlUserId = new URLSearchParams(window.location.search).get('user_id');
        if (urlUserId) {
            const select = document.getElementById('mv-user-select');
            if (select) {
                select.value = urlUserId;
                select.dispatchEvent(new Event('change'));
            }
        }
    };
})(window.AdminApp);

(function(app) {
    'use strict';

    app.initOrgMarketplaces = function() {
        const searchInput = document.getElementById('mkt-search');
        const deptFilter = document.getElementById('mkt-dept-filter');
        const table = document.getElementById('mkt-table');

        if (window.AdminApp.OrgCommon) {
            AdminApp.OrgCommon.initExpandRows('#mkt-table');
        }

        // Build department lookup from embedded JSON data
        const mktDepts = {};
        document.querySelectorAll('script[data-marketplace-detail]').forEach(function(el) {
            try {
                const data = JSON.parse(el.textContent);
                const id = el.getAttribute('data-marketplace-detail');
                mktDepts[id] = (data.departments || [])
                    .filter(function(d) { return d.assigned; })
                    .map(function(d) { return d.name; });
            } catch (e) { /* skip */ }
        });

        function filterRows() {
            if (!table) return;
            const query = (searchInput ? searchInput.value : '').toLowerCase();
            const dept = deptFilter ? deptFilter.value : '';
            const rows = table.querySelectorAll('tbody tr.clickable-row');
            rows.forEach(function(row) {
                const name = row.getAttribute('data-name') || '';
                const entityId = row.getAttribute('data-entity-id') || '';
                const matchName = !query || name.indexOf(query) !== -1;
                const matchDept = !dept || (mktDepts[entityId] && mktDepts[entityId].indexOf(dept) !== -1);
                const visible = matchName && matchDept;
                row.style.display = visible ? '' : 'none';
                const detailRow = table.querySelector('tr[data-detail-for="' + entityId + '"]');
                if (detailRow && !visible) {
                    detailRow.classList.remove('visible');
                    detailRow.style.display = 'none';
                }
            });
        }

        if (searchInput) {
            let debounceTimer;
            searchInput.addEventListener('input', function() {
                clearTimeout(debounceTimer);
                debounceTimer = setTimeout(filterRows, 200);
            });
        }
        if (deptFilter) {
            deptFilter.addEventListener('change', filterRows);
        }

        app.events.on('click', '[data-toggle-json]', function(e, jsonBtn) {
            const id = jsonBtn.getAttribute('data-toggle-json');
            const container = document.querySelector('[data-json-container="' + id + '"]');
            if (container) {
                if (container.style.display === 'none') {
                    const dataEl = document.querySelector('script[data-marketplace-detail="' + id + '"]');
                    if (dataEl) {
                        try {
                            const data = JSON.parse(dataEl.textContent);
                            container.innerHTML = AdminApp.OrgCommon ? AdminApp.OrgCommon.formatJson(data) : '<pre>' + app.escapeHtml(JSON.stringify(data, null, 2)) + '</pre>';
                        } catch (err) {
                            container.innerHTML = '<p>Error parsing JSON</p>';
                        }
                    }
                    container.style.display = 'block';
                    jsonBtn.textContent = 'Hide JSON';
                } else {
                    container.style.display = 'none';
                    jsonBtn.textContent = 'Show JSON';
                }
            }
        });

        app.events.on('click', '.actions-trigger', function(e, trigger) {
            e.stopPropagation();
            const menu = trigger.closest('.actions-menu');
            const dd = menu ? menu.querySelector('.actions-dropdown') : null;
            if (dd) {
                const isOpen = dd.classList.contains('open');
                app.shared.closeAllMenus();
                if (!isOpen) dd.classList.add('open');
            }
        });

        app.events.on('click', '[data-delete-marketplace]', function(e, deleteBtn) {
            const id = deleteBtn.getAttribute('data-delete-marketplace');
            showDeleteConfirm(id);
        });

        app.events.on('click', '[data-copy-install-link]', function(e, btn) {
            var id = btn.getAttribute('data-copy-install-link');
            var siteUrl = window.location.origin;
            var installUrl = siteUrl + '/api/public/marketplace/org/' + encodeURIComponent(id) + '.git';
            navigator.clipboard.writeText(installUrl).then(function() {
                app.Toast.show('Install link copied to clipboard', 'success');
            }).catch(function() {
                var textarea = document.createElement('textarea');
                textarea.value = installUrl;
                document.body.appendChild(textarea);
                textarea.select();
                document.execCommand('copy');
                document.body.removeChild(textarea);
                app.Toast.show('Install link copied to clipboard', 'success');
            });
            app.shared.closeAllMenus();
        });

        app.events.on('click', '[data-sync-marketplace]', function(e, btn) {
            var id = btn.getAttribute('data-sync-marketplace');
            btn.disabled = true;
            var origText = btn.textContent;
            btn.textContent = 'Syncing...';
            app.shared.closeAllMenus();
            fetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id) + '/sync', {
                method: 'POST',
                credentials: 'include'
            })
            .then(function(resp) { return resp.json().then(function(data) { return { ok: resp.ok, data: data }; }); })
            .then(function(result) {
                if (result.ok) {
                    var msg = 'Sync completed: ' + (result.data.plugins_synced || 0) + ' plugins';
                    if (!result.data.changed) msg = 'Already up to date';
                    app.Toast.show(msg, 'success');
                    if (result.data.changed) setTimeout(function() { window.location.reload(); }, 1000);
                } else {
                    app.Toast.show(result.data.error || 'Sync failed', 'error');
                }
                btn.disabled = false;
                btn.textContent = origText;
            })
            .catch(function() {
                app.Toast.show('Network error during sync', 'error');
                btn.disabled = false;
                btn.textContent = origText;
            });
        });

        app.events.on('click', '[data-publish-marketplace]', function(e, btn) {
            var id = btn.getAttribute('data-publish-marketplace');
            app.shared.closeAllMenus();
            var overlay = document.createElement('div');
            overlay.className = 'confirm-overlay';
            overlay.innerHTML = '<div class="confirm-dialog">' +
                '<h3 style="margin:0 0 var(--space-3)">Publish to GitHub?</h3>' +
                '<p style="margin:0 0 var(--space-4);color:var(--text-secondary);font-size:var(--text-sm)">This will push the current marketplace plugins to the linked GitHub repository. Any remote changes will be overwritten.</p>' +
                '<div style="display:flex;gap:var(--space-3);justify-content:flex-end">' +
                    '<button class="btn btn-secondary" data-confirm-cancel>Cancel</button>' +
                    '<button class="btn btn-primary" data-confirm-publish>Publish</button>' +
                '</div>' +
            '</div>';
            document.body.appendChild(overlay);
            overlay.addEventListener('click', function(ev) {
                if (ev.target === overlay || ev.target.closest('[data-confirm-cancel]')) {
                    overlay.remove();
                    return;
                }
                var pubBtn = ev.target.closest('[data-confirm-publish]');
                if (pubBtn) {
                    pubBtn.disabled = true;
                    pubBtn.textContent = 'Publishing...';
                    fetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id) + '/publish', {
                        method: 'POST',
                        credentials: 'include'
                    })
                    .then(function(resp) { return resp.json().then(function(data) { return { ok: resp.ok, data: data }; }); })
                    .then(function(result) {
                        overlay.remove();
                        if (result.ok) {
                            var msg = 'Published: ' + (result.data.plugins_synced || 0) + ' plugins';
                            if (!result.data.changed) msg = 'No changes to publish';
                            app.Toast.show(msg, 'success');
                            if (result.data.changed) setTimeout(function() { window.location.reload(); }, 1000);
                        } else {
                            app.Toast.show(result.data.error || 'Publish failed', 'error');
                        }
                    })
                    .catch(function() {
                        overlay.remove();
                        app.Toast.show('Network error during publish', 'error');
                    });
                }
            });
        });

        initManagePluginsPanel();
        initEditPanel();
    };

    function showDeleteConfirm(marketplaceId) {
        const overlay = document.createElement('div');
        overlay.className = 'confirm-overlay';
        overlay.innerHTML = '<div class="confirm-dialog">' +
            '<h3 style="margin:0 0 var(--space-3)">Delete Marketplace?</h3>' +
            '<p style="margin:0 0 var(--space-2);color:var(--text-secondary);font-size:var(--text-sm)">You are about to delete <strong>' + app.escapeHtml(marketplaceId) + '</strong>.</p>' +
            '<p style="margin:0 0 var(--space-5);color:var(--text-secondary);font-size:var(--text-sm)">This will remove the marketplace and all plugin associations. This action cannot be undone.</p>' +
            '<div style="display:flex;gap:var(--space-3);justify-content:flex-end">' +
                '<button class="btn btn-secondary" data-confirm-cancel>Cancel</button>' +
                '<button class="btn btn-danger" data-confirm-delete="' + app.escapeHtml(marketplaceId) + '">Delete Marketplace</button>' +
            '</div>' +
        '</div>';
        document.body.appendChild(overlay);

        overlay.addEventListener('click', async function(e) {
            if (e.target === overlay || e.target.closest('[data-confirm-cancel]')) {
                overlay.remove();
                return;
            }
            const confirmBtn = e.target.closest('[data-confirm-delete]');
            if (confirmBtn) {
                const id = confirmBtn.getAttribute('data-confirm-delete');
                confirmBtn.disabled = true;
                confirmBtn.textContent = 'Deleting...';
                try {
                    const resp = await fetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id), {
                        method: 'DELETE',
                        credentials: 'include'
                    });
                    if (resp.ok) {
                        app.Toast.show('Marketplace deleted', 'success');
                        setTimeout(function() { window.location.reload(); }, 500);
                    } else {
                        const data = await resp.json().catch(function() { return {}; });
                        app.Toast.show(data.error || 'Failed to delete', 'error');
                    }
                } catch (err) {
                    app.Toast.show('Network error', 'error');
                }
                overlay.remove();
            }
        });
    }

    function initManagePluginsPanel() {
        if (!window.AdminApp.OrgCommon) return;
        const panelApi = AdminApp.OrgCommon.initSidePanel('mkt-panel');
        if (!panelApi) return;

        app.events.on('click', '[data-manage-plugins]', function(e, btn) {
            const id = btn.getAttribute('data-manage-plugins');

            const dataEl = document.querySelector('script[data-marketplace-detail="' + id + '"]');
            if (!dataEl) return;
            let mktData;
            try { mktData = JSON.parse(dataEl.textContent); } catch (err) { return; }

            panelApi.setTitle('Manage Plugins - ' + (mktData.name || id));

            fetch(app.API_BASE + '/plugins', { credentials: 'include' })
                .then(function(r) { return r.json(); })
                .then(function(allPlugins) {
                    const currentIds = {};
                    (mktData.plugin_ids || []).forEach(function(pid) { currentIds[pid] = true; });

                    let html = '<div class="assign-panel-checklist">';
                    if (!allPlugins.length) {
                        html += '<p style="color:var(--text-tertiary);font-size:var(--text-sm)">No plugins available.</p>';
                    } else {
                        allPlugins.forEach(function(p) {
                            const pid = p.id || p.plugin_id;
                            const pname = p.name || pid;
                            const checked = currentIds[pid] ? ' checked' : '';
                            html += '<label class="acl-checkbox-row" style="display:flex;align-items:center;gap:var(--space-2);padding:var(--space-1) 0;cursor:pointer">' +
                                '<input type="checkbox" name="plugin_id" value="' + app.escapeHtml(pid) + '"' + checked + '>' +
                                '<span>' + app.escapeHtml(pname) + '</span>' +
                                '</label>';
                        });
                    }
                    html += '</div>';
                    panelApi.setBody(html);
                    panelApi.setFooter(
                        '<button class="btn btn-secondary" data-panel-close>Cancel</button> ' +
                        '<button class="btn btn-primary" id="mkt-save-plugins">Save</button>'
                    );

                    const footer = panelApi.panel.querySelector('[data-panel-footer]');
                    if (footer) {
                        const cancelBtn = footer.querySelector('[data-panel-close]');
                        if (cancelBtn) cancelBtn.addEventListener('click', panelApi.close);
                    }

                    const saveBtn = document.getElementById('mkt-save-plugins');
                    if (saveBtn) {
                        saveBtn.addEventListener('click', async function() {
                            const checked = panelApi.panel.querySelectorAll('input[name="plugin_id"]:checked');
                            const ids = [];
                            checked.forEach(function(cb) { ids.push(cb.value); });
                            saveBtn.disabled = true;
                            saveBtn.textContent = 'Saving...';
                            try {
                                const resp = await fetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id) + '/plugins', {
                                    method: 'PUT',
                                    credentials: 'include',
                                    headers: { 'Content-Type': 'application/json' },
                                    body: JSON.stringify({ plugin_ids: ids })
                                });
                                if (resp.ok) {
                                    app.Toast.show('Plugins updated', 'success');
                                    panelApi.close();
                                    setTimeout(function() { window.location.reload(); }, 500);
                                } else {
                                    const data = await resp.json().catch(function() { return {}; });
                                    app.Toast.show(data.error || 'Failed to update', 'error');
                                    saveBtn.disabled = false;
                                    saveBtn.textContent = 'Save';
                                }
                            } catch (err) {
                                app.Toast.show('Network error', 'error');
                                saveBtn.disabled = false;
                                saveBtn.textContent = 'Save';
                            }
                        });
                    }

                    panelApi.open();
                })
                .catch(function() {
                    app.Toast.show('Failed to load plugins', 'error');
                });
        });
    }

    function initEditPanel() {
        if (!window.AdminApp.OrgCommon) return;
        const panelApi = AdminApp.OrgCommon.initSidePanel('mkt-edit-panel');
        if (!panelApi) return;

        function readJsonEl(id) {
            const el = document.getElementById(id);
            if (!el) return [];
            try { return JSON.parse(el.textContent); } catch (e) { return []; }
        }

        const allRoles = readJsonEl('mkt-all-roles');
        const allDepts = readJsonEl('mkt-all-departments');

        function openEdit(marketplaceId) {
            const isEdit = !!marketplaceId;
            let mktData = {};

            if (isEdit) {
                const dataEl = document.querySelector('script[data-marketplace-detail="' + marketplaceId + '"]');
                if (dataEl) {
                    try { mktData = JSON.parse(dataEl.textContent); } catch (e) { /* skip */ }
                }
            }

            panelApi.setTitle(isEdit ? 'Edit Marketplace' : 'Create Marketplace');

            fetch(app.API_BASE + '/plugins', { credentials: 'include' })
                .then(function(r) { return r.json(); })
                .then(function(allPlugins) {
                    const currentPluginIds = {};
                    (mktData.plugin_ids || []).forEach(function(pid) { currentPluginIds[pid] = true; });

                    const currentRoles = {};
                    (mktData.roles || []).forEach(function(r) {
                        if (r.assigned) currentRoles[r.name] = true;
                    });

                    const currentDepts = {};
                    const deptDefaults = {};
                    (mktData.departments || []).forEach(function(d) {
                        if (d.assigned) {
                            currentDepts[d.name] = true;
                            deptDefaults[d.name] = d.default_included;
                        }
                    });

                    let html = '<form id="panel-edit-form">';

                    if (!isEdit) {
                        html += '<div class="form-group">' +
                            '<label class="field-label">Marketplace ID</label>' +
                            '<input type="text" class="field-input" name="marketplace_id" required placeholder="e.g. my-marketplace">' +
                            '</div>';
                    }

                    html += '<div class="form-group">' +
                        '<label class="field-label">Name</label>' +
                        '<input type="text" class="field-input" name="name" required value="' + app.escapeHtml(mktData.name || '') + '">' +
                        '</div>';

                    html += '<div class="form-group">' +
                        '<label class="field-label">Description</label>' +
                        '<textarea class="field-input" name="description" rows="3">' + app.escapeHtml(mktData.description || '') + '</textarea>' +
                        '</div>';

                    html += '<div class="form-group">' +
                        '<label class="field-label">GitHub Repository URL</label>' +
                        '<input type="url" class="field-input" name="github_repo_url" placeholder="https://github.com/org/repo" value="' + app.escapeHtml(mktData.github_repo_url || '') + '">' +
                        '<span class="field-hint">Link to a GitHub repository to enable sync/publish</span>' +
                        '</div>';

                    // Roles
                    html += '<div class="form-group">' +
                        '<label class="field-label">Roles</label>' +
                        '<div style="display:flex;flex-wrap:wrap;gap:var(--space-1);padding:var(--space-2) 0">';
                    allRoles.forEach(function(r) {
                        const val = r.value || r;
                        const checked = currentRoles[val] ? ' checked' : '';
                        html += '<label style="display:inline-flex;align-items:center;gap:var(--space-2);margin-right:var(--space-3);font-size:var(--text-sm);cursor:pointer">' +
                            '<input type="checkbox" name="roles" value="' + app.escapeHtml(val) + '"' + checked + '> ' +
                            app.escapeHtml(val) + '</label>';
                    });
                    html += '</div></div>';

                    // Departments
                    html += '<div class="form-group">' +
                        '<label class="field-label">Departments</label>' +
                        '<div class="checklist-container" style="max-height:300px;overflow-y:auto;border:1px solid var(--border-subtle);border-radius:var(--radius-md);padding:var(--space-2)">' +
                        '<div class="checklist-item" style="display:flex;align-items:center;gap:var(--space-2);padding:var(--space-2);border-bottom:1px solid var(--border-subtle)">' +
                        '<input type="checkbox" id="panel-dept-check-all">' +
                        '<label for="panel-dept-check-all" style="flex:1;font-size:var(--text-sm);cursor:pointer;color:var(--text-primary);font-weight:600">Check all</label>' +
                        '</div>';
                    allDepts.forEach(function(d, i) {
                        const val = d.value || d.name || d;
                        const checked = currentDepts[val] ? ' checked' : '';
                        const defaultChecked = deptDefaults[val] ? ' checked' : '';
                        html += '<div class="checklist-item" style="display:flex;align-items:center;gap:var(--space-2);padding:var(--space-2)">' +
                            '<input type="checkbox" name="departments" value="' + app.escapeHtml(val) + '"' + checked + ' id="panel-dept-' + i + '">' +
                            '<label for="panel-dept-' + i + '" style="flex:1;font-size:var(--text-sm);cursor:pointer;color:var(--text-primary)">' + app.escapeHtml(val) + '</label>' +
                            '<span class="badge badge-gray" style="font-size:var(--text-xs)">' + (d.user_count || 0) + ' users</span>' +
                            '<label style="display:inline-flex;align-items:center;gap:4px;font-size:var(--text-xs);color:var(--text-secondary);cursor:pointer;white-space:nowrap">' +
                            '<input type="checkbox" name="dept_default_' + val + '"' + defaultChecked + '> Default</label>' +
                            '</div>';
                    });
                    html += '</div>' +
                        '<span class="field-hint" style="margin-top:var(--space-2);display:block">At least one department is required.</span>' +
                        '</div>';

                    // Plugins
                    html += '<div class="form-group">' +
                        '<label class="field-label">Plugins</label>' +
                        '<input type="text" class="field-input" placeholder="Filter plugins..." id="panel-plugin-filter" style="margin-bottom:var(--space-2)">' +
                        '<div class="checklist-container" style="max-height:200px;overflow-y:auto;border:1px solid var(--border-subtle);border-radius:var(--radius-md);padding:var(--space-2)">';
                    allPlugins.forEach(function(p, i) {
                        const pid = p.id || p.plugin_id;
                        const pname = p.name || pid;
                        const checked = currentPluginIds[pid] ? ' checked' : '';
                        html += '<div class="checklist-item" data-item-name="' + app.escapeHtml((pname).toLowerCase()) + '">' +
                            '<input type="checkbox" name="plugin_ids" value="' + app.escapeHtml(pid) + '"' + checked + ' id="panel-plugin-' + i + '">' +
                            '<label for="panel-plugin-' + i + '">' + app.escapeHtml(pname) + '</label>' +
                            '</div>';
                    });
                    html += '</div></div>';

                    html += '</form>';

                    panelApi.setBody(html);

                    let footerHtml = '<button class="btn btn-secondary" data-panel-close>Cancel</button> ' +
                        '<button class="btn btn-primary" id="mkt-edit-save">' + (isEdit ? 'Save Changes' : 'Create Marketplace') + '</button>';
                    if (isEdit) {
                        footerHtml = '<button class="btn btn-danger" id="mkt-edit-delete" style="margin-right:auto">Delete</button> ' + footerHtml;
                    }
                    panelApi.setFooter(footerHtml);

                    // Wire up cancel
                    const footer = panelApi.panel.querySelector('[data-panel-footer]');
                    if (footer) {
                        const cancelBtn = footer.querySelector('[data-panel-close]');
                        if (cancelBtn) cancelBtn.addEventListener('click', panelApi.close);
                    }

                    // Wire up check-all
                    const checkAll = document.getElementById('panel-dept-check-all');
                    if (checkAll) {
                        checkAll.addEventListener('change', function() {
                            const boxes = panelApi.panel.querySelectorAll('input[name="departments"]');
                            boxes.forEach(function(cb) { cb.checked = checkAll.checked; });
                        });
                        const boxes = panelApi.panel.querySelectorAll('input[name="departments"]');
                        let allChecked = boxes.length > 0;
                        boxes.forEach(function(cb) { if (!cb.checked) allChecked = false; });
                        checkAll.checked = allChecked;
                        panelApi.panel.addEventListener('change', function(e) {
                            if (e.target.name === 'departments') {
                                const boxes = panelApi.panel.querySelectorAll('input[name="departments"]');
                                let all = boxes.length > 0;
                                boxes.forEach(function(cb) { if (!cb.checked) all = false; });
                                checkAll.checked = all;
                            }
                        });
                    }

                    // Wire up plugin filter
                    const pluginFilter = document.getElementById('panel-plugin-filter');
                    if (pluginFilter) {
                        pluginFilter.addEventListener('input', function() {
                            const q = pluginFilter.value.toLowerCase();
                            panelApi.panel.querySelectorAll('.checklist-item[data-item-name]').forEach(function(item) {
                                const name = item.getAttribute('data-item-name') || '';
                                item.style.display = (!q || name.indexOf(q) !== -1) ? '' : 'none';
                            });
                        });
                    }

                    // Wire up save
                    const saveBtn = document.getElementById('mkt-edit-save');
                    if (saveBtn) {
                        saveBtn.addEventListener('click', function() {
                            handlePanelSave(isEdit, marketplaceId, saveBtn, panelApi);
                        });
                    }

                    // Wire up delete
                    const deleteBtn = document.getElementById('mkt-edit-delete');
                    if (deleteBtn) {
                        deleteBtn.addEventListener('click', function() {
                            panelApi.close();
                            showDeleteConfirm(marketplaceId);
                        });
                    }

                    panelApi.open();
                })
                .catch(function() {
                    app.Toast.show('Failed to load plugins', 'error');
                });
        }

        app.events.on('click', '[data-edit-marketplace]', function(e, btn) {
            openEdit(btn.getAttribute('data-edit-marketplace'));
        });

        app.events.on('click', '[data-create-marketplace]', function(e, btn) {
            e.preventDefault();
            openEdit(null);
        });
    }

    async function handlePanelSave(isEdit, marketplaceId, saveBtn, panelApi) {
        const form = document.getElementById('panel-edit-form');
        if (!form) return;

        const deptChecked = form.querySelectorAll('input[name="departments"]:checked');
        if (deptChecked.length === 0) {
            app.Toast.show('Please select at least one department', 'error');
            return;
        }

        saveBtn.disabled = true;
        saveBtn.textContent = 'Saving...';

        const pluginIds = [];
        form.querySelectorAll('input[name="plugin_ids"]:checked').forEach(function(cb) { pluginIds.push(cb.value); });

        const selectedRoles = [];
        form.querySelectorAll('input[name="roles"]:checked').forEach(function(cb) { selectedRoles.push(cb.value); });

        const deptRules = [];
        form.querySelectorAll('input[name="departments"]').forEach(function(cb) {
            if (cb.checked) {
                const defaultToggle = form.querySelector('input[name="dept_default_' + cb.value + '"]');
                deptRules.push({
                    rule_type: 'department',
                    rule_value: cb.value,
                    access: 'allow',
                    default_included: defaultToggle ? defaultToggle.checked : false
                });
            }
        });

        let aclRules = [];
        selectedRoles.forEach(function(role) {
            aclRules.push({ rule_type: 'role', rule_value: role, access: 'allow', default_included: false });
        });
        aclRules = aclRules.concat(deptRules);

        var githubUrlInput = form.querySelector('input[name="github_repo_url"]');
        var githubUrl = githubUrlInput ? githubUrlInput.value.trim() : '';

        try {
            if (isEdit) {
                const body = {
                    name: form.querySelector('input[name="name"]').value,
                    description: form.querySelector('textarea[name="description"]').value,
                    github_repo_url: githubUrl || null,
                    plugin_ids: pluginIds
                };
                const resp = await fetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(marketplaceId), {
                    method: 'PUT',
                    credentials: 'include',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(body)
                });
                if (resp.ok) {
                    await fetch(app.API_BASE + '/access-control/entity/marketplace/' + encodeURIComponent(marketplaceId), {
                        method: 'PUT',
                        credentials: 'include',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ rules: aclRules, sync_yaml: false })
                    });
                    app.Toast.show('Marketplace updated', 'success');
                    panelApi.close();
                    setTimeout(function() { window.location.reload(); }, 500);
                } else {
                    const data = await resp.json().catch(function() { return {}; });
                    app.Toast.show(data.error || 'Failed to update', 'error');
                }
            } else {
                const body = {
                    id: form.querySelector('input[name="marketplace_id"]').value,
                    name: form.querySelector('input[name="name"]').value,
                    description: form.querySelector('textarea[name="description"]').value,
                    github_repo_url: githubUrl || null,
                    plugin_ids: pluginIds
                };
                const resp = await fetch(app.API_BASE + '/org/marketplaces', {
                    method: 'POST',
                    credentials: 'include',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(body)
                });
                if (resp.ok || resp.status === 201) {
                    const created = await resp.json().catch(function() { return {}; });
                    const createdId = created.id || body.id;
                    if (aclRules.length > 0 && createdId) {
                        await fetch(app.API_BASE + '/access-control/entity/marketplace/' + encodeURIComponent(createdId), {
                            method: 'PUT',
                            credentials: 'include',
                            headers: { 'Content-Type': 'application/json' },
                            body: JSON.stringify({ rules: aclRules, sync_yaml: false })
                        });
                    }
                    app.Toast.show('Marketplace created', 'success');
                    panelApi.close();
                    setTimeout(function() { window.location.reload(); }, 500);
                } else {
                    const data = await resp.json().catch(function() { return {}; });
                    app.Toast.show(data.error || 'Failed to create', 'error');
                }
            }
        } catch (err) {
            app.Toast.show('Network error', 'error');
        }

        saveBtn.disabled = false;
        saveBtn.textContent = isEdit ? 'Save Changes' : 'Create Marketplace';
    }

    app.initMarketplaceEditForm = function() {
        const form = document.getElementById('marketplace-edit-form');
        if (!form) return;

        const isEdit = !!form.querySelector('input[name="marketplace_id"][readonly]');

        form.addEventListener('submit', async function(e) {
            e.preventDefault();
            const submitBtn = form.querySelector('button[type="submit"]');
            if (submitBtn) {
                submitBtn.disabled = true;
                submitBtn.textContent = 'Saving...';
            }

            const deptChecked = form.querySelectorAll('input[name="departments"]:checked');
            if (deptChecked.length === 0) {
                app.Toast.show('Please select at least one department', 'error');
                if (submitBtn) {
                    submitBtn.disabled = false;
                    submitBtn.textContent = isEdit ? 'Save Changes' : 'Create Marketplace';
                }
                return;
            }

            const pluginCheckboxes = form.querySelectorAll('input[name="plugin_ids"]:checked');
            const pluginIds = [];
            pluginCheckboxes.forEach(function(cb) { pluginIds.push(cb.value); });

            const roleCheckboxes = form.querySelectorAll('input[name="roles"]:checked');
            const selectedRoles = [];
            roleCheckboxes.forEach(function(cb) { selectedRoles.push(cb.value); });

            const deptCheckboxes = form.querySelectorAll('input[name="departments"]');
            const deptRules = [];
            deptCheckboxes.forEach(function(cb) {
                if (cb.checked) {
                    const defaultToggle = form.querySelector('input[name="dept_default_' + cb.value + '"]');
                    deptRules.push({
                        rule_type: 'department',
                        rule_value: cb.value,
                        access: 'allow',
                        default_included: defaultToggle ? defaultToggle.checked : false
                    });
                }
            });

            let aclRules = [];
            selectedRoles.forEach(function(role) {
                aclRules.push({ rule_type: 'role', rule_value: role, access: 'allow', default_included: false });
            });
            aclRules = aclRules.concat(deptRules);

            var formGithubInput = form.querySelector('input[name="github_repo_url"]');
            var formGithubUrl = formGithubInput ? formGithubInput.value.trim() : '';

            if (isEdit) {
                const id = form.querySelector('input[name="marketplace_id"]').value;
                const body = {
                    name: form.querySelector('input[name="name"]').value,
                    description: form.querySelector('textarea[name="description"]').value,
                    github_repo_url: formGithubUrl || null,
                    plugin_ids: pluginIds
                };

                try {
                    const resp = await fetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id), {
                        method: 'PUT',
                        credentials: 'include',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(body)
                    });
                    if (resp.ok) {
                        await fetch(app.API_BASE + '/access-control/entity/marketplace/' + encodeURIComponent(id), {
                            method: 'PUT',
                            credentials: 'include',
                            headers: { 'Content-Type': 'application/json' },
                            body: JSON.stringify({ rules: aclRules, sync_yaml: false })
                        });
                        app.Toast.show('Marketplace updated', 'success');
                        setTimeout(function() { window.location.href = '/admin/org/marketplaces/'; }, 500);
                    } else {
                        const data = await resp.json().catch(function() { return {}; });
                        app.Toast.show(data.error || 'Failed to update', 'error');
                    }
                } catch (err) {
                    app.Toast.show('Network error', 'error');
                }
            } else {
                const body = {
                    id: form.querySelector('input[name="marketplace_id"]').value,
                    name: form.querySelector('input[name="name"]').value,
                    description: form.querySelector('textarea[name="description"]').value,
                    github_repo_url: formGithubUrl || null,
                    plugin_ids: pluginIds
                };

                try {
                    const resp = await fetch(app.API_BASE + '/org/marketplaces', {
                        method: 'POST',
                        credentials: 'include',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(body)
                    });
                    if (resp.ok || resp.status === 201) {
                        const created = await resp.json().catch(function() { return {}; });
                        const createdId = created.id || body.id;
                        if (aclRules.length > 0 && createdId) {
                            await fetch(app.API_BASE + '/access-control/entity/marketplace/' + encodeURIComponent(createdId), {
                                method: 'PUT',
                                credentials: 'include',
                                headers: { 'Content-Type': 'application/json' },
                                body: JSON.stringify({ rules: aclRules, sync_yaml: false })
                            });
                        }
                        app.Toast.show('Marketplace created', 'success');
                        setTimeout(function() { window.location.href = '/admin/org/marketplaces/'; }, 500);
                    } else {
                        const data = await resp.json().catch(function() { return {}; });
                        app.Toast.show(data.error || 'Failed to create', 'error');
                    }
                } catch (err) {
                    app.Toast.show('Network error', 'error');
                }
            }

            if (submitBtn) {
                submitBtn.disabled = false;
                submitBtn.textContent = isEdit ? 'Save Changes' : 'Create Marketplace';
            }
        });

        const deleteBtn = document.getElementById('btn-delete-marketplace');
        if (deleteBtn) {
            deleteBtn.addEventListener('click', function() {
                const idInput = form.querySelector('input[name="marketplace_id"]');
                if (idInput) showDeleteConfirm(idInput.value);
            });
        }

        const checkAllDept = form.querySelector('#dept-check-all');
        if (checkAllDept) {
            checkAllDept.addEventListener('change', function() {
                const boxes = form.querySelectorAll('input[name="departments"]');
                boxes.forEach(function(cb) { cb.checked = checkAllDept.checked; });
            });
            form.addEventListener('change', function(e) {
                if (e.target.name === 'departments') {
                    const boxes = form.querySelectorAll('input[name="departments"]');
                    let allChecked = boxes.length > 0;
                    boxes.forEach(function(cb) { if (!cb.checked) allChecked = false; });
                    checkAllDept.checked = allChecked;
                }
            });
            // Sync check-all initial state to match current department selections
            const boxes = form.querySelectorAll('input[name="departments"]');
            let allChecked = boxes.length > 0;
            boxes.forEach(function(cb) { if (!cb.checked) allChecked = false; });
            checkAllDept.checked = allChecked;
        }
    };

})(window.AdminApp = window.AdminApp || {});
