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
