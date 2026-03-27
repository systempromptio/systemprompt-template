(function() {
  'use strict';

  document.addEventListener('DOMContentLoaded', () => {
    const containers = document.querySelectorAll('[data-component="featured-image"]');

    containers.forEach((container) => {
      const img = container.querySelector('img');
      const placeholder = container.querySelector('.image-placeholder');

      if (img && placeholder) {
        img.addEventListener('error', () => {
          img.classList.add('is-hidden');
          placeholder.classList.remove('is-hidden');
        });
      }
    });
  });
})();
