(function() {
  'use strict';

  document.addEventListener('DOMContentLoaded', function() {
    const containers = document.querySelectorAll('[data-component="featured-image"]');

    containers.forEach(function(container) {
      const img = container.querySelector('img');
      const placeholder = container.querySelector('.image-placeholder');

      if (img && placeholder) {
        img.addEventListener('error', function() {
          img.classList.add('is-hidden');
          placeholder.classList.remove('is-hidden');
        });
      }
    });
  });
})();
