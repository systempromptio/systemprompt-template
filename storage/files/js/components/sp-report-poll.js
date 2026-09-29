// While a report is being written the page shows the loading widget and asks
// `/status` every two seconds (five after half a minute) until the row is
// generated — then it reloads itself with the verdict — or failed, when the
// error is shown in place and the Retry button in the header takes over.

import { rawResponse } from '../services/api.js';

const FAST_MS = 2000;
const SLOW_MS = 5000;
const SLOW_AFTER_MS = 30000;

const describeElapsed = (ms) => {
  const seconds = Math.round(ms / 1000);
  if (seconds < 60) return `${seconds}s so far`;
  return `${Math.floor(seconds / 60)}m ${seconds % 60}s so far`;
};

const fail = (widget, message) => {
  widget.classList.add('is-failed');
  const detail = widget.querySelector('[data-report-poll-detail]');
  if (detail) detail.textContent = message || 'The report could not be written. Use Retry to start a fresh one.';
  const hint = widget.querySelector('[data-report-poll-elapsed]');
  if (hint) hint.textContent = 'Retry starts a fresh report over the same scope.';
};

export const initReportPoll = () => {
  const widget = document.querySelector('[data-report-poll]');
  if (!widget) return;
  const url = widget.dataset.reportPoll;
  const hint = widget.querySelector('[data-report-poll-elapsed]');
  const started = Date.now();
  const tick = async () => {
    try {
      const resp = await rawResponse(url);
      if (!resp.ok) throw new Error(`status ${resp.status}`);
      const state = await resp.json();
      if (state.status === 'generated') {
        window.location.reload();
        return;
      }
      if (state.status === 'failed') {
        fail(widget, state.error);
        return;
      }
      if (hint) hint.textContent = `One structured call to the judge model — ${describeElapsed(Date.now() - started)}.`;
    } catch (error) {
      if (hint) hint.textContent = `Still waiting (${error.message}); the page keeps checking.`;
    }
    const elapsed = Date.now() - started;
    window.setTimeout(tick, elapsed > SLOW_AFTER_MS ? SLOW_MS : FAST_MS);
  };
  window.setTimeout(tick, FAST_MS);
};

initReportPoll();
