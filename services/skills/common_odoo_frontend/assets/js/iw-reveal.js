/**
 * Foodles Scroll Reveal - Intersection Observer for iw-reveal animations.
 * Drop this script at the end of any page using iw-reveal classes.
 * No dependencies. Works with Odoo website frontend.
 *
 * Usage: Add class "iw-reveal", "iw-reveal-left", "iw-reveal-right", or
 * "iw-reveal-scale" to any element. Wrap in a parent with "iw-stagger"
 * for staggered entrance animations.
 */
(function () {
  'use strict';

  var REVEAL_SELECTORS = '.iw-reveal, .iw-reveal-left, .iw-reveal-right, .iw-reveal-scale';

  var observer = new IntersectionObserver(
    function (entries) {
      entries.forEach(function (entry) {
        if (entry.isIntersecting) {
          entry.target.classList.add('iw-visible');
          observer.unobserve(entry.target);
        }
      });
    },
    { threshold: 0.15, rootMargin: '0px 0px -40px 0px' }
  );

  function init() {
    var elements = document.querySelectorAll(REVEAL_SELECTORS);
    elements.forEach(function (el) {
      observer.observe(el);
    });
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
