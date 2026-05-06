(function () {
    'use strict';

    function applyIntensity(root) {
        var max = parseInt(root.getAttribute('data-max-cell') || '0', 10);
        if (!Number.isFinite(max) || max <= 0) return;
        var cells = root.querySelectorAll('.heatmap__cell[data-count]');
        for (var i = 0; i < cells.length; i++) {
            var cell = cells[i];
            var n = parseInt(cell.getAttribute('data-count') || '0', 10);
            if (!Number.isFinite(n) || n <= 0) continue;
            // Map count→intensity using a sqrt curve so a single hit is still
            // visible while extreme cells don't saturate everything else.
            var intensity = Math.round(Math.sqrt(n / max) * 70);
            cell.style.setProperty('--intensity', String(intensity));
        }
    }

    function init() {
        var roots = document.querySelectorAll('.heatmap[data-max-cell]');
        for (var i = 0; i < roots.length; i++) {
            applyIntensity(roots[i]);
        }
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
