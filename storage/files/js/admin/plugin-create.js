(function(app) {
    'use strict';

    const TOTAL_STEPS = 7;
    const state = {
        step: 1,
        form: { plugin_id: '', name: '', description: '', version: '0.1.0', category: '', author_name: '', keywords: '', roles: {} },
        selectedSkills: {},
        selectedAgents: {},
        selectedMcpServers: {},
        hooks: []
    };
    let root = null;

    function getTemplate(id) {
        const tpl = document.getElementById(id);
        return tpl ? tpl.content.cloneNode(true) : document.createDocumentFragment();
    }

    function renderStepIndicator() {
        const labels = ['Basic Info', 'Skills', 'Agents', 'MCP Servers', 'Hooks', 'Roles & Access', 'Review'];
        const container = document.getElementById('wizard-step-indicator');
        if (!container) return;
        const steps = document.createElement('div');
        steps.className = 'wizard-steps';
        steps.style.cssText = 'display:flex;gap:var(--sp-space-1);margin-bottom:var(--sp-space-6);flex-wrap:wrap';
        for (let i = 1; i <= TOTAL_STEPS; i++) {
            const isActive = i === state.step;
            const isDone = i < state.step;
            const bgColor = isActive ? 'var(--sp-accent)' : (isDone ? 'var(--sp-success)' : 'var(--sp-bg-tertiary)');
            const textColor = (isActive || isDone) ? '#fff' : 'var(--sp-text-tertiary)';
            const stepDiv = document.createElement('div');
            stepDiv.style.cssText = 'display:flex;align-items:center;gap:var(--sp-space-2);padding:var(--sp-space-2) var(--sp-space-3);border-radius:var(--sp-radius-md);font-size:var(--sp-text-sm);background:' + bgColor + ';color:' + textColor + ';font-weight:' + (isActive ? '600' : '400');
            const numSpan = document.createElement('span');
            numSpan.style.cssText = 'width:20px;height:20px;border-radius:50%;background:rgba(255,255,255,0.2);display:inline-flex;align-items:center;justify-content:center;font-size:var(--sp-text-xs)';
            numSpan.textContent = i;
            const labelSpan = document.createElement('span');
            labelSpan.textContent = labels[i - 1];
            stepDiv.append(numSpan, labelSpan);
            steps.append(stepDiv);
        }
        container.replaceChildren(steps);
    }

    function renderNav() {
        const nav = document.getElementById('wizard-nav');
        if (!nav) return;
        const wrapper = document.createElement('div');
        wrapper.style.cssText = 'display:flex;gap:var(--sp-space-3);margin-top:var(--sp-space-6)';
        if (state.step > 1) {
            const prevBtn = document.createElement('button');
            prevBtn.type = 'button';
            prevBtn.className = 'btn btn-secondary';
            prevBtn.id = 'wizard-prev';
            prevBtn.textContent = 'Previous';
            wrapper.append(prevBtn);
        }
        if (state.step < TOTAL_STEPS) {
            const nextBtn = document.createElement('button');
            nextBtn.type = 'button';
            nextBtn.className = 'btn btn-primary';
            nextBtn.id = 'wizard-next';
            nextBtn.textContent = 'Next';
            wrapper.append(nextBtn);
        }
        if (state.step === TOTAL_STEPS) {
            const createBtn = document.createElement('button');
            createBtn.type = 'button';
            createBtn.className = 'btn btn-primary';
            createBtn.id = 'wizard-create';
            createBtn.textContent = 'Create Plugin';
            wrapper.append(createBtn);
        }
        nav.replaceChildren(wrapper);
    }

    function saveCurrentStepState() {
        if (!root) return;
        if (state.step === 1) {
            ['plugin_id', 'name', 'description', 'version', 'category'].forEach((name) => {
                const input = root.querySelector('[name="' + name + '"]');
                if (input) state.form[name] = input.tagName === 'TEXTAREA' ? input.value : input.value;
            });
        }
        if (state.step === 2) {
            state.selectedSkills = {};
            root.querySelectorAll('input[name="wizard-skills"]:checked').forEach((cb) => { state.selectedSkills[cb.value] = true; });
        }
        if (state.step === 3) {
            state.selectedAgents = {};
            root.querySelectorAll('input[name="wizard-agents"]:checked').forEach((cb) => { state.selectedAgents[cb.value] = true; });
        }
        if (state.step === 4) {
            state.selectedMcpServers = {};
            root.querySelectorAll('input[name="wizard-mcp"]:checked').forEach((cb) => { state.selectedMcpServers[cb.value] = true; });
        }
        if (state.step === 5) {
            const entries = root.querySelectorAll('.hook-entry');
            state.hooks = [];
            entries.forEach((entry) => {
                state.hooks.push({
                    event: (entry.querySelector('[name="hook_event"]') || {}).value || 'PostToolUse',
                    matcher: (entry.querySelector('[name="hook_matcher"]') || {}).value || '*',
                    command: (entry.querySelector('[name="hook_command"]') || {}).value || '',
                    async: !!(entry.querySelector('[name="hook_async"]') || {}).checked
                });
            });
        }
        if (state.step === 6) {
            state.form.roles = {};
            root.querySelectorAll('input[name="wizard-roles"]:checked').forEach((cb) => { state.form.roles[cb.value] = true; });
            const authorInput = root.querySelector('[name="author_name"]');
            if (authorInput) state.form.author_name = authorInput.value;
            const keywordsInput = root.querySelector('[name="keywords"]');
            if (keywordsInput) state.form.keywords = keywordsInput.value;
        }
    }

    function renderStep() {
        const contentEl = document.getElementById('wizard-step-content');
        if (!contentEl) return;
        contentEl.replaceChildren();

        if (state.step === 7) {
            const frag = getTemplate('tpl-step-7');
            contentEl.append(frag);
            renderReview();
        } else if (state.step === 5) {
            const frag5 = getTemplate('tpl-step-5');
            contentEl.append(frag5);
            renderHooks();
        } else {
            const frag2 = getTemplate('tpl-step-' + state.step);
            contentEl.append(frag2);
            restoreStepState();
        }

        renderStepIndicator();
        renderNav();
        app.formUtils.attachFilterHandlers(contentEl);
    }

    function restoreStepState() {
        if (state.step === 1) {
            ['plugin_id', 'name', 'description', 'version', 'category'].forEach((name) => {
                const input = root.querySelector('[name="' + name + '"]');
                if (input && state.form[name]) {
                    if (input.tagName === 'TEXTAREA') input.value = state.form[name];
                    else input.value = state.form[name];
                }
            });
        }
        if (state.step === 2) {
            Object.keys(state.selectedSkills).forEach((val) => {
                if (!state.selectedSkills[val]) return;
                const cb = root.querySelector('input[name="wizard-skills"][value="' + val + '"]');
                if (cb) cb.checked = true;
            });
        }
        if (state.step === 3) {
            Object.keys(state.selectedAgents).forEach((val) => {
                if (!state.selectedAgents[val]) return;
                const cb = root.querySelector('input[name="wizard-agents"][value="' + val + '"]');
                if (cb) cb.checked = true;
            });
        }
        if (state.step === 4) {
            Object.keys(state.selectedMcpServers).forEach((val) => {
                if (!state.selectedMcpServers[val]) return;
                const cb = root.querySelector('input[name="wizard-mcp"][value="' + val + '"]');
                if (cb) cb.checked = true;
            });
        }
        if (state.step === 6) {
            Object.keys(state.form.roles).forEach((val) => {
                if (!state.form.roles[val]) return;
                const cb = root.querySelector('input[name="wizard-roles"][value="' + val + '"]');
                if (cb) cb.checked = true;
            });
            const authorInput = root.querySelector('[name="author_name"]');
            if (authorInput && state.form.author_name) authorInput.value = state.form.author_name;
            const keywordsInput = root.querySelector('[name="keywords"]');
            if (keywordsInput && state.form.keywords) keywordsInput.value = state.form.keywords;
        }
    }

    function renderHooks() {
        const list = document.getElementById('hooks-list');
        if (!list) return;
        list.replaceChildren();
        state.hooks.forEach((hook) => {
            const frag = getTemplate('tpl-hook-entry');
            const entry = frag.querySelector('.hook-entry');
            if (entry) {
                const eventSel = entry.querySelector('[name="hook_event"]');
                if (eventSel) eventSel.value = hook.event || 'PostToolUse';
                const matcherIn = entry.querySelector('[name="hook_matcher"]');
                if (matcherIn) matcherIn.value = hook.matcher || '*';
                const commandIn = entry.querySelector('[name="hook_command"]');
                if (commandIn) commandIn.value = hook.command || '';
                const asyncCb = entry.querySelector('[name="hook_async"]');
                if (asyncCb) asyncCb.checked = !!hook.async;
            }
            list.append(frag);
        });
    }

    function renderReview() {
        const el = document.getElementById('wizard-review');
        if (!el) return;
        const f = state.form;
        const selectedSkills = Object.keys(state.selectedSkills).filter((k) => state.selectedSkills[k]);
        const selectedAgents = Object.keys(state.selectedAgents).filter((k) => state.selectedAgents[k]);
        const selectedMcp = Object.keys(state.selectedMcpServers).filter((k) => state.selectedMcpServers[k]);
        const selectedRoles = Object.keys(f.roles).filter((k) => f.roles[k]);

        function buildBadgeList(items, emptyMsg) {
            const container = document.createDocumentFragment();
            if (!items.length) {
                const empty = document.createElement('span');
                empty.style.cssText = 'color:var(--sp-text-tertiary)';
                empty.textContent = emptyMsg;
                container.append(empty);
                return container;
            }
            items.forEach(function(item) {
                const badge = document.createElement('span');
                badge.className = 'badge badge-blue';
                badge.style.cssText = 'margin:var(--sp-space-1)';
                badge.textContent = item;
                container.append(badge);
            });
            return container;
        }

        function addRow(frag, labelText, valueText) {
            const strong = document.createElement('strong');
            strong.textContent = labelText;
            const span = document.createElement('span');
            span.textContent = valueText;
            frag.append(strong, span);
        }

        function addBadgeRow(frag, labelText, items, emptyMsg, wrap) {
            const strong = document.createElement('strong');
            strong.textContent = labelText;
            const div = document.createElement('div');
            if (wrap) div.style.cssText = 'display:flex;flex-wrap:wrap';
            div.append(buildBadgeList(items, emptyMsg));
            frag.append(strong, div);
        }

        const frag = document.createDocumentFragment();
        addRow(frag, 'Plugin ID:', f.plugin_id || '-');
        addRow(frag, 'Name:', f.name || '-');
        addRow(frag, 'Description:', f.description || '-');
        addRow(frag, 'Version:', f.version || '0.1.0');
        addRow(frag, 'Category:', f.category || '-');
        addRow(frag, 'Author:', f.author_name || '-');
        addRow(frag, 'Keywords:', f.keywords || '-');
        addBadgeRow(frag, 'Roles:', selectedRoles, 'None selected', false);
        addBadgeRow(frag, 'Skills (' + selectedSkills.length + '):', selectedSkills, 'None selected', true);
        addBadgeRow(frag, 'Agents (' + selectedAgents.length + '):', selectedAgents, 'None selected', true);
        addBadgeRow(frag, 'MCP (' + selectedMcp.length + '):', selectedMcp, 'None selected', true);

        const hooksStrong = document.createElement('strong');
        hooksStrong.textContent = 'Hooks (' + state.hooks.length + '):';
        const hooksSpan = document.createElement('span');
        hooksSpan.textContent = state.hooks.length > 0
            ? state.hooks.map(function(h) { return h.event + ': ' + (h.command || '?'); }).join(', ')
            : 'None';
        frag.append(hooksStrong, hooksSpan);

        el.replaceChildren(frag);
    }

    function validateStep1() {
        const pid = state.form.plugin_id;
        const name = state.form.name;
        if (!pid || !pid.trim()) { app.Toast.show('Plugin ID is required', 'error'); return false; }
        if (!/^[a-z0-9]+(-[a-z0-9]+)*$/.test(pid.trim())) { app.Toast.show('Plugin ID must be kebab-case (e.g. my-plugin)', 'error'); return false; }
        if (!name || !name.trim()) { app.Toast.show('Name is required', 'error'); return false; }
        return true;
    }

    function buildPluginBody() {
        const f = state.form;
        return {
            id: f.plugin_id.trim(),
            name: f.name.trim(),
            description: f.description || '',
            version: f.version || '0.1.0',
            category: f.category || '',
            enabled: true,
            keywords: (f.keywords || '').split(',').map((t) => t.trim()).filter(Boolean),
            author: { name: f.author_name || '' },
            roles: Object.keys(f.roles).filter((k) => f.roles[k]),
            skills: Object.keys(state.selectedSkills).filter((k) => state.selectedSkills[k]),
            agents: Object.keys(state.selectedAgents).filter((k) => state.selectedAgents[k]),
            mcp_servers: Object.keys(state.selectedMcpServers).filter((k) => state.selectedMcpServers[k]),
            hooks: state.hooks.filter((h) => h.command).map((h) => {
                return { event: h.event || 'PostToolUse', matcher: h.matcher || '*', command: h.command, async: !!h.async };
            })
        };
    }

    async function createPlugin() {
        const body = buildPluginBody();
        const btn = root.querySelector('#wizard-create');
        if (btn) { btn.disabled = true; btn.textContent = 'Creating...'; }
        try {
            await app.api('/plugins', { method: 'POST', body: JSON.stringify(body) });
            app.Toast.show('Plugin created!', 'success');
            window.location.href = app.BASE + '/plugins/';
        } catch (err) {
            app.Toast.show(err.message || 'Failed to create plugin', 'error');
            if (btn) { btn.disabled = false; btn.textContent = 'Create Plugin'; }
        }
    }

    function showImportModal() {
        const modal = document.getElementById('import-modal');
        if (modal) { modal.style.display = 'flex'; const urlInput = modal.querySelector('#import-url'); if (urlInput) urlInput.focus(); }
    }
    function hideImportModal() {
        const modal = document.getElementById('import-modal');
        if (modal) { modal.style.display = 'none'; const err = modal.querySelector('#import-error'); if (err) err.style.display = 'none'; }
    }
    async function submitImport() {
        const urlInput = document.getElementById('import-url');
        const errorEl = document.getElementById('import-error');
        const submitBtn = document.getElementById('import-submit');
        const targetSelect = document.getElementById('import-target');
        if (!urlInput || !submitBtn) return;
        const url = urlInput.value.trim();
        if (!url) { if (errorEl) { errorEl.textContent = 'URL is required'; errorEl.style.display = 'block'; } return; }
        const importTarget = targetSelect ? targetSelect.value : 'site';
        submitBtn.disabled = true; submitBtn.textContent = 'Importing...';
        if (errorEl) errorEl.style.display = 'none';
        try {
            await app.api('/plugins/import', { method: 'POST', body: JSON.stringify({ url: url, import_target: importTarget }) });
            app.Toast.show('Plugin imported successfully!', 'success');
            window.location.href = app.BASE + '/plugins/';
        } catch (err) {
            if (errorEl) { errorEl.textContent = err.message || 'Failed to import plugin'; errorEl.style.display = 'block'; }
            submitBtn.disabled = false; submitBtn.textContent = 'Import';
        }
    }

    app.initPluginWizard = function() {
        root = document.getElementById('plugin-create-content');
        if (!root) return;

        renderStep();

        root.addEventListener('click', (e) => {
            if (e.target.closest('#btn-import-url')) { showImportModal(); return; }
            if (e.target.closest('#import-cancel')) { hideImportModal(); return; }
            if (e.target.closest('#import-submit')) { submitImport(); return; }
            if (e.target.id === 'import-modal') { hideImportModal(); return; }
            if (e.target.closest('#wizard-next')) {
                saveCurrentStepState();
                if (state.step === 1 && !validateStep1()) return;
                if (state.step < TOTAL_STEPS) { state.step++; renderStep(); }
                return;
            }
            if (e.target.closest('#wizard-prev')) {
                saveCurrentStepState();
                if (state.step > 1) { state.step--; renderStep(); }
                return;
            }
            if (e.target.closest('#wizard-create')) { saveCurrentStepState(); createPlugin(); return; }
            if (e.target.closest('#btn-add-hook')) {
                saveCurrentStepState();
                state.hooks.push({ event: 'PostToolUse', matcher: '*', command: '', async: false });
                renderHooks();
                return;
            }
            const removeBtn = e.target.closest('[data-remove-hook]');
            if (removeBtn) {
                saveCurrentStepState();
                const entry = removeBtn.closest('.hook-entry');
                const hookList = document.getElementById('hooks-list');
                if (entry && hookList) {
                    const hookEntries = Array.from(hookList.querySelectorAll('.hook-entry'));
                    const idx = hookEntries.indexOf(entry);
                    if (idx >= 0) state.hooks.splice(idx, 1);
                    renderHooks();
                }
                return;
            }
            const selectAllBtn = e.target.closest('[data-select-all]');
            if (selectAllBtn) {
                const listId = selectAllBtn.getAttribute('data-select-all');
                const container = root.querySelector('[data-checklist="' + listId + '"]');
                if (container) container.querySelectorAll('input[type="checkbox"]').forEach((cb) => { cb.checked = true; });
                return;
            }
            const deselectAllBtn = e.target.closest('[data-deselect-all]');
            if (deselectAllBtn) {
                const listId2 = deselectAllBtn.getAttribute('data-deselect-all');
                const container2 = root.querySelector('[data-checklist="' + listId2 + '"]');
                if (container2) container2.querySelectorAll('input[type="checkbox"]').forEach((cb) => { cb.checked = false; });
                return;
            }
        });

        root.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' && e.target.id === 'import-url') { e.preventDefault(); submitImport(); }
            if (e.key === 'Escape') hideImportModal();
        });
    };
})(window.AdminApp);
