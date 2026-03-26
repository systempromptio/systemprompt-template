(function(app) {
    'use strict';

    function openCreatePanel() {
        const overlay = document.getElementById('create-user-overlay');
        const panel = document.getElementById('create-user-panel');
        if (!overlay || !panel) return;
        overlay.classList.add('open');
        panel.classList.add('open');
        const first = panel.querySelector('input');
        if (first) setTimeout(function() { first.focus(); }, 350);
    }
    function closeCreatePanel() {
        const overlay = document.getElementById('create-user-overlay');
        const panel = document.getElementById('create-user-panel');
        if (panel) panel.classList.remove('open');
        if (overlay) overlay.classList.remove('open');
    }
    function resetForm() {
        const fields = ['new-user-id', 'new-user-name', 'new-user-email'];
        for (let i = 0; i < fields.length; i++) {
            const el = document.getElementById(fields[i]);
            if (el) el.value = '';
        }
        const dept = document.getElementById('new-user-dept');
        if (dept) dept.value = '';
        const boxes = document.querySelectorAll('#create-user-panel input[name="roles"]');
        for (let j = 0; j < boxes.length; j++) {
            boxes[j].checked = false;
        }
    }
    function bindCreatePanelEvents(refreshFn) {
        document.addEventListener('click', async function(e) {
            if (e.target.id === 'create-user-overlay') {
                closeCreatePanel();
                return;
            }
            const closeBtn = e.target.closest('#create-user-panel .panel-close');
            if (closeBtn) {
                closeCreatePanel();
                return;
            }
            const cancelBtn = e.target.closest('#create-user-panel [data-action="cancel"]');
            if (cancelBtn) {
                closeCreatePanel();
                return;
            }
            const saveBtn = e.target.closest('#create-user-panel [data-action="save"]');
            if (saveBtn) {
                const userId = document.getElementById('new-user-id').value.trim();
                const displayName = document.getElementById('new-user-name').value.trim();
                const email = document.getElementById('new-user-email').value.trim();
                const deptVal = document.getElementById('new-user-dept').value;
                const roleBoxes = document.querySelectorAll('#create-user-panel input[name="roles"]:checked');
                const roles = Array.from(roleBoxes).map(function(cb) { return cb.value; });
                if (!userId) {
                    app.Toast.show('User ID is required', 'error');
                    return;
                }
                try {
                    await app.api('/users', {
                        method: 'POST',
                        body: JSON.stringify({
                            user_id: userId,
                            display_name: displayName || userId,
                            email: email,
                            department: deptVal,
                            roles: roles
                        })
                    });
                    app.Toast.show('User created', 'success');
                    closeCreatePanel();
                    resetForm();
                    refreshFn();
                } catch (err) {
                    app.Toast.show(err.message || 'Failed to create user', 'error');
                }
            }
        });
    }
    app.usersPanel = {
        open: openCreatePanel,
        close: closeCreatePanel,
        bindEvents: bindCreatePanelEvents
    };
})(window.AdminApp);
