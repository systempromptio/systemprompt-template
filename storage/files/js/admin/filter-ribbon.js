// Identity filter ribbon — progressive enhancement: typeahead inside dropdown
// groups when option lists exceed 20 items. SSR-rendered checkboxes already
// work without JS; this only filters which <li> entries are visible.
(function () {
  'use strict';

  var TYPEAHEAD_THRESHOLD = 20;

  function initGroup(group) {
    var search = group.querySelector('[data-filter-typeahead]');
    var list = group.querySelector('[data-filter-list]');
    if (!search || !list) return;

    var items = Array.prototype.slice.call(
      list.querySelectorAll('.filter-ribbon__group-item'),
    );
    if (items.length <= TYPEAHEAD_THRESHOLD) {
      search.hidden = true;
      return;
    }

    search.addEventListener('input', function () {
      var query = search.value.trim().toLowerCase();
      items.forEach(function (item) {
        var label = (item.dataset.label || '').toLowerCase();
        if (!query || label.indexOf(query) !== -1) {
          item.hidden = false;
        } else {
          item.hidden = true;
        }
      });
    });
  }

  function closeOthersOnOpen(root) {
    var groups = root.querySelectorAll('details.filter-ribbon__group');
    groups.forEach(function (g) {
      g.addEventListener('toggle', function () {
        if (!g.open) return;
        groups.forEach(function (other) {
          if (other !== g) other.open = false;
        });
      });
    });
  }

  function init(root) {
    root.querySelectorAll('details.filter-ribbon__group').forEach(initGroup);
    closeOthersOnOpen(root);
  }

  function boot() {
    document.querySelectorAll('[data-filter-ribbon]').forEach(init);
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', boot);
  } else {
    boot();
  }
})();
