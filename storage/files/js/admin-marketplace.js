(function(app) {
    'use strict';

    let plugins = [];

    function showVisibilityModal(pluginId) {
        const plugin = plugins.find((p) => p.id === pluginId);
        if (!plugin) return;
        const rules = plugin.visibility_rules || [];

        const overlay = document.createElement('div');
        overlay.className = 'confirm-overlay';
        overlay.id = 'visibility-modal';

        const dialog = document.createElement('div');
        dialog.className = 'confirm-dialog';
        dialog.style.maxWidth = '500px';

        const heading = document.createElement('h3');
        heading.style.margin = '0 0 var(--sp-space-3)';
        heading.textContent = 'Edit Visibility - ' + plugin.name;

        const rulesList = document.createElement('div');
        rulesList.id = 'visibility-rules-list';
        renderRulesListDOM(rulesList, rules);

        const addSection = document.createElement('div');
        addSection.style.cssText = 'margin-top:var(--sp-space-4);padding-top:var(--sp-space-3);border-top:1px solid var(--sp-border-primary)';

        const addLabel = document.createElement('strong');
        addLabel.style.fontSize = 'var(--sp-text-sm)';
        addLabel.textContent = 'Add Rule';

        const addRow = document.createElement('div');
        addRow.style.cssText = 'display:flex;gap:var(--sp-space-2);margin-top:var(--sp-space-2);flex-wrap:wrap';

        const ruleTypeSelect = document.createElement('select');
        ruleTypeSelect.id = 'vis-rule-type';
        ruleTypeSelect.className = 'btn btn-secondary';
        ruleTypeSelect.style.cssText = 'cursor:pointer;font-size:var(--sp-text-sm)';
        const opt1 = document.createElement('option');
        opt1.value = 'department';
        opt1.textContent = 'Department';
        const opt2 = document.createElement('option');
        opt2.value = 'user';
        opt2.textContent = 'User';
        ruleTypeSelect.append(opt1, opt2);

        const ruleValueInput = document.createElement('input');
        ruleValueInput.type = 'text';
        ruleValueInput.id = 'vis-rule-value';
        ruleValueInput.className = 'search-input';
        ruleValueInput.placeholder = 'Value...';
        ruleValueInput.style.cssText = 'flex:1;min-width:120px;font-size:var(--sp-text-sm)';

        const ruleAccessSelect = document.createElement('select');
        ruleAccessSelect.id = 'vis-rule-access';
        ruleAccessSelect.className = 'btn btn-secondary';
        ruleAccessSelect.style.cssText = 'cursor:pointer;font-size:var(--sp-text-sm)';
        const optAllow = document.createElement('option');
        optAllow.value = 'allow';
        optAllow.textContent = 'Allow';
        const optDeny = document.createElement('option');
        optDeny.value = 'deny';
        optDeny.textContent = 'Deny';
        ruleAccessSelect.append(optAllow, optDeny);

        const addRuleBtn = document.createElement('button');
        addRuleBtn.className = 'btn btn-secondary';
        addRuleBtn.id = 'vis-add-rule';
        addRuleBtn.style.fontSize = 'var(--sp-text-sm)';
        addRuleBtn.textContent = 'Add';

        addRow.append(ruleTypeSelect, ruleValueInput, ruleAccessSelect, addRuleBtn);
        addSection.append(addLabel, addRow);

        const btnRow = document.createElement('div');
        btnRow.style.cssText = 'display:flex;gap:var(--sp-space-3);justify-content:flex-end;margin-top:var(--sp-space-4)';

        const cancelBtn = document.createElement('button');
        cancelBtn.className = 'btn btn-secondary';
        cancelBtn.setAttribute('data-confirm-cancel', '');
        cancelBtn.textContent = 'Cancel';

        const saveBtn = document.createElement('button');
        saveBtn.className = 'btn btn-primary';
        saveBtn.id = 'vis-save';
        saveBtn.textContent = 'Save';

        btnRow.append(cancelBtn, saveBtn);
        dialog.append(heading, rulesList, addSection, btnRow);
        overlay.append(dialog);

        document.body.append(overlay);

        const modalRules = rules.slice();

        function refreshRulesList() {
            const container = overlay.querySelector('#visibility-rules-list');
            if (container) renderRulesListDOM(container, modalRules);
        }

        overlay.addEventListener('click', async (e) => {
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

    function renderRulesListDOM(container, rules) {
        container.replaceChildren();
        if (!rules.length) {
            const emptyP = document.createElement('p');
            emptyP.style.cssText = 'font-size:var(--sp-text-sm);color:var(--sp-text-tertiary)';
            emptyP.textContent = 'No rules configured';
            container.append(emptyP);
            return;
        }
        rules.forEach(function(rule, idx) {
            const row = document.createElement('div');
            row.style.cssText = 'display:flex;align-items:center;gap:var(--sp-space-2);padding:var(--sp-space-1) 0;font-size:var(--sp-text-sm)';
            const badge = document.createElement('span');
            badge.className = 'badge ' + (rule.access === 'allow' ? 'badge-yellow' : 'badge-red');
            badge.textContent = rule.rule_type + ': ' + rule.rule_value + ' (' + rule.access + ')';
            const removeBtn = document.createElement('button');
            removeBtn.className = 'btn btn-danger';
            removeBtn.style.cssText = 'font-size:var(--sp-text-xs);padding:2px 6px';
            removeBtn.setAttribute('data-remove-rule', idx);
            removeBtn.textContent = 'Remove';
            row.append(badge, removeBtn);
            container.append(row);
        });
    }

    app.initMarketplace = (selector, pluginsData) => {
        const root = document.querySelector(selector);
        if (!root) return;
        plugins = pluginsData || [];

        const searchInput = document.getElementById('mkt-search');
        if (searchInput) {
            let debounceTimer = null;
            searchInput.addEventListener('input', () => {
                clearTimeout(debounceTimer);
                debounceTimer = setTimeout(() => {
                    const q = searchInput.value.toLowerCase().trim();
                    const cards = root.querySelectorAll('.plugin-card[data-plugin-id]');
                    for (let i = 0; i < cards.length; i++) {
                        const name = cards[i].getAttribute('data-search-name') || '';
                        const desc = cards[i].getAttribute('data-search-desc') || '';
                        const cat = cards[i].getAttribute('data-search-cat') || '';
                        cards[i].style.display = (!q || name.includes(q) || desc.includes(q) || cat.includes(q)) ? '' : 'none';
                    }
                }, 200);
            });
        }

        const sortSelect = document.getElementById('mkt-sort');
        if (sortSelect) {
            sortSelect.addEventListener('change', () => {
                const url = new URL(window.location.href);
                url.searchParams.set('sort', sortSelect.value);
                window.location.href = url.toString();
            });
        }

        root.addEventListener('click', async (e) => {
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
                showVisibilityModal(visBtn.getAttribute('data-edit-visibility'));
                return;
            }

            const loadUsersBtn = e.target.closest('[data-load-users]');
            if (loadUsersBtn) {
                const pluginId = loadUsersBtn.getAttribute('data-load-users');
                loadUsersBtn.disabled = true;
                loadUsersBtn.textContent = 'Loading...';
                try {
                    const usersData = await app.api('/marketplace-plugins/' + encodeURIComponent(pluginId) + '/users');
                    const users = usersData.users || usersData || [];
                    const container = root.querySelector('[data-users-for="' + pluginId + '"]');
                    if (container) {
                        container.replaceChildren();
                        if (users.length === 0) {
                            const noUsers = document.createElement('div');
                            noUsers.style.cssText = 'margin-top:var(--sp-space-2);font-size:var(--sp-text-xs);color:var(--sp-text-tertiary)';
                            noUsers.textContent = 'No users found';
                            container.append(noUsers);
                        } else {
                            const userList = document.createElement('div');
                            userList.style.cssText = 'margin-top:var(--sp-space-2);display:flex;flex-direction:column;gap:var(--sp-space-1)';
                            users.forEach(function(u) {
                                const userRow = document.createElement('div');
                                userRow.style.cssText = 'display:flex;align-items:center;gap:var(--sp-space-2);font-size:var(--sp-text-xs);padding:var(--sp-space-1) 0;border-bottom:1px solid var(--sp-border-primary)';
                                const nameSpan = document.createElement('span');
                                nameSpan.style.cssText = 'font-weight:600;color:var(--sp-text-primary)';
                                nameSpan.textContent = u.display_name || 'Unknown';
                                userRow.append(nameSpan);
                                if (u.department) {
                                    const deptBadge = document.createElement('span');
                                    deptBadge.className = 'badge badge-blue';
                                    deptBadge.textContent = u.department;
                                    userRow.append(deptBadge);
                                }
                                const eventBadge = document.createElement('span');
                                eventBadge.className = 'badge badge-gray';
                                eventBadge.textContent = (u.event_count || 0) + ' events';
                                userRow.append(eventBadge);
                                if (u.last_used) {
                                    const dateSpan = document.createElement('span');
                                    dateSpan.style.color = 'var(--sp-text-tertiary)';
                                    dateSpan.textContent = new Date(u.last_used).toLocaleDateString();
                                    userRow.append(dateSpan);
                                }
                                userList.append(userRow);
                            });
                            container.append(userList);
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

        const row = document.createElement('div');
        row.className = 'detail-item';

        const info = document.createElement('div');
        info.className = 'detail-item-info';

        const nameDiv = document.createElement('div');
        nameDiv.className = 'detail-item-name';
        nameDiv.append(document.createTextNode((skill.name || skill.skill_id) + ' '));

        const baseBadge = document.createElement('span');
        baseBadge.className = hasBase ? 'badge badge-yellow' : 'badge badge-gray';
        baseBadge.textContent = hasBase ? 'customized' : 'custom';
        nameDiv.append(baseBadge, document.createTextNode(' '));

        const enabledBadge = document.createElement('span');
        enabledBadge.className = skill.enabled === false ? 'badge badge-red' : 'badge badge-green';
        enabledBadge.textContent = skill.enabled === false ? 'disabled' : 'enabled';
        nameDiv.append(enabledBadge);

        const metaDiv = document.createElement('div');
        metaDiv.style.cssText = 'font-size:var(--sp-text-xs);color:var(--sp-text-tertiary);margin-top:2px';

        const codeEl = document.createElement('code');
        codeEl.style.cssText = 'background:var(--sp-bg-surface-raised);padding:1px 6px;border-radius:var(--sp-radius-xs)';
        codeEl.textContent = skill.skill_id;
        metaDiv.append(codeEl);

        if (skill.version) {
            const vSpan = document.createElement('span');
            vSpan.textContent = ' v' + skill.version;
            metaDiv.append(vSpan);
        }
        if (skill.description) {
            metaDiv.append(document.createTextNode(' \u2014 ' + app.shared.truncate(skill.description, 80)));
        }

        info.append(nameDiv, metaDiv);
        row.append(info);

        if (hasBase) {
            const isActive = activeDiff && activeDiff.versionId === versionId && activeDiff.skillId === skill.skill_id;
            const cmpBtn = document.createElement('button');
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
        const userLines = (userSkill.content || '').split('\n');
        const coreLines = (coreSkill.content || '').split('\n');
        const maxLen = Math.max(userLines.length, coreLines.length);

        const panel = document.createElement('div');
        panel.className = 'diff-panel';

        const header = document.createElement('div');
        header.className = 'diff-panel-header';

        const h4 = document.createElement('h4');
        h4.style.cssText = 'margin:0;font-size:var(--sp-text-sm);font-weight:600';
        h4.textContent = 'Diff: ' + userSkill.skill_id;

        const legendDiv = document.createElement('div');
        legendDiv.style.cssText = 'display:flex;gap:var(--sp-space-3);font-size:var(--sp-text-xs)';

        const coreLabel = document.createElement('span');
        const coreBadge = document.createElement('span');
        coreBadge.className = 'badge badge-blue';
        coreBadge.textContent = 'core';
        coreLabel.append(coreBadge, document.createTextNode(' Base skill'));

        const userLabel = document.createElement('span');
        const userBadge = document.createElement('span');
        userBadge.className = 'badge badge-green';
        userBadge.textContent = 'user';
        userLabel.append(userBadge, document.createTextNode(' User version'));

        legendDiv.append(coreLabel, userLabel);

        const closeBtn = document.createElement('button');
        closeBtn.className = 'btn btn-secondary btn-sm';
        closeBtn.setAttribute('data-close-diff', '');
        closeBtn.style.cssText = 'margin-left:auto;font-size:var(--sp-text-xs);padding:2px 8px';
        closeBtn.textContent = 'Close';

        header.append(h4, legendDiv, closeBtn);
        panel.append(header);

        let hasMetaDiff = false;
        if ((userSkill.name || '') !== (coreSkill.name || '') || (userSkill.description || '') !== (coreSkill.description || '')) {
            hasMetaDiff = true;
            const metaSection = document.createElement('div');
            metaSection.style.cssText = 'padding:var(--sp-space-3) var(--sp-space-4);border-bottom:1px solid var(--sp-border-subtle);font-size:var(--sp-text-sm)';

            if ((userSkill.name || '') !== (coreSkill.name || '')) {
                const nameRow = document.createElement('div');
                nameRow.style.marginBottom = 'var(--sp-space-2)';
                const nameLabel = document.createElement('strong');
                nameLabel.textContent = 'Name:';
                const nameOld = document.createElement('span');
                nameOld.className = 'diff-removed';
                nameOld.style.padding = '1px 4px';
                nameOld.textContent = coreSkill.name || '';
                const nameNew = document.createElement('span');
                nameNew.className = 'diff-added';
                nameNew.style.padding = '1px 4px';
                nameNew.textContent = userSkill.name || '';
                nameRow.append(nameLabel, document.createTextNode(' '), nameOld, document.createTextNode(' \u2192 '), nameNew);
                metaSection.append(nameRow);
            }

            if ((userSkill.description || '') !== (coreSkill.description || '')) {
                const descRow = document.createElement('div');
                descRow.style.marginBottom = 'var(--sp-space-2)';
                const descLabel = document.createElement('strong');
                descLabel.textContent = 'Description:';
                const descOld = document.createElement('span');
                descOld.className = 'diff-removed';
                descOld.style.padding = '1px 4px';
                descOld.textContent = coreSkill.description || '';
                const descNew = document.createElement('span');
                descNew.className = 'diff-added';
                descNew.style.padding = '1px 4px';
                descNew.textContent = userSkill.description || '';
                descRow.append(descLabel, document.createTextNode(' '), descOld, document.createTextNode(' \u2192 '), descNew);
                metaSection.append(descRow);
            }

            panel.append(metaSection);
        }

        const diffContent = document.createElement('div');
        diffContent.className = 'diff-content';

        let hasDiff = false;
        for (let i = 0; i < maxLen; i++) {
            const coreLine = i < coreLines.length ? coreLines[i] : '';
            const userLine = i < userLines.length ? userLines[i] : '';
            const lineNum = i + 1;
            if (coreLine === userLine) {
                const unchangedLine = document.createElement('div');
                unchangedLine.className = 'diff-line diff-unchanged';
                const numSpan = document.createElement('span');
                numSpan.className = 'diff-linenum';
                numSpan.textContent = lineNum;
                const textSpan = document.createElement('span');
                textSpan.className = 'diff-text';
                textSpan.textContent = coreLine;
                unchangedLine.append(numSpan, textSpan);
                diffContent.append(unchangedLine);
                hasDiff = true;
            } else {
                if (coreLine) {
                    const removedLine = document.createElement('div');
                    removedLine.className = 'diff-line diff-removed';
                    const rNumSpan = document.createElement('span');
                    rNumSpan.className = 'diff-linenum';
                    rNumSpan.textContent = lineNum;
                    const rTextSpan = document.createElement('span');
                    rTextSpan.className = 'diff-text';
                    rTextSpan.textContent = '- ' + coreLine;
                    removedLine.append(rNumSpan, rTextSpan);
                    diffContent.append(removedLine);
                    hasDiff = true;
                }
                if (userLine) {
                    const addedLine = document.createElement('div');
                    addedLine.className = 'diff-line diff-added';
                    const aNumSpan = document.createElement('span');
                    aNumSpan.className = 'diff-linenum';
                    aNumSpan.textContent = lineNum;
                    const aTextSpan = document.createElement('span');
                    aTextSpan.className = 'diff-text';
                    aTextSpan.textContent = '+ ' + userLine;
                    addedLine.append(aNumSpan, aTextSpan);
                    diffContent.append(addedLine);
                    hasDiff = true;
                }
            }
        }

        if (!hasDiff) {
            const identicalMsg = document.createElement('div');
            identicalMsg.style.cssText = 'padding:var(--sp-space-4);color:var(--sp-text-tertiary);text-align:center';
            identicalMsg.textContent = 'Content is identical';
            diffContent.append(identicalMsg);
        }

        panel.append(diffContent);
        return panel;
    }

    function renderVersionDetails(detailsContainer, versionId) {
        const detail = versionDetails[versionId];
        if (!detail || detail === 'loading') return;
        detailsContainer.replaceChildren();
        if (detail === 'error') {
            const errWrap = document.createElement('div');
            errWrap.style.padding = 'var(--sp-space-4)';
            const errState = document.createElement('div');
            errState.className = 'empty-state';
            const errP = document.createElement('p');
            errP.textContent = 'Failed to load version details.';
            errState.append(errP);
            errWrap.append(errState);
            detailsContainer.append(errWrap);
            return;
        }
        let skills = [];
        if (Array.isArray(detail.skills_snapshot)) {
            skills = detail.skills_snapshot;
        } else if (typeof detail.skills_snapshot === 'string') {
            try { skills = JSON.parse(detail.skills_snapshot); } catch(e) { skills = []; }
        }

        const skillsWrap = document.createElement('div');
        skillsWrap.style.padding = 'var(--sp-space-4)';

        const skillsLabel = document.createElement('div');
        skillsLabel.style.cssText = 'font-size:var(--sp-text-sm);font-weight:600;margin-bottom:var(--sp-space-2);color:var(--sp-text-secondary)';
        skillsLabel.textContent = 'Skills Snapshot (' + skills.length + ')';
        skillsWrap.append(skillsLabel);

        if (skills.length) {
            skills.forEach(function(s) {
                skillsWrap.append(renderSkillRow(s, versionId));
            });
        } else {
            const emptyState = document.createElement('div');
            emptyState.className = 'empty-state';
            emptyState.style.padding = 'var(--sp-space-4)';
            const emptyP = document.createElement('p');
            emptyP.textContent = 'No skills in this snapshot.';
            emptyState.append(emptyP);
            skillsWrap.append(emptyState);
        }

        detailsContainer.append(skillsWrap);

        if (activeDiff && activeDiff.versionId === versionId && diffCache[activeDiff.cacheKey]) {
            const userSkill = skills.find(function(s) { return s.skill_id === activeDiff.skillId; });
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
            const state = document.createElement('div');
            state.className = 'empty-state';
            const p = document.createElement('p');
            p.textContent = message;
            state.append(p);
            container.append(state);
        }

        async function loadChangelog(userId) {
            const container = document.getElementById('mv-changelog-tab');
            if (!container) return;
            if (!userId) {
                setEmptyState(container, 'Select a user to view changelog.');
                return;
            }
            container.replaceChildren();
            const loadingCenter = document.createElement('div');
            loadingCenter.className = 'loading-center';
            const spinner = document.createElement('div');
            spinner.className = 'loading-spinner';
            spinner.setAttribute('role', 'status');
            const srOnly = document.createElement('span');
            srOnly.className = 'sr-only';
            srOnly.textContent = 'Loading...';
            spinner.append(srOnly);
            loadingCenter.append(spinner);
            container.append(loadingCenter);
            try {
                const changelog = await marketplaceApi(userId, '/changelog');
                changelogLoaded[userId] = true;
                if (!changelog || !changelog.length) {
                    setEmptyState(container, 'No changelog entries found for this user.');
                    return;
                }
                container.replaceChildren();
                const tableContainer = document.createElement('div');
                tableContainer.className = 'table-container';
                const tableScroll = document.createElement('div');
                tableScroll.className = 'table-scroll';
                const table = document.createElement('table');
                table.className = 'data-table';
                const thead = document.createElement('thead');
                const headRow = document.createElement('tr');
                ['Action', 'Skill ID', 'Name', 'Detail', 'Time'].forEach(function(text) {
                    const th = document.createElement('th');
                    th.textContent = text;
                    headRow.append(th);
                });
                thead.append(headRow);
                const tbody = document.createElement('tbody');
                changelog.forEach(function(entry) {
                    let actionClass = 'badge-gray';
                    switch(entry.action) {
                        case 'added': actionClass = 'badge-green'; break;
                        case 'updated': actionClass = 'badge-yellow'; break;
                        case 'deleted': actionClass = 'badge-red'; break;
                        case 'restored': actionClass = 'badge-blue'; break;
                    }
                    const tr = document.createElement('tr');
                    const td1 = document.createElement('td');
                    const actionBadge = document.createElement('span');
                    actionBadge.className = 'badge ' + actionClass;
                    actionBadge.textContent = entry.action;
                    td1.append(actionBadge);

                    const td2 = document.createElement('td');
                    const codeEl = document.createElement('code');
                    codeEl.style.cssText = 'background:var(--sp-bg-surface-raised);padding:1px 4px;border-radius:var(--sp-radius-xs);font-size:var(--sp-text-xs)';
                    codeEl.textContent = entry.skill_id;
                    td2.append(codeEl);

                    const td3 = document.createElement('td');
                    td3.textContent = entry.skill_name;

                    const td4 = document.createElement('td');
                    td4.style.color = 'var(--sp-text-secondary)';
                    td4.textContent = entry.detail;

                    const td5 = document.createElement('td');
                    const timeSpan = document.createElement('span');
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

(function(app) {
    'use strict';

    const mktFetch = (url, opts = {}) => fetch(url, { credentials: 'include', ...opts });

    app.initOrgMarketplaces = () => {
        const searchInput = document.getElementById('mkt-search');
        const deptFilter = document.getElementById('mkt-dept-filter');
        const table = document.getElementById('mkt-table');

        if (window.AdminApp.OrgCommon) {
            AdminApp.OrgCommon.initExpandRows('#mkt-table');
        }

        const mktDepts = {};
        document.querySelectorAll('script[data-marketplace-detail]').forEach((el) => {
            try {
                const data = JSON.parse(el.textContent);
                const id = el.getAttribute('data-marketplace-detail');
                mktDepts[id] = (data.departments || [])
                    .filter((d) => d.assigned)
                    .map((d) => d.name);
            } catch (e) {}
        });

        const filterRows = () => {
            if (!table) return;
            const query = (searchInput ? searchInput.value : '').toLowerCase();
            const dept = deptFilter ? deptFilter.value : '';
            const rows = table.querySelectorAll('tbody tr.clickable-row');
            rows.forEach((row) => {
                const name = row.getAttribute('data-name') || '';
                const entityId = row.getAttribute('data-entity-id') || '';
                const matchName = !query || name.includes(query);
                const matchDept = !dept || (mktDepts[entityId] && mktDepts[entityId].includes(dept));
                const visible = matchName && matchDept;
                row.style.display = visible ? '' : 'none';
                const detailRow = table.querySelector('tr[data-detail-for="' + entityId + '"]');
                if (detailRow && !visible) {
                    detailRow.classList.remove('visible');
                    detailRow.style.display = 'none';
                }
            });
        };

        if (searchInput) {
            let debounceTimer;
            searchInput.addEventListener('input', () => {
                clearTimeout(debounceTimer);
                debounceTimer = setTimeout(filterRows, 200);
            });
        }
        if (deptFilter) {
            deptFilter.addEventListener('change', filterRows);
        }

        app.events.on('click', '[data-toggle-json]', (e, jsonBtn) => {
            const id = jsonBtn.getAttribute('data-toggle-json');
            const container = document.querySelector('[data-json-container="' + id + '"]');
            if (container) {
                if (container.style.display === 'none') {
                    const dataEl = document.querySelector('script[data-marketplace-detail="' + id + '"]');
                    if (dataEl) {
                        try {
                            const data = JSON.parse(dataEl.textContent);
                            container.replaceChildren();
                            if (AdminApp.OrgCommon) {
                                container.append(AdminApp.OrgCommon.formatJson(data));
                            } else {
                                const pre = document.createElement('pre');
                                pre.textContent = JSON.stringify(data, null, 2);
                                container.append(pre);
                            }
                        } catch (err) {
                            container.replaceChildren();
                            const errP = document.createElement('p');
                            errP.textContent = 'Error parsing JSON';
                            container.append(errP);
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

        app.events.on('click', '.actions-trigger', (e, trigger) => {
            const menu = trigger.closest('.actions-menu');
            const dd = menu ? menu.querySelector('.actions-dropdown') : null;
            if (dd) {
                const isOpen = dd.classList.contains('open');
                app.shared.closeAllMenus();
                if (!isOpen) dd.classList.add('open');
            }
        });

        app.events.on('click', '[data-delete-marketplace]', (e, deleteBtn) => {
            const id = deleteBtn.getAttribute('data-delete-marketplace');
            showDeleteConfirm(id);
        });

        app.events.on('click', '[data-copy-install-link]', (e, btn) => {
            const id = btn.getAttribute('data-copy-install-link');
            const siteUrl = window.location.origin;
            const installUrl = siteUrl + '/api/public/marketplace/org/' + encodeURIComponent(id) + '.git';
            navigator.clipboard.writeText(installUrl).then(() => {
                app.Toast.show('Install link copied to clipboard', 'success');
            }).catch(() => {
                const textarea = document.createElement('textarea');
                textarea.value = installUrl;
                document.body.append(textarea);
                textarea.select();
                document.execCommand('copy');
                textarea.remove();
                app.Toast.show('Install link copied to clipboard', 'success');
            });
            app.shared.closeAllMenus();
        });

        app.events.on('click', '[data-sync-marketplace]', (e, btn) => {
            const id = btn.getAttribute('data-sync-marketplace');
            btn.disabled = true;
            const origText = btn.textContent;
            btn.textContent = 'Syncing...';
            app.shared.closeAllMenus();
            mktFetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id) + '/sync', {
                method: 'POST'
            })
            .then((resp) => resp.json().then((data) => ({ ok: resp.ok, data: data })))
            .then((result) => {
                if (result.ok) {
                    let msg = 'Sync completed: ' + (result.data.plugins_synced || 0) + ' plugins';
                    if (!result.data.changed) msg = 'Already up to date';
                    app.Toast.show(msg, 'success');
                    if (result.data.changed) setTimeout(() => { window.location.reload(); }, 1000);
                } else {
                    app.Toast.show(result.data.error || 'Sync failed', 'error');
                }
                btn.disabled = false;
                btn.textContent = origText;
            })
            .catch(() => {
                app.Toast.show('Network error during sync', 'error');
                btn.disabled = false;
                btn.textContent = origText;
            });
        });

        app.events.on('click', '[data-publish-marketplace]', (e, btn) => {
            const id = btn.getAttribute('data-publish-marketplace');
            app.shared.closeAllMenus();
            const overlay = document.createElement('div');
            overlay.className = 'confirm-overlay';
            const pubDialog = document.createElement('div');
            pubDialog.className = 'confirm-dialog';
            const pubH3 = document.createElement('h3');
            pubH3.style.margin = '0 0 var(--sp-space-3)';
            pubH3.textContent = 'Publish to GitHub?';
            const pubP = document.createElement('p');
            pubP.style.cssText = 'margin:0 0 var(--sp-space-4);color:var(--sp-text-secondary);font-size:var(--sp-text-sm)';
            pubP.textContent = 'This will push the current marketplace plugins to the linked GitHub repository. Any remote changes will be overwritten.';
            const pubBtnRow = document.createElement('div');
            pubBtnRow.style.cssText = 'display:flex;gap:var(--sp-space-3);justify-content:flex-end';
            const pubCancelBtn = document.createElement('button');
            pubCancelBtn.className = 'btn btn-secondary';
            pubCancelBtn.setAttribute('data-confirm-cancel', '');
            pubCancelBtn.textContent = 'Cancel';
            const pubConfirmBtn = document.createElement('button');
            pubConfirmBtn.className = 'btn btn-primary';
            pubConfirmBtn.setAttribute('data-confirm-publish', '');
            pubConfirmBtn.textContent = 'Publish';
            pubBtnRow.append(pubCancelBtn, pubConfirmBtn);
            pubDialog.append(pubH3, pubP, pubBtnRow);
            overlay.append(pubDialog);
            document.body.append(overlay);
            overlay.addEventListener('click', (ev) => {
                if (ev.target === overlay || ev.target.closest('[data-confirm-cancel]')) {
                    overlay.remove();
                    return;
                }
                const pubBtn = ev.target.closest('[data-confirm-publish]');
                if (pubBtn) {
                    pubBtn.disabled = true;
                    pubBtn.textContent = 'Publishing...';
                    mktFetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id) + '/publish', {
                        method: 'POST'
                    })
                    .then((resp) => resp.json().then((data) => ({ ok: resp.ok, data: data })))
                    .then((result) => {
                        overlay.remove();
                        if (result.ok) {
                            let msg = 'Published: ' + (result.data.plugins_synced || 0) + ' plugins';
                            if (!result.data.changed) msg = 'No changes to publish';
                            app.Toast.show(msg, 'success');
                            if (result.data.changed) setTimeout(() => { window.location.reload(); }, 1000);
                        } else {
                            app.Toast.show(result.data.error || 'Publish failed', 'error');
                        }
                    })
                    .catch(() => {
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
        const delDialog = document.createElement('div');
        delDialog.className = 'confirm-dialog';
        const delH3 = document.createElement('h3');
        delH3.style.margin = '0 0 var(--sp-space-3)';
        delH3.textContent = 'Delete Marketplace?';
        const delP1 = document.createElement('p');
        delP1.style.cssText = 'margin:0 0 var(--sp-space-2);color:var(--sp-text-secondary);font-size:var(--sp-text-sm)';
        delP1.append(document.createTextNode('You are about to delete '));
        const delStrong = document.createElement('strong');
        delStrong.textContent = marketplaceId;
        delP1.append(delStrong, document.createTextNode('.'));
        const delP2 = document.createElement('p');
        delP2.style.cssText = 'margin:0 0 var(--sp-space-5);color:var(--sp-text-secondary);font-size:var(--sp-text-sm)';
        delP2.textContent = 'This will remove the marketplace and all plugin associations. This action cannot be undone.';
        const delBtnRow = document.createElement('div');
        delBtnRow.style.cssText = 'display:flex;gap:var(--sp-space-3);justify-content:flex-end';
        const delCancelBtn = document.createElement('button');
        delCancelBtn.className = 'btn btn-secondary';
        delCancelBtn.setAttribute('data-confirm-cancel', '');
        delCancelBtn.textContent = 'Cancel';
        const delConfirmBtn = document.createElement('button');
        delConfirmBtn.className = 'btn btn-danger';
        delConfirmBtn.setAttribute('data-confirm-delete', marketplaceId);
        delConfirmBtn.textContent = 'Delete Marketplace';
        delBtnRow.append(delCancelBtn, delConfirmBtn);
        delDialog.append(delH3, delP1, delP2, delBtnRow);
        overlay.append(delDialog);
        document.body.append(overlay);

        overlay.addEventListener('click', async (e) => {
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
                    const resp = await mktFetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id), {
                        method: 'DELETE'
                    });
                    if (resp.ok) {
                        app.Toast.show('Marketplace deleted', 'success');
                        setTimeout(() => { window.location.reload(); }, 500);
                    } else {
                        const data = await (resp.headers.get('content-type')?.includes('json') ? resp.json() : Promise.resolve({}));
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

        app.events.on('click', '[data-manage-plugins]', (e, btn) => {
            const id = btn.getAttribute('data-manage-plugins');

            const dataEl = document.querySelector('script[data-marketplace-detail="' + id + '"]');
            if (!dataEl) return;
            let mktData;
            try { mktData = JSON.parse(dataEl.textContent); } catch (err) { return; }

            panelApi.setTitle('Manage Plugins - ' + (mktData.name || id));

            mktFetch(app.API_BASE + '/plugins')
                .then((r) => r.json())
                .then((allPlugins) => {
                    const currentIds = {};
                    (mktData.plugin_ids || []).forEach((pid) => { currentIds[pid] = true; });

                    const checklist = document.createElement('div');
                    checklist.className = 'assign-panel-checklist';
                    if (!allPlugins.length) {
                        const noPlugins = document.createElement('p');
                        noPlugins.style.cssText = 'color:var(--sp-text-tertiary);font-size:var(--sp-text-sm)';
                        noPlugins.textContent = 'No plugins available.';
                        checklist.append(noPlugins);
                    } else {
                        allPlugins.forEach(function(p) {
                            const pid = p.id || p.plugin_id;
                            const pname = p.name || pid;
                            const label = document.createElement('label');
                            label.className = 'acl-checkbox-row';
                            label.style.cssText = 'display:flex;align-items:center;gap:var(--sp-space-2);padding:var(--sp-space-1) 0;cursor:pointer';
                            const cb = document.createElement('input');
                            cb.type = 'checkbox';
                            cb.name = 'plugin_id';
                            cb.value = pid;
                            if (currentIds[pid]) cb.checked = true;
                            const nameSpan = document.createElement('span');
                            nameSpan.textContent = pname;
                            label.append(cb, nameSpan);
                            checklist.append(label);
                        });
                    }
                    panelApi.setBodyDom(checklist);

                    const footerFrag = document.createDocumentFragment();
                    const panelCancelBtn = document.createElement('button');
                    panelCancelBtn.className = 'btn btn-secondary';
                    panelCancelBtn.setAttribute('data-panel-close', '');
                    panelCancelBtn.textContent = 'Cancel';
                    const panelSaveBtn = document.createElement('button');
                    panelSaveBtn.className = 'btn btn-primary';
                    panelSaveBtn.id = 'mkt-save-plugins';
                    panelSaveBtn.textContent = 'Save';
                    footerFrag.append(panelCancelBtn, document.createTextNode(' '), panelSaveBtn);
                    panelApi.setFooterDom(footerFrag);

                    const footer = panelApi.panel.querySelector('[data-panel-footer]');
                    if (footer) {
                        const cancelBtn = footer.querySelector('[data-panel-close]');
                        if (cancelBtn) cancelBtn.addEventListener('click', panelApi.close);
                    }

                    const saveBtn = document.getElementById('mkt-save-plugins');
                    if (saveBtn) {
                        saveBtn.addEventListener('click', async () => {
                            const checked = panelApi.panel.querySelectorAll('input[name="plugin_id"]:checked');
                            const ids = [];
                            checked.forEach((cb) => { ids.push(cb.value); });
                            saveBtn.disabled = true;
                            saveBtn.textContent = 'Saving...';
                            try {
                                const resp = await mktFetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id) + '/plugins', {
                                    method: 'PUT',
                                    headers: { 'Content-Type': 'application/json' },
                                    body: JSON.stringify({ plugin_ids: ids })
                                });
                                if (resp.ok) {
                                    app.Toast.show('Plugins updated', 'success');
                                    panelApi.close();
                                    setTimeout(() => { window.location.reload(); }, 500);
                                } else {
                                    const data = await (resp.headers.get('content-type')?.includes('json') ? resp.json() : Promise.resolve({}));
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
                .catch(() => {
                    app.Toast.show('Failed to load plugins', 'error');
                });
        });
    }

    function initEditPanel() {
        if (!window.AdminApp.OrgCommon) return;
        const panelApi = AdminApp.OrgCommon.initSidePanel('mkt-edit-panel');
        if (!panelApi) return;

        const readJsonEl = (id) => {
            const el = document.getElementById(id);
            if (!el) return [];
            try { return JSON.parse(el.textContent); } catch (e) { return []; }
        };

        const allRoles = readJsonEl('mkt-all-roles');
        const allDepts = readJsonEl('mkt-all-departments');

        const openEdit = (marketplaceId) => {
            const isEdit = !!marketplaceId;
            let mktData = {};

            if (isEdit) {
                const dataEl = document.querySelector('script[data-marketplace-detail="' + marketplaceId + '"]');
                if (dataEl) {
                    try { mktData = JSON.parse(dataEl.textContent); } catch (e) {}
                }
            }

            panelApi.setTitle(isEdit ? 'Edit Marketplace' : 'Create Marketplace');

            mktFetch(app.API_BASE + '/plugins')
                .then((r) => r.json())
                .then((allPlugins) => {
                    const currentPluginIds = {};
                    (mktData.plugin_ids || []).forEach((pid) => { currentPluginIds[pid] = true; });

                    const currentRoles = {};
                    (mktData.roles || []).forEach((r) => {
                        if (r.assigned) currentRoles[r.name] = true;
                    });

                    const currentDepts = {};
                    const deptDefaults = {};
                    (mktData.departments || []).forEach((d) => {
                        if (d.assigned) {
                            currentDepts[d.name] = true;
                            deptDefaults[d.name] = d.default_included;
                        }
                    });

                    const form = document.createElement('form');
                    form.id = 'panel-edit-form';

                    function makeFormGroup(labelText, inputEl) {
                        const group = document.createElement('div');
                        group.className = 'form-group';
                        const lbl = document.createElement('label');
                        lbl.className = 'field-label';
                        lbl.textContent = labelText;
                        group.append(lbl, inputEl);
                        return group;
                    }

                    if (!isEdit) {
                        const idInput = document.createElement('input');
                        idInput.type = 'text';
                        idInput.className = 'field-input';
                        idInput.name = 'marketplace_id';
                        idInput.required = true;
                        idInput.placeholder = 'e.g. my-marketplace';
                        form.append(makeFormGroup('Marketplace ID', idInput));
                    }

                    const nameInput = document.createElement('input');
                    nameInput.type = 'text';
                    nameInput.className = 'field-input';
                    nameInput.name = 'name';
                    nameInput.required = true;
                    nameInput.value = mktData.name || '';
                    form.append(makeFormGroup('Name', nameInput));

                    const descTextarea = document.createElement('textarea');
                    descTextarea.className = 'field-input';
                    descTextarea.name = 'description';
                    descTextarea.rows = 3;
                    descTextarea.textContent = mktData.description || '';
                    form.append(makeFormGroup('Description', descTextarea));

                    const ghGroup = document.createElement('div');
                    ghGroup.className = 'form-group';
                    const ghLabel = document.createElement('label');
                    ghLabel.className = 'field-label';
                    ghLabel.textContent = 'GitHub Repository URL';
                    const ghInput = document.createElement('input');
                    ghInput.type = 'url';
                    ghInput.className = 'field-input';
                    ghInput.name = 'github_repo_url';
                    ghInput.placeholder = 'https://github.com/org/repo';
                    ghInput.value = mktData.github_repo_url || '';
                    const ghHint = document.createElement('span');
                    ghHint.className = 'field-hint';
                    ghHint.textContent = 'Link to a GitHub repository to enable sync/publish';
                    ghGroup.append(ghLabel, ghInput, ghHint);
                    form.append(ghGroup);

                    const rolesGroup = document.createElement('div');
                    rolesGroup.className = 'form-group';
                    const rolesLabel = document.createElement('label');
                    rolesLabel.className = 'field-label';
                    rolesLabel.textContent = 'Roles';
                    const rolesWrap = document.createElement('div');
                    rolesWrap.style.cssText = 'display:flex;flex-wrap:wrap;gap:var(--sp-space-1);padding:var(--sp-space-2) 0';
                    allRoles.forEach(function(r) {
                        const val = r.value || r;
                        const rLabel = document.createElement('label');
                        rLabel.style.cssText = 'display:inline-flex;align-items:center;gap:var(--sp-space-2);margin-right:var(--sp-space-3);font-size:var(--sp-text-sm);cursor:pointer';
                        const rCb = document.createElement('input');
                        rCb.type = 'checkbox';
                        rCb.name = 'roles';
                        rCb.value = val;
                        if (currentRoles[val]) rCb.checked = true;
                        rLabel.append(rCb, document.createTextNode(' ' + val));
                        rolesWrap.append(rLabel);
                    });
                    rolesGroup.append(rolesLabel, rolesWrap);
                    form.append(rolesGroup);

                    const deptGroup = document.createElement('div');
                    deptGroup.className = 'form-group';
                    const deptLabel = document.createElement('label');
                    deptLabel.className = 'field-label';
                    deptLabel.textContent = 'Departments';
                    const deptContainer = document.createElement('div');
                    deptContainer.className = 'checklist-container';
                    deptContainer.style.cssText = 'max-height:300px;overflow-y:auto;border:1px solid var(--sp-border-subtle);border-radius:var(--sp-radius-md);padding:var(--sp-space-2)';

                    const checkAllItem = document.createElement('div');
                    checkAllItem.className = 'checklist-item';
                    checkAllItem.style.cssText = 'display:flex;align-items:center;gap:var(--sp-space-2);padding:var(--sp-space-2);border-bottom:1px solid var(--sp-border-subtle)';
                    const checkAllCb = document.createElement('input');
                    checkAllCb.type = 'checkbox';
                    checkAllCb.id = 'panel-dept-check-all';
                    const checkAllLabel = document.createElement('label');
                    checkAllLabel.htmlFor = 'panel-dept-check-all';
                    checkAllLabel.style.cssText = 'flex:1;font-size:var(--sp-text-sm);cursor:pointer;color:var(--sp-text-primary);font-weight:600';
                    checkAllLabel.textContent = 'Check all';
                    checkAllItem.append(checkAllCb, checkAllLabel);
                    deptContainer.append(checkAllItem);

                    allDepts.forEach(function(d, i) {
                        const val = d.value || d.name || d;
                        const dItem = document.createElement('div');
                        dItem.className = 'checklist-item';
                        dItem.style.cssText = 'display:flex;align-items:center;gap:var(--sp-space-2);padding:var(--sp-space-2)';
                        const dCb = document.createElement('input');
                        dCb.type = 'checkbox';
                        dCb.name = 'departments';
                        dCb.value = val;
                        if (currentDepts[val]) dCb.checked = true;
                        dCb.id = 'panel-dept-' + i;
                        const dLabel = document.createElement('label');
                        dLabel.htmlFor = 'panel-dept-' + i;
                        dLabel.style.cssText = 'flex:1;font-size:var(--sp-text-sm);cursor:pointer;color:var(--sp-text-primary)';
                        dLabel.textContent = val;
                        const countBadge = document.createElement('span');
                        countBadge.className = 'badge badge-gray';
                        countBadge.style.fontSize = 'var(--sp-text-xs)';
                        countBadge.textContent = (d.user_count || 0) + ' users';
                        const defaultLabel = document.createElement('label');
                        defaultLabel.style.cssText = 'display:inline-flex;align-items:center;gap:4px;font-size:var(--sp-text-xs);color:var(--sp-text-secondary);cursor:pointer;white-space:nowrap';
                        const defaultCb = document.createElement('input');
                        defaultCb.type = 'checkbox';
                        defaultCb.name = 'dept_default_' + val;
                        if (deptDefaults[val]) defaultCb.checked = true;
                        defaultLabel.append(defaultCb, document.createTextNode(' Default'));
                        dItem.append(dCb, dLabel, countBadge, defaultLabel);
                        deptContainer.append(dItem);
                    });

                    const deptHint = document.createElement('span');
                    deptHint.className = 'field-hint';
                    deptHint.style.cssText = 'margin-top:var(--sp-space-2);display:block';
                    deptHint.textContent = 'At least one department is required.';
                    deptGroup.append(deptLabel, deptContainer, deptHint);
                    form.append(deptGroup);

                    const pluginGroup = document.createElement('div');
                    pluginGroup.className = 'form-group';
                    const pluginLabel = document.createElement('label');
                    pluginLabel.className = 'field-label';
                    pluginLabel.textContent = 'Plugins';
                    const pluginFilterInput = document.createElement('input');
                    pluginFilterInput.type = 'text';
                    pluginFilterInput.className = 'field-input';
                    pluginFilterInput.placeholder = 'Filter plugins...';
                    pluginFilterInput.id = 'panel-plugin-filter';
                    pluginFilterInput.style.marginBottom = 'var(--sp-space-2)';
                    const pluginContainer = document.createElement('div');
                    pluginContainer.className = 'checklist-container';
                    pluginContainer.style.cssText = 'max-height:200px;overflow-y:auto;border:1px solid var(--sp-border-subtle);border-radius:var(--sp-radius-md);padding:var(--sp-space-2)';
                    allPlugins.forEach(function(p, i) {
                        const pid = p.id || p.plugin_id;
                        const pname = p.name || pid;
                        const pItem = document.createElement('div');
                        pItem.className = 'checklist-item';
                        pItem.setAttribute('data-item-name', pname.toLowerCase());
                        const pCb = document.createElement('input');
                        pCb.type = 'checkbox';
                        pCb.name = 'plugin_ids';
                        pCb.value = pid;
                        if (currentPluginIds[pid]) pCb.checked = true;
                        pCb.id = 'panel-plugin-' + i;
                        const pLabel = document.createElement('label');
                        pLabel.htmlFor = 'panel-plugin-' + i;
                        pLabel.textContent = pname;
                        pItem.append(pCb, pLabel);
                        pluginContainer.append(pItem);
                    });
                    pluginGroup.append(pluginLabel, pluginFilterInput, pluginContainer);
                    form.append(pluginGroup);

                    panelApi.setBodyDom(form);

                    const editFooter = document.createDocumentFragment();
                    if (isEdit) {
                        const editDelBtn = document.createElement('button');
                        editDelBtn.className = 'btn btn-danger';
                        editDelBtn.id = 'mkt-edit-delete';
                        editDelBtn.style.marginRight = 'auto';
                        editDelBtn.textContent = 'Delete';
                        editFooter.append(editDelBtn, document.createTextNode(' '));
                    }
                    const editCancelBtn = document.createElement('button');
                    editCancelBtn.className = 'btn btn-secondary';
                    editCancelBtn.setAttribute('data-panel-close', '');
                    editCancelBtn.textContent = 'Cancel';
                    const editSaveBtn = document.createElement('button');
                    editSaveBtn.className = 'btn btn-primary';
                    editSaveBtn.id = 'mkt-edit-save';
                    editSaveBtn.textContent = isEdit ? 'Save Changes' : 'Create Marketplace';
                    editFooter.append(editCancelBtn, document.createTextNode(' '), editSaveBtn);
                    panelApi.setFooterDom(editFooter);

                    const footer = panelApi.panel.querySelector('[data-panel-footer]');
                    if (footer) {
                        const cancelBtn = footer.querySelector('[data-panel-close]');
                        if (cancelBtn) cancelBtn.addEventListener('click', panelApi.close);
                    }

                    const checkAll = document.getElementById('panel-dept-check-all');
                    if (checkAll) {
                        checkAll.addEventListener('change', () => {
                            const boxes = panelApi.panel.querySelectorAll('input[name="departments"]');
                            boxes.forEach((cb) => { cb.checked = checkAll.checked; });
                        });
                        const boxes = panelApi.panel.querySelectorAll('input[name="departments"]');
                        let allChecked = boxes.length > 0;
                        boxes.forEach((cb) => { if (!cb.checked) allChecked = false; });
                        checkAll.checked = allChecked;
                        panelApi.panel.addEventListener('change', (e) => {
                            if (e.target.name === 'departments') {
                                const boxes = panelApi.panel.querySelectorAll('input[name="departments"]');
                                let all = boxes.length > 0;
                                boxes.forEach((cb) => { if (!cb.checked) all = false; });
                                checkAll.checked = all;
                            }
                        });
                    }

                    const pluginFilter = document.getElementById('panel-plugin-filter');
                    if (pluginFilter) {
                        pluginFilter.addEventListener('input', () => {
                            const q = pluginFilter.value.toLowerCase();
                            panelApi.panel.querySelectorAll('.checklist-item[data-item-name]').forEach((item) => {
                                const name = item.getAttribute('data-item-name') || '';
                                item.style.display = (!q || name.includes(q)) ? '' : 'none';
                            });
                        });
                    }

                    const saveBtn = document.getElementById('mkt-edit-save');
                    if (saveBtn) {
                        saveBtn.addEventListener('click', () => {
                            handlePanelSave(isEdit, marketplaceId, saveBtn, panelApi);
                        });
                    }

                    const deleteBtn = document.getElementById('mkt-edit-delete');
                    if (deleteBtn) {
                        deleteBtn.addEventListener('click', () => {
                            panelApi.close();
                            showDeleteConfirm(marketplaceId);
                        });
                    }

                    panelApi.open();
                })
                .catch(() => {
                    app.Toast.show('Failed to load plugins', 'error');
                });
        };

        app.events.on('click', '[data-edit-marketplace]', (e, btn) => {
            openEdit(btn.getAttribute('data-edit-marketplace'));
        });

        app.events.on('click', '[data-create-marketplace]', (e, btn) => {
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
        form.querySelectorAll('input[name="plugin_ids"]:checked').forEach((cb) => { pluginIds.push(cb.value); });

        const selectedRoles = [];
        form.querySelectorAll('input[name="roles"]:checked').forEach((cb) => { selectedRoles.push(cb.value); });

        const deptRules = [];
        form.querySelectorAll('input[name="departments"]').forEach((cb) => {
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
        selectedRoles.forEach((role) => {
            aclRules.push({ rule_type: 'role', rule_value: role, access: 'allow', default_included: false });
        });
        aclRules = aclRules.concat(deptRules);

        const githubUrlInput = form.querySelector('input[name="github_repo_url"]');
        const githubUrl = githubUrlInput ? githubUrlInput.value.trim() : '';

        try {
            if (isEdit) {
                const body = {
                    name: form.querySelector('input[name="name"]').value,
                    description: form.querySelector('textarea[name="description"]').value,
                    github_repo_url: githubUrl || null,
                    plugin_ids: pluginIds
                };
                const resp = await mktFetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(marketplaceId), {
                    method: 'PUT',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(body)
                });
                if (resp.ok) {
                    await mktFetch(app.API_BASE + '/access-control/entity/marketplace/' + encodeURIComponent(marketplaceId), {
                        method: 'PUT',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ rules: aclRules, sync_yaml: false })
                    });
                    app.Toast.show('Marketplace updated', 'success');
                    panelApi.close();
                    setTimeout(() => { window.location.reload(); }, 500);
                } else {
                    const data = await (resp.headers.get('content-type')?.includes('json') ? resp.json() : Promise.resolve({}));
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
                const resp = await mktFetch(app.API_BASE + '/org/marketplaces', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(body)
                });
                if (resp.ok || resp.status === 201) {
                    const created = await (resp.headers.get('content-type')?.includes('json') ? resp.json() : Promise.resolve({}));
                    const createdId = created.id || body.id;
                    if (aclRules.length > 0 && createdId) {
                        await mktFetch(app.API_BASE + '/access-control/entity/marketplace/' + encodeURIComponent(createdId), {
                            method: 'PUT',
                            headers: { 'Content-Type': 'application/json' },
                            body: JSON.stringify({ rules: aclRules, sync_yaml: false })
                        });
                    }
                    app.Toast.show('Marketplace created', 'success');
                    panelApi.close();
                    setTimeout(() => { window.location.reload(); }, 500);
                } else {
                    const data = await (resp.headers.get('content-type')?.includes('json') ? resp.json() : Promise.resolve({}));
                    app.Toast.show(data.error || 'Failed to create', 'error');
                }
            }
        } catch (err) {
            app.Toast.show('Network error', 'error');
        }

        saveBtn.disabled = false;
        saveBtn.textContent = isEdit ? 'Save Changes' : 'Create Marketplace';
    }

    app.initMarketplaceEditForm = () => {
        const form = document.getElementById('marketplace-edit-form');
        if (!form) return;

        const isEdit = !!form.querySelector('input[name="marketplace_id"][readonly]');

        form.addEventListener('submit', async (e) => {
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
            pluginCheckboxes.forEach((cb) => { pluginIds.push(cb.value); });

            const roleCheckboxes = form.querySelectorAll('input[name="roles"]:checked');
            const selectedRoles = [];
            roleCheckboxes.forEach((cb) => { selectedRoles.push(cb.value); });

            const deptCheckboxes = form.querySelectorAll('input[name="departments"]');
            const deptRules = [];
            deptCheckboxes.forEach((cb) => {
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
            selectedRoles.forEach((role) => {
                aclRules.push({ rule_type: 'role', rule_value: role, access: 'allow', default_included: false });
            });
            aclRules = aclRules.concat(deptRules);

            const formGithubInput = form.querySelector('input[name="github_repo_url"]');
            const formGithubUrl = formGithubInput ? formGithubInput.value.trim() : '';

            if (isEdit) {
                const id = form.querySelector('input[name="marketplace_id"]').value;
                const body = {
                    name: form.querySelector('input[name="name"]').value,
                    description: form.querySelector('textarea[name="description"]').value,
                    github_repo_url: formGithubUrl || null,
                    plugin_ids: pluginIds
                };

                try {
                    const resp = await mktFetch(app.API_BASE + '/org/marketplaces/' + encodeURIComponent(id), {
                        method: 'PUT',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(body)
                    });
                    if (resp.ok) {
                        await mktFetch(app.API_BASE + '/access-control/entity/marketplace/' + encodeURIComponent(id), {
                            method: 'PUT',
                            headers: { 'Content-Type': 'application/json' },
                            body: JSON.stringify({ rules: aclRules, sync_yaml: false })
                        });
                        app.Toast.show('Marketplace updated', 'success');
                        setTimeout(() => { window.location.href = '/admin/org/marketplaces/'; }, 500);
                    } else {
                        const data = await (resp.headers.get('content-type')?.includes('json') ? resp.json() : Promise.resolve({}));
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
                    const resp = await mktFetch(app.API_BASE + '/org/marketplaces', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(body)
                    });
                    if (resp.ok || resp.status === 201) {
                        const created = await (resp.headers.get('content-type')?.includes('json') ? resp.json() : Promise.resolve({}));
                        const createdId = created.id || body.id;
                        if (aclRules.length > 0 && createdId) {
                            await mktFetch(app.API_BASE + '/access-control/entity/marketplace/' + encodeURIComponent(createdId), {
                                method: 'PUT',
                                headers: { 'Content-Type': 'application/json' },
                                body: JSON.stringify({ rules: aclRules, sync_yaml: false })
                            });
                        }
                        app.Toast.show('Marketplace created', 'success');
                        setTimeout(() => { window.location.href = '/admin/org/marketplaces/'; }, 500);
                    } else {
                        const data = await (resp.headers.get('content-type')?.includes('json') ? resp.json() : Promise.resolve({}));
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
            deleteBtn.addEventListener('click', () => {
                const idInput = form.querySelector('input[name="marketplace_id"]');
                if (idInput) showDeleteConfirm(idInput.value);
            });
        }

        const checkAllDept = form.querySelector('#dept-check-all');
        if (checkAllDept) {
            checkAllDept.addEventListener('change', () => {
                const boxes = form.querySelectorAll('input[name="departments"]');
                boxes.forEach((cb) => { cb.checked = checkAllDept.checked; });
            });
            form.addEventListener('change', (e) => {
                if (e.target.name === 'departments') {
                    const boxes = form.querySelectorAll('input[name="departments"]');
                    let allChecked = boxes.length > 0;
                    boxes.forEach((cb) => { if (!cb.checked) allChecked = false; });
                    checkAllDept.checked = allChecked;
                }
            });
            const boxes = form.querySelectorAll('input[name="departments"]');
            let allChecked = boxes.length > 0;
            boxes.forEach((cb) => { if (!cb.checked) allChecked = false; });
            checkAllDept.checked = allChecked;
        }
    };

})(window.AdminApp = window.AdminApp || {});
