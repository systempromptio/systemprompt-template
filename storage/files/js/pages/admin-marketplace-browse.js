import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';

export const initMarketplace = (selector, pluginsData) => {
  const root = document.querySelector(selector);
  if (root) {
    setMktPlugins(pluginsData || []);

    initMktSearch(root);
    initMktSort();

    root.addEventListener('click', async (e) => {
      const toggleBtn = e.target.closest('[data-toggle-plugin]');
      if (toggleBtn) {
        handlePluginToggle(toggleBtn);
      } else {
        const visBtn = e.target.closest('[data-edit-visibility]');
        if (visBtn) {
          e.stopPropagation();
          showVisibilityModal(visBtn.getAttribute('data-edit-visibility'));
        } else {
          const loadUsersBtn = e.target.closest('[data-load-users]');
          if (loadUsersBtn) {
            e.stopPropagation();
            await handleLoadUsers(root, loadUsersBtn);
          }
        }
      }
    });
  }
};

const handlePluginToggle = (toggleBtn) => {
  const card = toggleBtn.closest('.plugin-card');
  const details = card?.querySelector('.plugin-details');
  if (details) {
    details.hidden = !details.hidden;
    const icon = toggleBtn.querySelector('.expand-icon');
    if (icon) icon.classList.toggle('sp-expand-icon-open', !details.hidden);
  }
};

const initMktSearch = (root) => {
  const searchInput = document.getElementById('mkt-search');
  if (searchInput) {
    let timer;
    searchInput.addEventListener('input', () => {
      clearTimeout(timer);
      timer = setTimeout(() => {
        const q = searchInput.value.toLowerCase().trim();
        for (const card of root.querySelectorAll('.plugin-card[data-plugin-id]')) {
          const name = card.getAttribute('data-search-name') || '';
          const desc = card.getAttribute('data-search-desc') || '';
          const cat = card.getAttribute('data-search-cat') || '';
          const match = !q || name.includes(q) || desc.includes(q) || cat.includes(q);
          card.hidden = !match;
        }
      }, 200);
    });
  }
};

const initMktSort = () => {
  const sortSelect = document.getElementById('mkt-sort');
  if (sortSelect) {
    sortSelect.addEventListener('change', () => {
      const url = new URL(window.location.href);
      url.searchParams.set('sort', sortSelect.value);
      window.location.href = url.toString();
    });
  }
};

const handleLoadUsers = async (root, btn) => {
  const pluginId = btn.getAttribute('data-load-users');
  btn.disabled = true;
  btn.textContent = 'Loading...';
  try {
    const usersData = await apiFetch('/marketplace-plugins/' + encodeURIComponent(pluginId) + '/users');
    const users = usersData.users || usersData || [];
    const container = root.querySelector('[data-users-for="' + pluginId + '"]');
    if (container) {
      container.textContent = '';
      if (!users.length) {
        renderEmptyUsers(container);
      } else {
        renderUsersList(container, users);
      }
    }
    btn.hidden = true;
  } catch (err) {
    showToast(err.message || 'Failed to load users', 'error');
    btn.disabled = false;
    btn.textContent = 'Load Users';
  }
};

const renderEmptyUsers = (container) => {
  const noUsers = document.createElement('div');
  noUsers.className = 'sp-mkt-empty-users';
  noUsers.textContent = 'No users found';
  container.append(noUsers);
};

const renderUsersList = (container, users) => {
  const list = document.createElement('div');
  list.className = 'sp-mkt-users-list';
  for (const u of users) {
    list.append(renderUserRow(u));
  }
  container.append(list);
};

const renderUserRow = (u) => {
  const row = document.createElement('div');
  row.className = 'sp-mkt-user-row';
  const nameSpan = document.createElement('span');
  nameSpan.className = 'sp-mkt-user-name';
  nameSpan.textContent = u.display_name || 'Unknown';
  row.append(nameSpan);
  const eventBadge = document.createElement('span');
  eventBadge.className = 'badge badge-gray';
  eventBadge.textContent = (u.event_count || 0) + ' events';
  row.append(eventBadge);
  if (u.last_used) {
    const dateSpan = document.createElement('span');
    dateSpan.className = 'sp-mkt-user-date';
    dateSpan.textContent = new Date(u.last_used).toLocaleDateString();
    row.append(dateSpan);
  }
  return row;
};
