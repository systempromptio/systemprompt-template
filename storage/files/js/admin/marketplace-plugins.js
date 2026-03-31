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
        var opt1 = document.createElement('option');
        opt1.value = 'department';
        opt1.textContent = 'Department';
        var opt2 = document.createElement('option');
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
        var optAllow = document.createElement('option');
        optAllow.value = 'allow';
        optAllow.textContent = 'Allow';
        var optDeny = document.createElement('option');
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
            var emptyP = document.createElement('p');
            emptyP.style.cssText = 'font-size:var(--sp-text-sm);color:var(--sp-text-tertiary)';
            emptyP.textContent = 'No rules configured';
            container.append(emptyP);
            return;
        }
        rules.forEach(function(rule, idx) {
            var row = document.createElement('div');
            row.style.cssText = 'display:flex;align-items:center;gap:var(--sp-space-2);padding:var(--sp-space-1) 0;font-size:var(--sp-text-sm)';
            var badge = document.createElement('span');
            badge.className = 'badge ' + (rule.access === 'allow' ? 'badge-yellow' : 'badge-red');
            badge.textContent = rule.rule_type + ': ' + rule.rule_value + ' (' + rule.access + ')';
            var removeBtn = document.createElement('button');
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
                            var noUsers = document.createElement('div');
                            noUsers.style.cssText = 'margin-top:var(--sp-space-2);font-size:var(--sp-text-xs);color:var(--sp-text-tertiary)';
                            noUsers.textContent = 'No users found';
                            container.append(noUsers);
                        } else {
                            var userList = document.createElement('div');
                            userList.style.cssText = 'margin-top:var(--sp-space-2);display:flex;flex-direction:column;gap:var(--sp-space-1)';
                            users.forEach(function(u) {
                                var userRow = document.createElement('div');
                                userRow.style.cssText = 'display:flex;align-items:center;gap:var(--sp-space-2);font-size:var(--sp-text-xs);padding:var(--sp-space-1) 0;border-bottom:1px solid var(--sp-border-primary)';
                                var nameSpan = document.createElement('span');
                                nameSpan.style.cssText = 'font-weight:600;color:var(--sp-text-primary)';
                                nameSpan.textContent = u.display_name || 'Unknown';
                                userRow.append(nameSpan);
                                if (u.department) {
                                    var deptBadge = document.createElement('span');
                                    deptBadge.className = 'badge badge-blue';
                                    deptBadge.textContent = u.department;
                                    userRow.append(deptBadge);
                                }
                                var eventBadge = document.createElement('span');
                                eventBadge.className = 'badge badge-gray';
                                eventBadge.textContent = (u.event_count || 0) + ' events';
                                userRow.append(eventBadge);
                                if (u.last_used) {
                                    var dateSpan = document.createElement('span');
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
