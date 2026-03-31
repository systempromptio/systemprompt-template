function initMobileMenu() {
  'use strict';

  const menuToggle = document.querySelector('.mobile-menu-toggle');
  const navLinks = document.querySelector('.nav-links');
  const docsSidebar = document.querySelector('.docs-sidebar');

  if (!menuToggle) return;

  const isDocsPage = !!docsSidebar;

  menuToggle.addEventListener('click', () => {
    const isExpanded = menuToggle.getAttribute('aria-expanded') === 'true';
    menuToggle.setAttribute('aria-expanded', !isExpanded);

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

  // Register with public overlay manager instead of per-module Escape handler
  window._overlays = window._overlays || [];
  if (!window._overlayEscapeInit) {
    window._overlayEscapeInit = true;
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        for (var i = window._overlays.length - 1; i >= 0; i--) {
          if (window._overlays[i]()) break;
        }
      }
    });
  }
  window._overlays.push(() => {
    if (document.body.classList.contains('menu-open')) {
      closeMenu();
      return true;
    }
    return false;
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
}

document.addEventListener('DOMContentLoaded', initMobileMenu);
