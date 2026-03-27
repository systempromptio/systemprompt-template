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
