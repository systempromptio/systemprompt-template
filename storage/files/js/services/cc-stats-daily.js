import { escapeHtml } from '../utils/dom.js';

const formatApm = (val) => {
  if (val == null) return '--';
  return Number(val).toFixed(1);
};

const formatRound = (val) => {
  if (val == null) return '--';
  const n = Number(val);
  return Number.isInteger(n) ? String(n) : n.toFixed(1);
};

const setText = (id, text) => {
  const el = document.getElementById(id);
  if (el) el.textContent = text;
};

export const updateDailySummary = (data) => {
  const el = document.getElementById('cc-daily-summary');
  if (el) {
    const metrics = el.querySelectorAll('.cc-daily-metrics .cc-daily-metric-value');
    if (metrics[0]) metrics[0].textContent = data.sessions_count || 0;
    if (metrics[1]) metrics[1].textContent = data.analysed_count || 0;
    if (metrics[2]) metrics[2].textContent = data.avg_quality || '0.0';
    if (metrics[3]) metrics[3].textContent = data.goals_achieved || 0;
    const rec = el.querySelector('.cc-daily-recommendation');
    if (rec && data.top_recommendation) rec.textContent = data.top_recommendation;

    const achSection = document.getElementById('cc-achievements-today-section');
    const achList = document.getElementById('cc-achievements-today-list');
    if (achSection && achList && Array.isArray(data.achievements_today) && data.achievements_today.length > 0) {
      achSection.hidden = false;
      achList.innerHTML = '';
      for (const a of data.achievements_today) {
        const badge = document.createElement('span');
        badge.className = 'cc-achievement-badge';
        badge.textContent = a;
        achList.append(badge);
      }
    }

    const corrSection = document.getElementById('cc-apm-correlation-section');
    const corrEl = document.getElementById('cc-apm-correlation');
    if (corrSection && corrEl && data.apm_correlation) {
      corrSection.hidden = false;
      const c = data.apm_correlation;
      const text = document.createElement('span');
      text.className = 'cc-apm-correlation-text';
      text.textContent = 'High APM: ' + (c.high_apm_success_pct ?? '--') + '% success | Low APM: ' + (c.low_apm_success_pct ?? '--') + '% success';
      corrEl.innerHTML = '';
      corrEl.append(text);
    }

    if (data.avg_apm != null) {
      const apmMetric = document.getElementById('cc-eapm-current');
      if (apmMetric) apmMetric.textContent = formatApm(data.avg_apm);
    }
  }
};

export const updateApmMetrics = (data) => {
  if (data) {
    const apm = data.apm;
    if (apm) {
      setText('cc-apm-current', formatApm(apm.current));
      setText('cc-apm-peak', formatApm(apm.peak));
      setText('cc-eapm-current', formatApm(apm.avg));
    }

    const conc = data.concurrency;
    if (conc) {
      setText('cc-concurrency-current', formatRound(conc.current));
      setText('cc-concurrency-peak', formatRound(conc.peak));
    }

    const tp = data.throughput;
    if (tp) {
      setText('cc-throughput-total', tp.total_display || '--');
      setText('cc-throughput-rate', tp.rate_display || '--');
    }

    setText('cc-tool-diversity', data.tool_diversity ?? '--');
    setText('cc-multitasking', formatRound(data.multitasking_score));
  }
};

export const updatePerformanceSummary = (data) => {
  if (data) {
    setText('cc-perf-sessions', data.total_sessions ?? '--');
    setText('cc-perf-actions', data.total_actions ?? '--');
    setText('cc-perf-error-rate', (data.error_rate_pct ?? '--') + '%');
    setText('cc-perf-active-min', data.active_minutes ?? '--');
  }
};
