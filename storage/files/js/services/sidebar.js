// Why: two nav links can render for the same page id — `/admin` and
// `/admin/analytics` are both the dashboard — so the server marks a best guess
// and the exact pathname settles it here. Only one link may be active, so a
// match clears the others. Record rows (`.sp-nav-child`) are left out: their
// href is the page itself, and the server already marked them.
const markActiveLink = (sidebar) => {
  const links = [...sidebar.querySelectorAll('nav a:not(.sp-nav-child)')];
  const exact = links.find((link) => new URL(link.href, window.location.origin).pathname === window.location.pathname);
  if (!exact) return;
  for (const link of links) {
    link.classList.toggle('is-active', link === exact);
    if (link === exact) {
      link.setAttribute('aria-current', 'page');
    } else {
      link.removeAttribute('aria-current');
    }
  }
};

export const initSidebar = () => {
  const toggle = document.querySelector('.sp-topbar__nav-toggle');
  const sidebar = document.getElementById('sp-admin-sidebar');
  const overlay = document.getElementById('sp-sidebar-overlay');
  const closeBtn = document.querySelector('.sp-sidebar-close-btn');
  if (sidebar) markActiveLink(sidebar);
  if (toggle && sidebar) {
    const close = () => {
      sidebar.classList.remove('is-open');
      overlay?.classList.remove('is-open');
      toggle.setAttribute('aria-expanded', 'false');
    };

    toggle.addEventListener('click', (e) => {
      e.stopPropagation();
      const isOpen = sidebar.classList.contains('is-open');
      if (isOpen) {
        close();
      } else {
        sidebar.classList.add('is-open');
        overlay?.classList.add('is-open');
        toggle.setAttribute('aria-expanded', 'true');
      }
    });

    closeBtn?.addEventListener('click', (e) => {
      e.stopPropagation();
      close();
    });
    overlay?.addEventListener('click', (e) => {
      e.stopPropagation();
      close();
    });

    for (const link of sidebar.querySelectorAll('nav a')) {
      link.addEventListener('click', close);
    }

  }
};
