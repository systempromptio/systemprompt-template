(function() {
  'use strict';

  document.addEventListener('DOMContentLoaded', init);

  function init() {
    initTocHighlight();
    initNavActiveState();
    initSmoothScroll();
    initCollapsibleNav();
    initMobileToc();
    initPagination();
    initExportMarkdown();
  }

  function initMobileToc() {
    const mobileDetails = document.querySelector('.toc-mobile-details');
    if (!mobileDetails) return;

    mobileDetails.querySelectorAll('a').forEach(function(link) {
      link.addEventListener('click', function() {
        setTimeout(function() {
          mobileDetails.removeAttribute('open');
        }, 100);
      });
    });
  }

  function initTocHighlight() {
    const tocLinks = document.querySelectorAll('.toc-content a, .toc-content--mobile a');
    if (!tocLinks.length) return;

    const headings = [];
    tocLinks.forEach(link => {
      const href = link.getAttribute('href');
      if (href && href.startsWith('#')) {
        const heading = document.getElementById(href.slice(1));
        if (heading) {
          headings.push({ link, heading });
        }
      }
    });

    if (!headings.length) return;

    function updateActiveLink() {
      const scrollTop = window.scrollY;
      const offset = 100;

      let activeIndex = 0;
      headings.forEach((item, index) => {
        if (item.heading.offsetTop - offset <= scrollTop) {
          activeIndex = index;
        }
      });

      tocLinks.forEach(link => link.classList.remove('active'));
      headings[activeIndex].link.classList.add('active');
    }

    window.addEventListener('scroll', throttle(updateActiveLink, 100));
    updateActiveLink();
  }

  function initNavActiveState() {
    const currentPath = window.location.pathname;
    const navLinks = document.querySelectorAll('.docs-nav-link');

    navLinks.forEach(link => {
      const href = link.getAttribute('href');
      if (href === currentPath) {
        link.classList.add('docs-nav-link--active');

        const details = link.closest('details');
        if (details) {
          details.open = true;
        }
      }
    });
  }

  function initSmoothScroll() {
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
      anchor.addEventListener('click', function(e) {
        const href = this.getAttribute('href');
        if (href === '#') return;

        const target = document.getElementById(href.slice(1));
        if (!target) return;

        e.preventDefault();

        const offset = 80;
        const targetPosition = target.offsetTop - offset;

        window.scrollTo({
          top: targetPosition,
          behavior: 'smooth'
        });

        history.pushState(null, null, href);
      });
    });
  }

  function initCollapsibleNav() {
    const currentPath = window.location.pathname;
    const details = document.querySelectorAll('.docs-nav-details');

    details.forEach(detail => {
      const links = detail.querySelectorAll('.docs-nav-link');
      links.forEach(link => {
        if (link.getAttribute('href') === currentPath) {
          detail.open = true;
        }
      });
    });
  }

  function throttle(func, limit) {
    let inThrottle;
    return function() {
      const args = arguments;
      const context = this;
      if (!inThrottle) {
        func.apply(context, args);
        inThrottle = true;
        setTimeout(() => inThrottle = false, limit);
      }
    };
  }

  function initPagination() {
    const paginationNav = document.querySelector('.docs-pagination');
    const sidebarLinks = document.querySelectorAll('.docs-sidebar .docs-nav-link');

    if (!sidebarLinks.length) return;

    const currentPath = window.location.pathname;
    const links = Array.from(sidebarLinks);

    let currentIndex = -1;
    links.forEach((link, index) => {
      if (link.getAttribute('href') === currentPath) {
        currentIndex = index;
      }
    });

    if (currentIndex === -1) return;

    const prevLink = currentIndex > 0 ? links[currentIndex - 1] : null;
    const nextLink = currentIndex < links.length - 1 ? links[currentIndex + 1] : null;

    let nav = paginationNav;
    if (!nav) {
      nav = document.createElement('nav');
      nav.className = 'docs-pagination';
      nav.setAttribute('aria-label', 'Pagination');

      const article = document.querySelector('.docs-article');
      if (article) {
        article.appendChild(nav);
      }
    }

    nav.innerHTML = '';

    if (prevLink) {
      const prevHref = prevLink.getAttribute('href');
      const prevTitle = prevLink.textContent.trim();
      nav.innerHTML += `
        <a href="${prevHref}" class="docs-pagination-link docs-pagination-prev">
          <span class="docs-pagination-label">Previous</span>
          <span class="docs-pagination-title">${prevTitle}</span>
        </a>
      `;
    }

    if (nextLink) {
      const nextHref = nextLink.getAttribute('href');
      const nextTitle = nextLink.textContent.trim();
      nav.innerHTML += `
        <a href="${nextHref}" class="docs-pagination-link docs-pagination-next">
          <span class="docs-pagination-label">Next</span>
          <span class="docs-pagination-title">${nextTitle}</span>
        </a>
      `;
    }
  }

  function initExportMarkdown() {
    const btn = document.querySelector('.docs-export-btn');
    if (!btn) return;

    btn.addEventListener('click', async function() {
      const title = document.querySelector('.docs-header h1');
      const description = document.querySelector('.docs-description');
      const content = document.querySelector('.docs-content');

      let markdown = '';
      if (title) markdown += `# ${title.textContent}\n\n`;
      if (description) markdown += `${description.textContent}\n\n`;
      if (content) markdown += content.innerText;

      await navigator.clipboard.writeText(markdown);
      showCopiedFeedback(btn);
    });

    function showCopiedFeedback(btn) {
      btn.classList.add('copied');
      setTimeout(() => {
        btn.classList.remove('copied');
      }, 2000);
    }
  }
})();
