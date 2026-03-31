import { on } from './events.js';

let portal = null;
let activeDropdown = null;
let activeMenu = null;
let activeTrigger = null;

const init = () => {
  if (!portal) {
    portal = document.createElement('div');
    portal.id = 'dropdown-portal';
    portal.classList.add('sp-dropdown-portal');
    document.body.append(portal);

    document.addEventListener('click', (e) => {
      if (activeDropdown) {
        if (!activeDropdown.contains(e.target)) {
          if (!e.target.closest('[data-action="menu"]')) {
            close();
          }
        }
      }
    }, true);

  }
};

export const open = (triggerBtn) => {
  close();
  const menu = triggerBtn.closest('.actions-menu');
  if (menu) {
    const dropdown = menu.querySelector('.actions-dropdown');
    if (dropdown) {
      const rect = triggerBtn.getBoundingClientRect();
      const clone = dropdown.cloneNode(true);
      clone.classList.add('sp-dropdown-menu');
      clone.style.top = (rect.bottom + 4) + 'px';
      clone.style.right = (window.innerWidth - rect.right) + 'px';
      clone.setAttribute('data-portal-dropdown', 'true');

      portal.append(clone);
      activeMenu = menu;
      activeDropdown = clone;
      activeTrigger = triggerBtn;
      menu.classList.add('open');
    }
  }
};

export const close = () => {
  activeDropdown?.remove();
  activeDropdown = null;
  activeMenu?.classList.remove('open');
  activeMenu = null;
  activeTrigger = null;
};

export const closeAllMenus = () => {
  close();
  for (const m of document.querySelectorAll('.actions-menu.open')) {
    m.classList.remove('open');
  }
  const installMenu = document.getElementById('install-menu');
  if (installMenu?.classList.contains('open')) {
    installMenu.classList.remove('open');
    installMenu.querySelector('.install-trigger')?.setAttribute('aria-expanded', 'false');
  }
  const headerActions = document.getElementById('header-actions');
  if (headerActions?.classList.contains('open')) {
    headerActions.classList.remove('open');
    headerActions.querySelector('.header-actions-toggle')?.setAttribute('aria-expanded', 'false');
  }
  const sidebar = document.getElementById('admin-sidebar');
  if (sidebar?.classList.contains('open')) {
    sidebar.classList.remove('open');
    document.getElementById('sidebar-overlay')?.classList.remove('open');
    document.querySelector('.sidebar-toggle')?.setAttribute('aria-expanded', 'false');
  }
  for (const p of document.querySelectorAll('.sf-action-menu--portal')) p.remove();
};

export const initDropdown = () => {
  init();
  on('click', '[data-action="menu"]', (e, trigger) => {
    e.stopPropagation();
    open(trigger);
  }, { exclusive: true });
};
