export const initSidebar = () => {
  const toggle = document.querySelector('.sidebar-toggle');
  const sidebar = document.getElementById('admin-sidebar');
  const overlay = document.getElementById('sidebar-overlay');
  const closeBtn = document.querySelector('.sidebar-close-btn');
  if (toggle && sidebar) {
    const close = () => {
      sidebar.classList.remove('open');
      overlay?.classList.remove('open');
      toggle.setAttribute('aria-expanded', 'false');
    };

    toggle.addEventListener('click', (e) => {
      e.stopPropagation();
      const isOpen = sidebar.classList.contains('open');
      if (isOpen) {
        close();
      } else {
        sidebar.classList.add('open');
        overlay?.classList.add('open');
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
