document.addEventListener('DOMContentLoaded', () => {
    initTabs();
    initSparklines();
    initGenerateReport();
});

function initTabs() {
    const tabs = document.querySelectorAll('.profile-tabs .sp-tab');
    if (!tabs.length) return;

    const urlTab = new URL(window.location).searchParams.get('tab');
    if (urlTab === 'ai') {
        for (const t of tabs) {
            const active = t.dataset.tab === 'profile-ai';
            t.classList.toggle('sp-tab--active', active);
            t.setAttribute('aria-selected', active ? 'true' : 'false');
        }
        for (const p of document.querySelectorAll('.sp-tab-panel')) {
            if (p.id === 'tab-profile-ai') {
                p.removeAttribute('hidden');
            } else {
                p.setAttribute('hidden', '');
            }
        }
    }

    for (const tab of tabs) {
        tab.addEventListener('click', () => {
            const target = tab.dataset.tab;
            for (const t of tabs) {
                t.classList.remove('sp-tab--active');
                t.setAttribute('aria-selected', 'false');
            }
            for (const p of document.querySelectorAll('.sp-tab-panel')) p.setAttribute('hidden', '');
            tab.classList.add('sp-tab--active');
            tab.setAttribute('aria-selected', 'true');
            const panel = document.getElementById('tab-' + target);
            if (panel) panel.removeAttribute('hidden');

            const url = new URL(window.location);
            if (target === 'profile-ai') {
                url.searchParams.set('tab', 'ai');
            } else {
                url.searchParams.delete('tab');
            }
            history.replaceState(null, '', url);
        });
    }
}

function initSparklines() {
    document.querySelectorAll('.profile-sparkline').forEach(svg => {
        const raw = svg.dataset.values || '';
        const globalAvg = parseFloat(svg.dataset.global || '0');
        const values = raw.split(',').map(Number).filter(v => !isNaN(v));
        if (values.length < 2) return;

        const w = 120;
        const h = 30;
        const padding = 2;
        const allValues = [...values, globalAvg].filter(v => v > 0);
        const min = Math.min(...allValues) * 0.9;
        const max = Math.max(...allValues) * 1.1 || 1;
        const range = max - min || 1;

        const toX = (i) => padding + (i / (values.length - 1)) * (w - padding * 2);
        const toY = (v) => h - padding - ((v - min) / range) * (h - padding * 2);

        if (globalAvg > 0) {
            const gy = toY(globalAvg);
            const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
            line.setAttribute('x1', padding);
            line.setAttribute('y1', gy);
            line.setAttribute('x2', w - padding);
            line.setAttribute('y2', gy);
            line.setAttribute('stroke', 'var(--sp-text-tertiary, #ccc)');
            line.setAttribute('stroke-width', '1');
            line.setAttribute('stroke-dasharray', '3,3');
            svg.appendChild(line);
        }

        const points = values.map((v, i) => `${toX(i)},${toY(v)}`).join(' ');
        const polyline = document.createElementNS('http://www.w3.org/2000/svg', 'polyline');
        polyline.setAttribute('points', points);
        polyline.setAttribute('fill', 'none');
        polyline.setAttribute('stroke', 'var(--sp-accent, #6366f1)');
        polyline.setAttribute('stroke-width', '1.5');
        polyline.setAttribute('stroke-linecap', 'round');
        polyline.setAttribute('stroke-linejoin', 'round');
        svg.appendChild(polyline);

        const last = values[values.length - 1];
        const circle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
        circle.setAttribute('cx', toX(values.length - 1));
        circle.setAttribute('cy', toY(last));
        circle.setAttribute('r', '2.5');
        circle.setAttribute('fill', 'var(--sp-accent, #6366f1)');
        svg.appendChild(circle);
    });
}

function initGenerateReport() {
    const btn = document.getElementById('generate-profile-report');
    if (!btn) return;

    btn.addEventListener('click', async () => {
        btn.disabled = true;
        btn.textContent = 'Generating...';

        try {
            const res = await fetch('/control-center/api/generate-profile-report', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
            });

            if (res.ok) {
                const url = new URL(window.location);
                url.searchParams.set('tab', 'ai');
                window.location.href = url.toString();
            } else {
                const data = await res.json().catch(() => ({}));
                const msg = data.message || 'Failed to generate analysis';
                btn.textContent = msg;
                setTimeout(() => {
                    btn.textContent = 'Regenerate';
                    btn.disabled = false;
                }, 4000);
            }
        } catch {
            btn.textContent = 'Error - try again';
            btn.disabled = false;
        }
    });
}
