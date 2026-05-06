// Time range picker — progressive enhancement.
// Reveals custom inputs when "Custom" preset is clicked without a full
// page nav, so users can choose dates before submitting.
(function () {
  'use strict';

  function init(root) {
    var customPanel = root.querySelector('[data-time-range-custom]');
    if (!customPanel) return;

    root.querySelectorAll('.time-range__btn').forEach(function (btn) {
      btn.addEventListener('click', function (event) {
        if (btn.dataset.preset !== 'custom') return;
        // Only intercept if custom panel is currently hidden — otherwise
        // let the link navigate normally to apply the custom range.
        if (!customPanel.hasAttribute('hidden')) return;
        event.preventDefault();
        customPanel.removeAttribute('hidden');
        root.querySelectorAll('.time-range__btn').forEach(function (b) {
          b.classList.remove('time-range__btn--active');
          b.removeAttribute('aria-current');
        });
        btn.classList.add('time-range__btn--active');
        btn.setAttribute('aria-current', 'true');
        var fromInput = customPanel.querySelector('[data-time-range-from]');
        if (fromInput) fromInput.focus();
      });
    });
  }

  function boot() {
    document.querySelectorAll('[data-time-range]').forEach(init);
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', boot);
  } else {
    boot();
  }
})();
