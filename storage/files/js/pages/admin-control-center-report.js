import { buildReportHtml, buildReportText } from './admin-cc-report-build.js';
import { showToast } from '../services/toast.js';

export const renderSparklines = () => {
  for (const svg of document.querySelectorAll('.cc-report-sparkline')) {
    if (svg.querySelector('polyline')) continue;
    const raw = svg.getAttribute('data-values');
    if (!raw) continue;
    const values = raw.split(',').map(Number).filter(v => !isNaN(v));
    if (values.length < 2) continue;
    const min = Math.min(...values);
    const max = Math.max(...values);
    const range = max - min || 1;
    const n = values.length;
    const points = values.map((v, i) => {
      const x = (i / (n - 1)) * 60;
      const y = 18 - ((v - min) / range) * 16 + 2;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    }).join(' ');
    const ns = 'http://www.w3.org/2000/svg';
    const polyline = document.createElementNS(ns, 'polyline');
    polyline.setAttribute('points', points);
    svg.append(polyline);
    const lastY = (18 - ((values[n - 1] - min) / range) * 16 + 2).toFixed(1);
    const circle = document.createElementNS(ns, 'circle');
    circle.setAttribute('cx', '60.0');
    circle.setAttribute('cy', lastY);
    circle.setAttribute('r', '2');
    svg.append(circle);
  }
};

export const initReportCountdown = () => {
  const el = document.getElementById('cc-report-countdown');
  if (!el) return;
  const targetHour = parseInt(el.getAttribute('data-target-hour') || '23', 10);
  const valueEl = document.getElementById('cc-report-countdown-value');
  if (valueEl) {
    const update = () => {
      const now = new Date();
      let diff = ((targetHour - now.getUTCHours()) * 3600) - (now.getUTCMinutes() * 60) - now.getUTCSeconds();
      if (diff <= 0) diff += 86400;
      const h = Math.floor(diff / 3600);
      const m = Math.floor((diff % 3600) / 60);
      valueEl.textContent = h + 'h ' + m + 'm';
    };
    update();
    setInterval(update, 60000);
  }
};

export const initReportActions = () => {
  const printBtn = document.getElementById('cc-report-print');
  if (printBtn) {
    printBtn.addEventListener('click', () => {
      const reportTab = document.getElementById('tab-report');
      const wasHidden = reportTab && reportTab.hidden;
      if (wasHidden) reportTab.hidden = false;
      renderSparklines();
      window.print();
      if (wasHidden) reportTab.hidden = true;
    });
  }
  const shareBtn = document.getElementById('cc-report-share');
  const shareMenu = document.getElementById('cc-report-share-menu');
  if (shareBtn && shareMenu) {
    shareBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      shareMenu.classList.toggle('cc-report-share-menu--open');
    });
    document.addEventListener('click', () => shareMenu.classList.remove('cc-report-share-menu--open'));
    shareMenu.addEventListener('click', (e) => e.stopPropagation());
    for (const item of shareMenu.querySelectorAll('.cc-report-share-item')) {
      item.addEventListener('click', async () => {
        const action = item.dataset.action;
        shareMenu.classList.remove('cc-report-share-menu--open');
        if (action === 'copy-html') {
          const html = buildReportHtml();
          try {
            await navigator.clipboard.write([new ClipboardItem({
              'text/html': new Blob([html], { type: 'text/html' }),
              'text/plain': new Blob([html], { type: 'text/plain' }),
            })]);
            showToast('Report HTML copied to clipboard');
          } catch {
            await navigator.clipboard.writeText(html);
            showToast('Report HTML copied to clipboard');
          }
        }
        if (action === 'download-pdf') {
          const html = buildReportHtml();
          const w = window.open('', '_blank');
          if (w) {
            w.document.write(html);
            w.document.close();
            w.addEventListener('load', () => { w.print(); });
            setTimeout(() => w.print(), 500);
            showToast('Use "Save as PDF" in the print dialog');
          }
        }
        if (action === 'download-html') {
          const html = buildReportHtml();
          const dateEl = document.querySelector('#tab-report .cc-report-date');
          const date = dateEl?.textContent?.trim()?.replace(/\s+/g, '-') || 'report';
          const blob = new Blob([html], { type: 'text/html' });
          const url = URL.createObjectURL(blob);
          const a = document.createElement('a');
          a.href = url;
          a.download = `systemprompt-report-${date}.html`;
          a.click();
          URL.revokeObjectURL(url);
          showToast('Report downloaded as HTML');
        }
        if (action === 'copy-text') {
          const text = buildReportText();
          await navigator.clipboard.writeText(text);
          showToast('Report text copied to clipboard');
        }
      });
    }
  }
};
