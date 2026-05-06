(function(app) {
    'use strict';

    const openCreatePanel = () => {
        const overlay = document.getElementById('create-user-overlay');
        const panel = document.getElementById('create-user-panel');
        if (!overlay || !panel) return;
        overlay.classList.add('open');
        panel.classList.add('open');
        const first = panel.querySelector('input');
        if (first) setTimeout(() => { first.focus(); }, 350);
    };
    const closeCreatePanel = () => {
        const overlay = document.getElementById('create-user-overlay');
        const panel = document.getElementById('create-user-panel');
        if (panel) panel.classList.remove('open');
        if (overlay) overlay.classList.remove('open');
    };
    const resetForm = () => {
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
    };
    const bindCreatePanelEvents = (refreshFn) => {
        app.events.on('click', '#create-user-overlay', () => {
            closeCreatePanel();
        });

        app.events.on('click', '#create-user-panel .panel-close', () => {
            closeCreatePanel();
        });

        app.events.on('click', '#create-user-panel [data-action="cancel"]', () => {
            closeCreatePanel();
        });

        app.events.on('click', '#create-user-panel [data-action="save"]', async () => {
            const userId = document.getElementById('new-user-id').value.trim();
            const displayName = document.getElementById('new-user-name').value.trim();
            const email = document.getElementById('new-user-email').value.trim();
            const deptVal = document.getElementById('new-user-dept').value;
            const roleBoxes = document.querySelectorAll('#create-user-panel input[name="roles"]:checked');
            const roles = Array.from(roleBoxes).map((cb) => cb.value);
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
        });
    };
    app.usersPanel = {
        open: openCreatePanel,
        close: closeCreatePanel,
        bindEvents: bindCreatePanelEvents
    };
})(window.AdminApp);

(function(app) {
    'use strict';

    const showConfirmDialog = app.shared.showConfirmDialog;
    let activePopupId = null;

    const closeAllPopups = () => {
        const portal = document.getElementById('user-actions-popup');
        if (portal) {
            portal.classList.remove('open');
            activePopupId = null;
        }
        const triggers = document.querySelectorAll('.btn-actions-trigger.active');
        for (let i = 0; i < triggers.length; i++) {
            triggers[i].classList.remove('active');
            triggers[i].setAttribute('aria-expanded', 'false');
        }
    };

    const getOrCreatePortal = () => {
        let portal = document.getElementById('user-actions-popup');
        if (!portal) {
            portal = document.createElement('div');
            portal.id = 'user-actions-popup';
            portal.className = 'actions-popup';
            portal.setAttribute('role', 'menu');
            document.body.append(portal);
        }
        return portal;
    };

    const positionPopup = (portal, trigger) => {
        const rect = trigger.getBoundingClientRect();
        const popupH = portal.offsetHeight || 120;
        const spaceBelow = window.innerHeight - rect.bottom;
        portal.style.top = (spaceBelow < popupH)
            ? (rect.top - popupH) + 'px'
            : (rect.bottom + 4) + 'px';
        const popupW = portal.offsetWidth || 180;
        if (window.innerWidth < popupW + 16) {
            portal.style.left = '8px';
            portal.style.right = '8px';
        } else {
            portal.style.right = (window.innerWidth - rect.right) + 'px';
            portal.style.left = '';
        }
    };

    app.usersInteractions = () => {
        app.events.on('click', '.btn-actions-trigger', (e, trigger) => {
            const userId = trigger.dataset.userId;
            const portal = getOrCreatePortal();
            const isOpen = portal.classList.contains('open') && activePopupId === userId;
            closeAllPopups();
            if (isOpen) return;

            const row = trigger.closest('tr');
            const isActive = row && row.querySelector('.badge-green') !== null;
            const toggleLabel = isActive ? 'Deactivate' : 'Activate';
            const toggleClass = isActive ? ' actions-popup-item--danger' : '';

            portal.replaceChildren();

            const editBtn = document.createElement('button');
            editBtn.className = 'actions-popup-item';
            editBtn.setAttribute('data-action', 'edit');
            editBtn.setAttribute('data-user-id', userId);
            const editIcon = document.createElement('span');
            editIcon.className = 'popup-icon';
            editIcon.textContent = '\u270E';
            editBtn.append(editIcon, document.createTextNode(' Edit User'));

            const separator = document.createElement('div');
            separator.className = 'actions-popup-separator';

            const toggleBtn = document.createElement('button');
            toggleBtn.className = 'actions-popup-item' + toggleClass;
            toggleBtn.setAttribute('data-action', 'toggle');
            toggleBtn.setAttribute('data-user-id', userId);
            toggleBtn.setAttribute('data-is-active', String(isActive));
            const toggleIcon = document.createElement('span');
            toggleIcon.className = 'popup-icon';
            toggleIcon.textContent = isActive ? '\u2716' : '\u2714';
            toggleBtn.append(toggleIcon, document.createTextNode(' ' + toggleLabel));

            portal.append(editBtn, separator, toggleBtn);

            activePopupId = userId;
            portal.classList.add('open');
            trigger.classList.add('active');
            trigger.setAttribute('aria-expanded', 'true');
            positionPopup(portal, trigger);

            portal.querySelectorAll('.actions-popup-item').forEach((item) => {
                item.addEventListener('click', (ev) => {
                    const action = item.dataset.action;
                    const itemUserId = item.dataset.userId;
                    closeAllPopups();
                    if (action === 'edit') {
                        window.location.href = app.BASE + '/user/?id=' + encodeURIComponent(itemUserId);
                    } else if (action === 'toggle') {
                        const currentlyActive = item.dataset.isActive === 'true';
                        if (currentlyActive) {
                            showConfirmDialog(
                                'Deactivate User?',
                                'This will prevent the user from accessing the system. You can reactivate them later.',
                                'Deactivate',
                                async () => {
                                    try {
                                        await app.api('/users/' + encodeURIComponent(itemUserId), {
                                            method: 'PUT',
                                            body: JSON.stringify({ is_active: false })
                                        });
                                        app.Toast.show('User deactivated', 'success');
                                        window.location.reload();
                                    } catch (err) {
                                        app.Toast.show(err.message || 'Failed to deactivate user', 'error');
                                    }
                                }
                            );
                        } else {
                            app.api('/users/' + encodeURIComponent(itemUserId), {
                                method: 'PUT',
                                body: JSON.stringify({ is_active: true })
                            }).then(() => {
                                app.Toast.show('User activated', 'success');
                                window.location.reload();
                            }).catch((err) => {
                                app.Toast.show(err.message || 'Failed to activate user', 'error');
                            });
                        }
                    }
                });
            });
        });

        app.events.on('click', '*', (e) => {
            if (!e.target.closest('.btn-actions-trigger') && !e.target.closest('#user-actions-popup')) {
                closeAllPopups();
            }
        });

        const tableScroll = document.querySelector('.table-scroll');
        if (tableScroll) {
            tableScroll.addEventListener('scroll', closeAllPopups);
        }
    };
})(window.AdminApp);
