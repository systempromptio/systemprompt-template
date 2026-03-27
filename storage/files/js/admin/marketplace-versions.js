(function(app) {
    'use strict';

    const escapeHtml = app.escapeHtml;
    const versionDetails = {};
    const diffCache = {};
    let activeDiff = null;

    function marketplaceApi(userId, path) {
        const url = '/api/public/marketplace/' + encodeURIComponent(userId) + path;
        return fetch(url, { headers: { 'Content-Type': 'application/json' } })
            .then((resp) => {
                if (!resp.ok) return resp.text().then((t) => { throw new Error(t || resp.statusText); });
                return resp.json();
            });
    }

    function renderSkillRow(skill, versionId) {
        const hasBase = skill.base_skill_id && skill.base_skill_id !== 'null';
        let compareBtn = '';
        if (hasBase) {
            const isActive = activeDiff && activeDiff.versionId === versionId && activeDiff.skillId === skill.skill_id;
            compareBtn = '<button class="btn btn-secondary btn-sm" data-compare-skill="' + escapeHtml(skill.skill_id) +
                '" data-compare-version="' + escapeHtml(versionId) +
                '" data-base-skill="' + escapeHtml(skill.base_skill_id) +
                '" style="font-size:var(--sp-text-xs);padding:2px 8px;white-space:nowrap"' +
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
                '<div style="font-size:var(--sp-text-xs);color:var(--sp-text-tertiary);margin-top:2px">' +
                    '<code style="background:var(--sp-bg-surface-raised);padding:1px 6px;border-radius:var(--sp-radius-xs)">' + escapeHtml(skill.skill_id) + '</code>' +
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
            metaDiff += '<div style="margin-bottom:var(--sp-space-2)"><strong>Name:</strong> <span class="diff-removed" style="padding:1px 4px">' + escapeHtml(coreSkill.name || '') + '</span> &rarr; <span class="diff-added" style="padding:1px 4px">' + escapeHtml(userSkill.name || '') + '</span></div>';
        }
        if ((userSkill.description || '') !== (coreSkill.description || '')) {
            metaDiff += '<div style="margin-bottom:var(--sp-space-2)"><strong>Description:</strong> <span class="diff-removed" style="padding:1px 4px">' + escapeHtml(coreSkill.description || '') + '</span> &rarr; <span class="diff-added" style="padding:1px 4px">' + escapeHtml(userSkill.description || '') + '</span></div>';
        }

        return '<div class="diff-panel">' +
            '<div class="diff-panel-header">' +
                '<h4 style="margin:0;font-size:var(--sp-text-sm);font-weight:600">Diff: ' + escapeHtml(userSkill.skill_id) + '</h4>' +
                '<div style="display:flex;gap:var(--sp-space-3);font-size:var(--sp-text-xs)">' +
                    '<span><span class="badge badge-blue">core</span> Base skill</span>' +
                    '<span><span class="badge badge-green">user</span> User version</span>' +
                '</div>' +
                '<button class="btn btn-secondary btn-sm" data-close-diff style="margin-left:auto;font-size:var(--sp-text-xs);padding:2px 8px">Close</button>' +
            '</div>' +
            (metaDiff ? '<div style="padding:var(--sp-space-3) var(--sp-space-4);border-bottom:1px solid var(--sp-border-subtle);font-size:var(--sp-text-sm)">' + metaDiff + '</div>' : '') +
            '<div class="diff-content">' + (diffHtml || '<div style="padding:var(--sp-space-4);color:var(--sp-text-tertiary);text-align:center">Content is identical</div>') + '</div>' +
        '</div>';
    }

    function renderVersionDetails(detailsContainer, versionId) {
        const detail = versionDetails[versionId];
        if (!detail || detail === 'loading') return;
        if (detail === 'error') {
            detailsContainer.innerHTML = '<div style="padding:var(--sp-space-4)"><div class="empty-state"><p>Failed to load version details.</p></div></div>';
            return;
        }
        let skills = [];
        if (Array.isArray(detail.skills_snapshot)) {
            skills = detail.skills_snapshot;
        } else if (typeof detail.skills_snapshot === 'string') {
            try { skills = JSON.parse(detail.skills_snapshot); } catch(e) { skills = []; }
        }
        const skillsHtml = skills.length
            ? skills.map((s) => renderSkillRow(s, versionId)).join('')
            : '<div class="empty-state" style="padding:var(--sp-space-4)"><p>No skills in this snapshot.</p></div>';

        let diffHtml = '';
        if (activeDiff && activeDiff.versionId === versionId && diffCache[activeDiff.cacheKey]) {
            const userSkill = skills.find((s) => s.skill_id === activeDiff.skillId);
            if (userSkill) diffHtml = renderDiffPanel(userSkill, diffCache[activeDiff.cacheKey]);
        }

        detailsContainer.innerHTML =
            '<div style="padding:var(--sp-space-4)">' +
                '<div style="font-size:var(--sp-text-sm);font-weight:600;margin-bottom:var(--sp-space-2);color:var(--sp-text-secondary)">Skills Snapshot (' + skills.length + ')</div>' +
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

    app.initMarketplaceVersions = (selector) => {
        const root = document.querySelector(selector);
        if (!root) return;

        let activeTab = 'versions';
        const changelogLoaded = {};

        root.addEventListener('click', async (e) => {
            const tabBtn = e.target.closest('[data-tab]');
            if (tabBtn) {
                const newTab = tabBtn.getAttribute('data-tab');
                if (activeTab === newTab) return;
                activeTab = newTab;
                root.querySelectorAll('[data-tab]').forEach((btn) => {
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
                const skillId = compareBtn.getAttribute('data-compare-skill');
                const baseSkillId = compareBtn.getAttribute('data-base-skill');
                const compareVersionId = compareBtn.getAttribute('data-compare-version');
                const versionCard = compareBtn.closest('.version-card');
                const detailsEl = versionCard && versionCard.querySelector('.plugin-details');
                if (detailsEl) await loadCoreDiff(skillId, baseSkillId, compareVersionId, detailsEl);
                return;
            }

            if (e.target.closest('[data-close-diff]')) {
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
                await handleRestore(
                    restoreBtn.getAttribute('data-restore-version'),
                    restoreBtn.getAttribute('data-restore-num'),
                    restoreBtn.getAttribute('data-restore-user')
                );
                return;
            }
        });

        root.addEventListener('change', (e) => {
            if (e.target.id === 'mv-user-select') {
                const userId = e.target.value;
                const groups = root.querySelectorAll('.version-user-group');
                groups.forEach((group) => {
                    const versions = group.querySelectorAll('[data-version-user]');
                    let hasMatch = !userId;
                    versions.forEach((v) => {
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
                const rows = changelog.map((entry) => {
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
                        '<td><code style="background:var(--sp-bg-surface-raised);padding:1px 4px;border-radius:var(--sp-radius-xs);font-size:var(--sp-text-xs)">' + escapeHtml(entry.skill_id) + '</code></td>' +
                        '<td>' + escapeHtml(entry.skill_name) + '</td>' +
                        '<td style="color:var(--sp-text-secondary)">' + escapeHtml(entry.detail) + '</td>' +
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
