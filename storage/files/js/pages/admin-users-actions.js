import { apiFetch, BASE } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';
import { on } from '../services/events.js';

let activePopupId = null;

const closeAllPopups = () => {
  const portal = document.getElementById('user-actions-popup');
  if (portal) {
    portal.classList.remove('open');
    activePopupId = null;
  }
  for (const t of document.querySelectorAll('.btn-actions-trigger.active')) {
    t.classList.remove('active');
    t.setAttribute('aria-expanded', 'false');
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

const buildPopupButton = (className, action, userId, iconChar, label) => {
  const btn = document.createElement('button');
  btn.className = className;
  btn.dataset.action = action;
  btn.dataset.userId = userId;
  const icon = document.createElement('span');
  icon.className = 'popup-icon';
  icon.textContent = iconChar;
  btn.append(icon);
  btn.append(document.createTextNode(' ' + label));
  return btn;
};

const buildPopupContent = (portal, userId, isActive) => {
  const toggleLabel = isActive ? 'Deactivate' : 'Activate';
  const toggleClass = isActive ? ' actions-popup-item--danger' : '';
  portal.textContent = '';
  portal.append(buildPopupButton('actions-popup-item', 'edit', userId, '\u270E', 'Edit User'));
  const sep = document.createElement('div');
  sep.className = 'actions-popup-separator';
  portal.append(sep);
  const toggleBtn = buildPopupButton('actions-popup-item' + toggleClass, 'toggle', userId, isActive ? '\u2716' : '\u2714', toggleLabel);
  toggleBtn.dataset.isActive = isActive;
  portal.append(toggleBtn);
};

const handlePopupItemClick = async (item) => {
  const action = item.dataset.action;
  const itemUserId = item.dataset.userId;
  closeAllPopups();
  if (action === 'edit') {
    window.location.href = BASE + '/user/?id=' + encodeURIComponent(itemUserId);
  } else if (action === 'toggle') {
    const currentlyActive = item.dataset.isActive === 'true';
    if (currentlyActive) {
      showConfirmDialog('Deactivate User?', 'This will prevent the user from accessing the system.', 'Deactivate', async () => {
        try {
          await apiFetch('/users/' + encodeURIComponent(itemUserId), { method: 'PUT', body: JSON.stringify({ is_active: false }) });
          showToast('User deactivated', 'success');
          window.location.reload();
        } catch (err) { showToast(err.message || 'Failed to deactivate user', 'error'); }
      });
    } else {
      try {
        await apiFetch('/users/' + encodeURIComponent(itemUserId), { method: 'PUT', body: JSON.stringify({ is_active: true }) });
        showToast('User activated', 'success');
        window.location.reload();
      } catch (err) { showToast(err.message || 'Failed to activate user', 'error'); }
    }
  }
};

export const bindActionsPopup = () => {
  on('click', '.btn-actions-trigger', (e, trigger) => {
    e.stopPropagation();
    const userId = trigger.dataset.userId;
    const portal = getOrCreatePortal();
    const isOpen = portal.classList.contains('open') && activePopupId === userId;
    closeAllPopups();
    if (!isOpen) {
      const row = trigger.closest('tr');
      const isActive = row && row.querySelector('.badge-success') !== null;
      buildPopupContent(portal, userId, isActive);
      activePopupId = userId;
      portal.classList.add('open');
      trigger.classList.add('active');
      trigger.setAttribute('aria-expanded', 'true');
      positionPopup(portal, trigger);
      for (const item of portal.querySelectorAll('.actions-popup-item')) {
        item.addEventListener('click', (ev) => {
          ev.stopPropagation();
          handlePopupItemClick(item);
        });
      }
    }
  });

  on('click', '*', (e) => {
    if (!e.target.closest('.btn-actions-trigger') && !e.target.closest('#user-actions-popup')) closeAllPopups();
  });

  const tableScroll = document.querySelector('.table-scroll');
  if (tableScroll) tableScroll.addEventListener('scroll', closeAllPopups);
};
