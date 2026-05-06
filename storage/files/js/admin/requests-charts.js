(function () {
    'use strict';

    function applyHistogram(root) {
        var max = parseInt(root.getAttribute('data-histogram-max') || '0', 10);
        if (!Number.isFinite(max) || max <= 0) max = 1;
        var bars = root.querySelectorAll('.latency-histogram__bar');
        for (var i = 0; i < bars.length; i++) {
            var bar = bars[i];
            var c = parseInt(bar.getAttribute('data-count') || '0', 10);
            var ratio = (Number.isFinite(c) && c > 0) ? c / max : 0;
            bar.style.setProperty('--ratio', ratio.toFixed(4));
        }
    }

    function applyCostSpark(root) {
        var max = parseInt(root.getAttribute('data-cost-max') || '0', 10);
        if (!Number.isFinite(max) || max <= 0) max = 1;
        var bars = root.querySelectorAll('.cost-spark__bar');
        for (var i = 0; i < bars.length; i++) {
            var bar = bars[i];
            var c = parseInt(bar.getAttribute('data-cost') || '0', 10);
            var pct = (Number.isFinite(c) && c > 0) ? (c / max) * 100 : 0;
            bar.style.setProperty('--cost', pct.toFixed(2));
        }
    }

    function init() {
        var hists = document.querySelectorAll('.latency-histogram[data-histogram-max]');
        for (var i = 0; i < hists.length; i++) applyHistogram(hists[i]);
        var sparks = document.querySelectorAll('.cost-spark[data-cost-max]');
        for (var j = 0; j < sparks.length; j++) applyCostSpark(sparks[j]);
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
