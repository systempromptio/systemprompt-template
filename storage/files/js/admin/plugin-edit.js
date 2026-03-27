(function(app) {
    'use strict';

    app.initPluginEditForm = function() {
        const form = document.getElementById('plugin-edit-form');
        if (!form) return;

        const pluginIdInput = form.querySelector('input[name="plugin_id"]');
        const pluginId = pluginIdInput ? pluginIdInput.value : '';

        form.addEventListener('submit', async (e) => {
            e.preventDefault();
            const formData = new FormData(form);
            const keywordsRaw = formData.get('keywords') || '';
            const keywords = keywordsRaw.split(',').map((t) => t.trim()).filter(Boolean);
            const body = {
                name: formData.get('name'),
                description: formData.get('description') || '',
                version: formData.get('version') || '0.1.0',
                category: formData.get('category') || '',
                enabled: !!form.querySelector('input[name="enabled"]').checked,
                keywords: keywords,
                author: { name: formData.get('author_name') || '' },
                roles: app.formUtils.getCheckedValues(form, 'roles'),
                skills: app.formUtils.getCheckedValues(form, 'skills'),
                agents: app.formUtils.getCheckedValues(form, 'agents'),
                mcp_servers: app.formUtils.getCheckedValues(form, 'mcp_servers')
            };
            const submitBtn = form.querySelector('[type="submit"]');
            if (submitBtn) { submitBtn.disabled = true; submitBtn.textContent = 'Saving...'; }
            try {
                await app.api('/plugins/' + encodeURIComponent(pluginId), {
                    method: 'PUT',
                    body: JSON.stringify(body)
                });
                app.Toast.show('Plugin saved!', 'success');
                window.location.href = app.BASE + '/plugins/';
            } catch (err) {
                app.Toast.show(err.message || 'Failed to save plugin', 'error');
                if (submitBtn) { submitBtn.disabled = false; submitBtn.textContent = 'Save Changes'; }
            }
        });

        const deleteBtn = document.getElementById('btn-delete-plugin');
        if (deleteBtn) {
            deleteBtn.addEventListener('click', () => {
                app.shared.showConfirmDialog('Delete Plugin?', 'Are you sure you want to delete this plugin? This cannot be undone.', 'Delete', async () => {
                    deleteBtn.disabled = true;
                    deleteBtn.textContent = 'Deleting...';
                    try {
                        await app.api('/plugins/' + encodeURIComponent(pluginId), { method: 'DELETE' });
                        app.Toast.show('Plugin deleted', 'success');
                        window.location.href = app.BASE + '/plugins/';
                    } catch (err) {
                        app.Toast.show(err.message || 'Failed to delete plugin', 'error');
                        deleteBtn.disabled = false;
                        deleteBtn.textContent = 'Delete';
                    }
                });
            });
        }

        app.formUtils.attachFilterHandlers(form);
    };
})(window.AdminApp);
