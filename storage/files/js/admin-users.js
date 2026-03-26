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

(function(app) {
    'use strict';

    const showConfirmDialog = app.shared.showConfirmDialog;
    let activePopupId = null;

    function closeAllPopups() {
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
    }

    function getOrCreatePortal() {
        let portal = document.getElementById('user-actions-popup');
        if (!portal) {
            portal = document.createElement('div');
            portal.id = 'user-actions-popup';
            portal.className = 'actions-popup';
            portal.setAttribute('role', 'menu');
            document.body.appendChild(portal);
        }
        return portal;
    }

    function positionPopup(portal, trigger) {
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
    }

    app.usersInteractions = function() {
        app.events.on('click', '.btn-actions-trigger', function(e, trigger) {
            e.stopPropagation();
            const userId = trigger.dataset.userId;
            const portal = getOrCreatePortal();
            const isOpen = portal.classList.contains('open') && activePopupId === userId;
            closeAllPopups();
            if (isOpen) return;

            const row = trigger.closest('tr');
            const isActive = row && row.querySelector('.badge-green') !== null;
            const toggleLabel = isActive ? 'Deactivate' : 'Activate';
            const toggleIcon = isActive ? '&#10006;' : '&#10004;';
            const toggleClass = isActive ? ' actions-popup-item--danger' : '';

            portal.innerHTML =
                '<button class="actions-popup-item" data-action="edit" data-user-id="' + userId + '"><span class="popup-icon">&#9998;</span> Edit User</button>' +
                '<div class="actions-popup-separator"></div>' +
                '<button class="actions-popup-item' + toggleClass + '" data-action="toggle" data-user-id="' + userId + '" data-is-active="' + isActive + '"><span class="popup-icon">' + toggleIcon + '</span> ' + toggleLabel + '</button>';

            activePopupId = userId;
            portal.classList.add('open');
            trigger.classList.add('active');
            trigger.setAttribute('aria-expanded', 'true');
            positionPopup(portal, trigger);

            portal.querySelectorAll('.actions-popup-item').forEach(function(item) {
                item.addEventListener('click', function(ev) {
                    ev.stopPropagation();
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
                                async function() {
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
                            }).then(function() {
                                app.Toast.show('User activated', 'success');
                                window.location.reload();
                            }).catch(function(err) {
                                app.Toast.show(err.message || 'Failed to activate user', 'error');
                            });
                        }
                    }
                });
            });
        });

        app.events.on('click', '*', function(e) {
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
