(function(app) {
    'use strict';

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

    function buildVarList(vars) {
        const frag = document.createDocumentFragment();
        if (!vars.length) {
            const empty = document.createElement('div');
            empty.className = 'empty-state';
            empty.style.cssText = 'padding:var(--sp-space-6)';
            const p = document.createElement('p');
            p.textContent = 'No environment variables defined for this plugin.';
            empty.append(p);
            frag.append(empty);
            return frag;
        }
        vars.forEach((v, i) => {
            const group = document.createElement('div');
            group.className = 'form-group';

            const label = document.createElement('label');
            label.textContent = v.name;
            if (v.required) {
                const reqBadge = document.createElement('span');
                reqBadge.className = 'badge badge-red';
                reqBadge.textContent = 'required';
                label.append(document.createTextNode(' '), reqBadge);
            }
            if (v.secret) {
                const secBadge = document.createElement('span');
                secBadge.className = 'badge badge-gray';
                secBadge.textContent = 'secret';
                label.append(document.createTextNode(' '), secBadge);
            }
            group.append(label);

            if (v.description) {
                const desc = document.createElement('p');
                desc.style.cssText = 'margin:0 0 var(--sp-space-1);font-size:var(--sp-text-xs);color:var(--sp-text-tertiary)';
                desc.textContent = v.description;
                group.append(desc);
            }

            const input = document.createElement('input');
            input.type = v.secret ? 'password' : 'text';
            input.className = 'plugin-env-input';
            input.setAttribute('data-var-index', i);
            input.setAttribute('data-var-name', v.name);
            input.setAttribute('data-is-secret', v.secret ? '1' : '0');
            input.value = v.value;
            if (v.example) input.placeholder = v.example;
            group.append(input);

            frag.append(group);
        });
        return frag;
    }

    function buildModal(vars) {
        const frag = document.createDocumentFragment();

        const heading = document.createElement('h3');
        heading.style.cssText = 'margin:0 0 var(--sp-space-4)';
        heading.textContent = currentPluginName + ' \u2014 Environment Variables';

        const scrollArea = document.createElement('div');
        scrollArea.style.cssText = 'max-height:60vh;overflow-y:auto';
        scrollArea.append(buildVarList(vars));

        const actions = document.createElement('div');
        actions.className = 'form-actions';
        actions.style.cssText = 'display:flex;gap:var(--sp-space-3);justify-content:flex-end;margin-top:var(--sp-space-4)';

        const closeBtn = document.createElement('button');
        closeBtn.className = 'btn btn-secondary';
        closeBtn.id = 'plugin-env-close';
        closeBtn.textContent = 'Close';

        const saveBtn = document.createElement('button');
        saveBtn.className = 'btn btn-primary';
        saveBtn.id = 'plugin-env-save';
        saveBtn.textContent = 'Save';

        actions.append(closeBtn, saveBtn);
        frag.append(heading, scrollArea, actions);
        return frag;
    }

    function updatePanel(vars) {
        const panel = overlay && overlay.querySelector('.confirm-dialog');
        if (panel) {
            panel.replaceChildren(buildModal(vars));
        }
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
                saveBtn.style.background = 'var(--sp-success)';
                saveBtn.style.borderColor = 'var(--sp-success)';
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

        const dialog = document.createElement('div');
        dialog.className = 'confirm-dialog';
        dialog.style.cssText = 'width:560px;max-width:90vw';

        const loadingDiv = document.createElement('div');
        loadingDiv.style.cssText = 'display:flex;align-items:center;justify-content:center;padding:var(--sp-space-6);color:var(--sp-text-tertiary)';
        loadingDiv.textContent = 'Loading...';

        dialog.append(loadingDiv);
        overlay.append(dialog);
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
