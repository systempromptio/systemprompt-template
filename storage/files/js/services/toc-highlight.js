const throttle = (func, limit) => {
  let inThrottle;
  return (...args) => {
    if (!inThrottle) {
      func(...args);
      inThrottle = true;
      setTimeout(() => { inThrottle = false; }, limit);
    }
  };
};

export const initTocHighlight = () => {
  const tocLinks = document.querySelectorAll('.toc-content a, .toc-content--mobile a');
  if (tocLinks.length) {
    const headings = [];
    for (const link of tocLinks) {
      const href = link.getAttribute('href');
      if (href?.startsWith('#')) {
        const heading = document.getElementById(href.slice(1));
        if (heading) headings.push({ link, heading });
      }
    }
    if (headings.length) {
      const updateActiveLink = () => {
        const scrollTop = window.scrollY;
        let activeIndex = 0;
        for (const [index, item] of headings.entries()) {
          if (item.heading.offsetTop - 100 <= scrollTop) activeIndex = index;
        }
        for (const link of tocLinks) link.classList.remove('active');
        headings[activeIndex].link.classList.add('active');
      };

      window.addEventListener('scroll', throttle(updateActiveLink, 100));
      updateActiveLink();
    }
  }
};
