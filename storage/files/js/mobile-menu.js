(function() {
  'use strict';

  document.addEventListener('DOMContentLoaded', () => {
    const menuToggle = document.querySelector('.mobile-menu-toggle');
    const navLinks = document.querySelector('.nav-links');
    const docsSidebar = document.querySelector('.docs-sidebar');

    if (!menuToggle) return;

    const isDocsPage = !!docsSidebar;

    menuToggle.addEventListener('click', function() {
      const isExpanded = this.getAttribute('aria-expanded') === 'true';
      this.setAttribute('aria-expanded', !isExpanded);

      if (isDocsPage) {
        docsSidebar.classList.toggle('is-open');
      } else {
        if (navLinks) {
          navLinks.classList.toggle('is-open');
        }
      }

      document.body.classList.toggle('menu-open');
    });

    if (isDocsPage && docsSidebar) {
      docsSidebar.querySelectorAll('a').forEach((link) => {
        link.addEventListener('click', () => {
          closeMenu();
        });
      });
    } else if (navLinks) {
      navLinks.querySelectorAll('a').forEach((link) => {
        link.addEventListener('click', () => {
          closeMenu();
        });
      });
    }

    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && document.body.classList.contains('menu-open')) {
        closeMenu();
      }
    });

    function closeMenu() {
      if (isDocsPage && docsSidebar) {
        docsSidebar.classList.remove('is-open');
      } else if (navLinks) {
        navLinks.classList.remove('is-open');
      }
      if (menuToggle) {
        menuToggle.setAttribute('aria-expanded', 'false');
      }
      document.body.classList.remove('menu-open');
    }
  });
})();
