(function(app) {
    'use strict';

    const escapeHtml = app.escapeHtml;
    let overlay = null;
    let currentPluginId = null;
    let currentPluginName = '';
    let envVars = [];
    let varDefs = [];

    function mergeDefsWithValues(defs, stored) {
        const merged = [];
        const storedMap = {};
        stored.forEach((v) => { storedMap[v.var_name] = v; });

        defs.forEach((def) => {
            const existing = storedMap[def.name];
            merged.push({
                name: def.name,
                description: def.description || '',
                required: def.required !== false,
                secret: def.secret || false,
                example: def.example || '',
                value: existing ? existing.var_value : '',
                fromDef: true
            });
            delete storedMap[def.name];
        });

        Object.keys(storedMap).forEach((key) => {
            const s = storedMap[key];
            merged.push({
                name: s.var_name,
                description: '',
                required: false,
                secret: s.is_secret,
                example: '',
                value: s.var_value,
                fromDef: false
            });
        });

        return merged;
    }

    function renderVarList(vars) {
        if (!vars.length) {
            return '<div class="empty-state" style="padding:var(--space-6)"><p>No environment variables defined for this plugin.</p></div>';
        }
        let html = '';
        vars.forEach((v, i) => {
            const inputType = v.secret ? 'password' : 'text';
            const placeholder = v.example ? v.example : '';
            const requiredBadge = v.required ? ' <span class="badge badge-red">required</span>' : '';
            const secretBadge = v.secret ? ' <span class="badge badge-gray">secret</span>' : '';
            html += '<div class="form-group">' +
                '<label>' + escapeHtml(v.name) + requiredBadge + secretBadge + '</label>' +
                (v.description ? '<p style="margin:0 0 var(--space-1);font-size:var(--text-xs);color:var(--text-tertiary)">' + escapeHtml(v.description) + '</p>' : '') +
                '<input type="' + inputType + '" class="plugin-env-input" data-var-index="' + i + '" data-var-name="' + escapeHtml(v.name) + '" data-is-secret="' + (v.secret ? '1' : '0') + '" ' +
                    'value="' + escapeHtml(v.value) + '" placeholder="' + escapeHtml(placeholder) + '">' +
            '</div>';
        });
        return html;
    }

    function renderModal(vars) {
        return '<h3 style="margin:0 0 var(--space-4)">' + escapeHtml(currentPluginName) + ' — Environment Variables</h3>' +
            '<div style="max-height:60vh;overflow-y:auto">' +
                renderVarList(vars) +
            '</div>' +
            '<div class="form-actions" style="display:flex;gap:var(--space-3);justify-content:flex-end;margin-top:var(--space-4)">' +
                '<button class="btn btn-secondary" id="plugin-env-close">Close</button>' +
                '<button class="btn btn-primary" id="plugin-env-save">Save</button>' +
            '</div>';
    }

    function updatePanel(vars) {
        const panel = overlay && overlay.querySelector('.confirm-dialog');
        if (panel) panel.innerHTML = renderModal(vars);
        bindEvents(vars);
    }

    function bindEvents(vars) {
        if (!overlay) return;

        const closeBtn = overlay.querySelector('#plugin-env-close');
        if (closeBtn) closeBtn.addEventListener('click', close);

        const saveBtn = overlay.querySelector('#plugin-env-save');
        if (saveBtn) saveBtn.addEventListener('click', () => { handleSave(vars); });
    }

    async function handleSave(vars) {
        const saveBtn = overlay && overlay.querySelector('#plugin-env-save');
        if (saveBtn) {
            saveBtn.disabled = true;
            saveBtn.textContent = 'Saving...';
        }
        try {
            const inputs = overlay.querySelectorAll('.plugin-env-input');
            const payload = [];
            inputs.forEach((input) => {
                const name = input.getAttribute('data-var-name');
                const isSecret = input.getAttribute('data-is-secret') === '1';
                const value = input.value;
                if (isSecret && value === '••••••••') return;
                payload.push({ var_name: name, var_value: value, is_secret: isSecret });
            });
            await app.api('/plugins/' + encodeURIComponent(currentPluginId) + '/env', {
                method: 'PUT',
                body: JSON.stringify({ variables: payload }),
                headers: { 'Content-Type': 'application/json' }
            });
            window.dispatchEvent(new CustomEvent('env-saved', { detail: { pluginId: currentPluginId } }));
            if (saveBtn) {
                saveBtn.textContent = 'Saved';
                saveBtn.style.background = 'var(--success)';
                saveBtn.style.borderColor = 'var(--success)';
            }
            app.Toast.show('Environment variables saved', 'success');
            setTimeout(() => { close(); }, 600);
        } catch (err) {
            app.Toast.show(err.message || 'Save failed', 'error');
            if (saveBtn) {
                saveBtn.disabled = false;
                saveBtn.textContent = 'Save';
            }
        }
    }

    async function loadAndRender() {
        try {
            const data = await app.api('/plugins/' + encodeURIComponent(currentPluginId) + '/env');
            envVars = data.stored || [];
            varDefs = data.definitions || [];
            const merged = mergeDefsWithValues(varDefs, envVars);
            updatePanel(merged);
        } catch (err) {
            envVars = [];
            varDefs = [];
            app.Toast.show(err.message || 'Failed to load env vars', 'error');
            updatePanel([]);
        }
    }

    function close() {
        if (overlay) {
            overlay.remove();
            overlay = null;
        }
        currentPluginId = null;
        currentPluginName = '';
        envVars = [];
        varDefs = [];
    }

    async function open(pluginId, pluginName) {
        close();
        currentPluginId = pluginId;
        currentPluginName = pluginName || pluginId;

        overlay = document.createElement('div');
        overlay.className = 'confirm-overlay';
        overlay.innerHTML = '<div class="confirm-dialog" style="width:560px;max-width:90vw">' +
            '<div style="display:flex;align-items:center;justify-content:center;padding:var(--space-6);color:var(--text-tertiary)">Loading...</div>' +
        '</div>';
        document.body.append(overlay);

        overlay.addEventListener('click', (e) => {
            if (e.target === overlay) close();
        });

        await loadAndRender();
    }

    app.pluginEnv = {
        open: open,
        close: close
    };
})(window.AdminApp);
