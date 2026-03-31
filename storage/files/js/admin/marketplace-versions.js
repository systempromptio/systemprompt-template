(function(app) {
    'use strict';

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
        var hasBase = skill.base_skill_id && skill.base_skill_id !== 'null';

        var row = document.createElement('div');
        row.className = 'detail-item';

        var info = document.createElement('div');
        info.className = 'detail-item-info';

        var nameDiv = document.createElement('div');
        nameDiv.className = 'detail-item-name';
        nameDiv.append(document.createTextNode((skill.name || skill.skill_id) + ' '));

        var baseBadge = document.createElement('span');
        baseBadge.className = hasBase ? 'badge badge-yellow' : 'badge badge-gray';
        baseBadge.textContent = hasBase ? 'customized' : 'custom';
        nameDiv.append(baseBadge, document.createTextNode(' '));

        var enabledBadge = document.createElement('span');
        enabledBadge.className = skill.enabled === false ? 'badge badge-red' : 'badge badge-green';
        enabledBadge.textContent = skill.enabled === false ? 'disabled' : 'enabled';
        nameDiv.append(enabledBadge);

        var metaDiv = document.createElement('div');
        metaDiv.style.cssText = 'font-size:var(--sp-text-xs);color:var(--sp-text-tertiary);margin-top:2px';

        var codeEl = document.createElement('code');
        codeEl.style.cssText = 'background:var(--sp-bg-surface-raised);padding:1px 6px;border-radius:var(--sp-radius-xs)';
        codeEl.textContent = skill.skill_id;
        metaDiv.append(codeEl);

        if (skill.version) {
            var vSpan = document.createElement('span');
            vSpan.textContent = ' v' + skill.version;
            metaDiv.append(vSpan);
        }
        if (skill.description) {
            metaDiv.append(document.createTextNode(' \u2014 ' + app.shared.truncate(skill.description, 80)));
        }

        info.append(nameDiv, metaDiv);
        row.append(info);

        if (hasBase) {
            var isActive = activeDiff && activeDiff.versionId === versionId && activeDiff.skillId === skill.skill_id;
            var cmpBtn = document.createElement('button');
            cmpBtn.className = 'btn btn-secondary btn-sm';
            cmpBtn.setAttribute('data-compare-skill', skill.skill_id);
            cmpBtn.setAttribute('data-compare-version', versionId);
            cmpBtn.setAttribute('data-base-skill', skill.base_skill_id);
            cmpBtn.style.cssText = 'font-size:var(--sp-text-xs);padding:2px 8px;white-space:nowrap';
            if (isActive) cmpBtn.disabled = true;
            cmpBtn.textContent = isActive ? 'Viewing Diff' : 'Compare to Core';
            row.append(cmpBtn);
        }

        return row;
    }

    function renderDiffPanel(userSkill, coreSkill) {
        var userLines = (userSkill.content || '').split('\n');
        var coreLines = (coreSkill.content || '').split('\n');
        var maxLen = Math.max(userLines.length, coreLines.length);

        var panel = document.createElement('div');
        panel.className = 'diff-panel';

        var header = document.createElement('div');
        header.className = 'diff-panel-header';

        var h4 = document.createElement('h4');
        h4.style.cssText = 'margin:0;font-size:var(--sp-text-sm);font-weight:600';
        h4.textContent = 'Diff: ' + userSkill.skill_id;

        var legendDiv = document.createElement('div');
        legendDiv.style.cssText = 'display:flex;gap:var(--sp-space-3);font-size:var(--sp-text-xs)';

        var coreLabel = document.createElement('span');
        var coreBadge = document.createElement('span');
        coreBadge.className = 'badge badge-blue';
        coreBadge.textContent = 'core';
        coreLabel.append(coreBadge, document.createTextNode(' Base skill'));

        var userLabel = document.createElement('span');
        var userBadge = document.createElement('span');
        userBadge.className = 'badge badge-green';
        userBadge.textContent = 'user';
        userLabel.append(userBadge, document.createTextNode(' User version'));

        legendDiv.append(coreLabel, userLabel);

        var closeBtn = document.createElement('button');
        closeBtn.className = 'btn btn-secondary btn-sm';
        closeBtn.setAttribute('data-close-diff', '');
        closeBtn.style.cssText = 'margin-left:auto;font-size:var(--sp-text-xs);padding:2px 8px';
        closeBtn.textContent = 'Close';

        header.append(h4, legendDiv, closeBtn);
        panel.append(header);

        var hasMetaDiff = false;
        if ((userSkill.name || '') !== (coreSkill.name || '') || (userSkill.description || '') !== (coreSkill.description || '')) {
            hasMetaDiff = true;
            var metaSection = document.createElement('div');
            metaSection.style.cssText = 'padding:var(--sp-space-3) var(--sp-space-4);border-bottom:1px solid var(--sp-border-subtle);font-size:var(--sp-text-sm)';

            if ((userSkill.name || '') !== (coreSkill.name || '')) {
                var nameRow = document.createElement('div');
                nameRow.style.marginBottom = 'var(--sp-space-2)';
                var nameLabel = document.createElement('strong');
                nameLabel.textContent = 'Name:';
                var nameOld = document.createElement('span');
                nameOld.className = 'diff-removed';
                nameOld.style.padding = '1px 4px';
                nameOld.textContent = coreSkill.name || '';
                var nameNew = document.createElement('span');
                nameNew.className = 'diff-added';
                nameNew.style.padding = '1px 4px';
                nameNew.textContent = userSkill.name || '';
                nameRow.append(nameLabel, document.createTextNode(' '), nameOld, document.createTextNode(' \u2192 '), nameNew);
                metaSection.append(nameRow);
            }

            if ((userSkill.description || '') !== (coreSkill.description || '')) {
                var descRow = document.createElement('div');
                descRow.style.marginBottom = 'var(--sp-space-2)';
                var descLabel = document.createElement('strong');
                descLabel.textContent = 'Description:';
                var descOld = document.createElement('span');
                descOld.className = 'diff-removed';
                descOld.style.padding = '1px 4px';
                descOld.textContent = coreSkill.description || '';
                var descNew = document.createElement('span');
                descNew.className = 'diff-added';
                descNew.style.padding = '1px 4px';
                descNew.textContent = userSkill.description || '';
                descRow.append(descLabel, document.createTextNode(' '), descOld, document.createTextNode(' \u2192 '), descNew);
                metaSection.append(descRow);
            }

            panel.append(metaSection);
        }

        var diffContent = document.createElement('div');
        diffContent.className = 'diff-content';

        var hasDiff = false;
        for (var i = 0; i < maxLen; i++) {
            var coreLine = i < coreLines.length ? coreLines[i] : '';
            var userLine = i < userLines.length ? userLines[i] : '';
            var lineNum = i + 1;
            if (coreLine === userLine) {
                var unchangedLine = document.createElement('div');
                unchangedLine.className = 'diff-line diff-unchanged';
                var numSpan = document.createElement('span');
                numSpan.className = 'diff-linenum';
                numSpan.textContent = lineNum;
                var textSpan = document.createElement('span');
                textSpan.className = 'diff-text';
                textSpan.textContent = coreLine;
                unchangedLine.append(numSpan, textSpan);
                diffContent.append(unchangedLine);
                hasDiff = true;
            } else {
                if (coreLine) {
                    var removedLine = document.createElement('div');
                    removedLine.className = 'diff-line diff-removed';
                    var rNumSpan = document.createElement('span');
                    rNumSpan.className = 'diff-linenum';
                    rNumSpan.textContent = lineNum;
                    var rTextSpan = document.createElement('span');
                    rTextSpan.className = 'diff-text';
                    rTextSpan.textContent = '- ' + coreLine;
                    removedLine.append(rNumSpan, rTextSpan);
                    diffContent.append(removedLine);
                    hasDiff = true;
                }
                if (userLine) {
                    var addedLine = document.createElement('div');
                    addedLine.className = 'diff-line diff-added';
                    var aNumSpan = document.createElement('span');
                    aNumSpan.className = 'diff-linenum';
                    aNumSpan.textContent = lineNum;
                    var aTextSpan = document.createElement('span');
                    aTextSpan.className = 'diff-text';
                    aTextSpan.textContent = '+ ' + userLine;
                    addedLine.append(aNumSpan, aTextSpan);
                    diffContent.append(addedLine);
                    hasDiff = true;
                }
            }
        }

        if (!hasDiff) {
            var identicalMsg = document.createElement('div');
            identicalMsg.style.cssText = 'padding:var(--sp-space-4);color:var(--sp-text-tertiary);text-align:center';
            identicalMsg.textContent = 'Content is identical';
            diffContent.append(identicalMsg);
        }

        panel.append(diffContent);
        return panel;
    }

    function renderVersionDetails(detailsContainer, versionId) {
        var detail = versionDetails[versionId];
        if (!detail || detail === 'loading') return;
        detailsContainer.replaceChildren();
        if (detail === 'error') {
            var errWrap = document.createElement('div');
            errWrap.style.padding = 'var(--sp-space-4)';
            var errState = document.createElement('div');
            errState.className = 'empty-state';
            var errP = document.createElement('p');
            errP.textContent = 'Failed to load version details.';
            errState.append(errP);
            errWrap.append(errState);
            detailsContainer.append(errWrap);
            return;
        }
        var skills = [];
        if (Array.isArray(detail.skills_snapshot)) {
            skills = detail.skills_snapshot;
        } else if (typeof detail.skills_snapshot === 'string') {
            try { skills = JSON.parse(detail.skills_snapshot); } catch(e) { skills = []; }
        }

        var skillsWrap = document.createElement('div');
        skillsWrap.style.padding = 'var(--sp-space-4)';

        var skillsLabel = document.createElement('div');
        skillsLabel.style.cssText = 'font-size:var(--sp-text-sm);font-weight:600;margin-bottom:var(--sp-space-2);color:var(--sp-text-secondary)';
        skillsLabel.textContent = 'Skills Snapshot (' + skills.length + ')';
        skillsWrap.append(skillsLabel);

        if (skills.length) {
            skills.forEach(function(s) {
                skillsWrap.append(renderSkillRow(s, versionId));
            });
        } else {
            var emptyState = document.createElement('div');
            emptyState.className = 'empty-state';
            emptyState.style.padding = 'var(--sp-space-4)';
            var emptyP = document.createElement('p');
            emptyP.textContent = 'No skills in this snapshot.';
            emptyState.append(emptyP);
            skillsWrap.append(emptyState);
        }

        detailsContainer.append(skillsWrap);

        if (activeDiff && activeDiff.versionId === versionId && diffCache[activeDiff.cacheKey]) {
            var userSkill = skills.find(function(s) { return s.skill_id === activeDiff.skillId; });
            if (userSkill) detailsContainer.append(renderDiffPanel(userSkill, diffCache[activeDiff.cacheKey]));
        }
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

        function setEmptyState(container, message) {
            container.replaceChildren();
            var state = document.createElement('div');
            state.className = 'empty-state';
            var p = document.createElement('p');
            p.textContent = message;
            state.append(p);
            container.append(state);
        }

        async function loadChangelog(userId) {
            var container = document.getElementById('mv-changelog-tab');
            if (!container) return;
            if (!userId) {
                setEmptyState(container, 'Select a user to view changelog.');
                return;
            }
            container.replaceChildren();
            var loadingCenter = document.createElement('div');
            loadingCenter.className = 'loading-center';
            var spinner = document.createElement('div');
            spinner.className = 'loading-spinner';
            spinner.setAttribute('role', 'status');
            var srOnly = document.createElement('span');
            srOnly.className = 'sr-only';
            srOnly.textContent = 'Loading...';
            spinner.append(srOnly);
            loadingCenter.append(spinner);
            container.append(loadingCenter);
            try {
                var changelog = await marketplaceApi(userId, '/changelog');
                changelogLoaded[userId] = true;
                if (!changelog || !changelog.length) {
                    setEmptyState(container, 'No changelog entries found for this user.');
                    return;
                }
                container.replaceChildren();
                var tableContainer = document.createElement('div');
                tableContainer.className = 'table-container';
                var tableScroll = document.createElement('div');
                tableScroll.className = 'table-scroll';
                var table = document.createElement('table');
                table.className = 'data-table';
                var thead = document.createElement('thead');
                var headRow = document.createElement('tr');
                ['Action', 'Skill ID', 'Name', 'Detail', 'Time'].forEach(function(text) {
                    var th = document.createElement('th');
                    th.textContent = text;
                    headRow.append(th);
                });
                thead.append(headRow);
                var tbody = document.createElement('tbody');
                changelog.forEach(function(entry) {
                    var actionClass = 'badge-gray';
                    switch(entry.action) {
                        case 'added': actionClass = 'badge-green'; break;
                        case 'updated': actionClass = 'badge-yellow'; break;
                        case 'deleted': actionClass = 'badge-red'; break;
                        case 'restored': actionClass = 'badge-blue'; break;
                    }
                    var tr = document.createElement('tr');
                    var td1 = document.createElement('td');
                    var actionBadge = document.createElement('span');
                    actionBadge.className = 'badge ' + actionClass;
                    actionBadge.textContent = entry.action;
                    td1.append(actionBadge);

                    var td2 = document.createElement('td');
                    var codeEl = document.createElement('code');
                    codeEl.style.cssText = 'background:var(--sp-bg-surface-raised);padding:1px 4px;border-radius:var(--sp-radius-xs);font-size:var(--sp-text-xs)';
                    codeEl.textContent = entry.skill_id;
                    td2.append(codeEl);

                    var td3 = document.createElement('td');
                    td3.textContent = entry.skill_name;

                    var td4 = document.createElement('td');
                    td4.style.color = 'var(--sp-text-secondary)';
                    td4.textContent = entry.detail;

                    var td5 = document.createElement('td');
                    var timeSpan = document.createElement('span');
                    timeSpan.title = app.formatDate(entry.created_at);
                    timeSpan.textContent = app.formatRelativeTime(entry.created_at);
                    td5.append(timeSpan);

                    tr.append(td1, td2, td3, td4, td5);
                    tbody.append(tr);
                });
                table.append(thead, tbody);
                tableScroll.append(table);
                tableContainer.append(tableScroll);
                container.append(tableContainer);
            } catch(err) {
                setEmptyState(container, 'Failed to load changelog.');
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
