import { fillStars, rateSession, rateSkill, renderChart } from '../services/control-center-stats.js';
import { initActions, initRatings, initSuggestionDetails } from './admin-control-center-actions.js';
import { initBatch } from '../services/control-center-batch.js';
import { initTabs, initFilters } from './admin-control-center-panels.js';
import { initSSE, loadInitialData } from './admin-control-center-sse.js';
import { renderSparklines, initReportCountdown, initReportActions } from './admin-control-center-report.js';

initSSE();
loadInitialData();

initTabs(renderChart, renderSparklines);
initFilters();
initRatings(fillStars, rateSession, rateSkill);
initActions();
initBatch();
initSuggestionDetails();
renderSparklines();
initReportCountdown();
initReportActions();
initJobCountdown();

function initJobCountdown() {
  const fmt = (s) => {
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    return h + 'h ' + m + 'm';
  };
  const update = () => {
    const now = new Date();
    const secsInDay = now.getUTCHours() * 3600 + now.getUTCMinutes() * 60 + now.getUTCSeconds();
    const analysisSecs = 23 * 3600;
    const cleanupSecs = 24 * 3600;
    let toAnalysis = analysisSecs - secsInDay;
    if (toAnalysis < 0) toAnalysis += 86400;
    let toCleanup = cleanupSecs - secsInDay;
    if (toCleanup <= 0) toCleanup += 86400;
    const aEl = document.getElementById('cc-analysis-countdown');
    const cEl = document.getElementById('cc-cleanup-countdown');
    if (aEl) aEl.textContent = fmt(toAnalysis);
    if (cEl) cEl.textContent = fmt(toCleanup);
  };
  update();
  setInterval(update, 60000);
}

const refreshBtn = document.getElementById('cc-daily-refresh');
if (refreshBtn) {
  refreshBtn.addEventListener('click', (e) => {
    e.preventDefault();
    refreshBtn.classList.add('cc-daily-refresh-btn--loading');
    refreshBtn.textContent = '\u21bb refreshing\u2026';
    window.location.reload();
  });
}
