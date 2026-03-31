import { injectSuggestionRow } from './cc-stats-suggestions.js';

export const rateSession = (sessionId, rating, outcome) => {
  const body = { session_id: sessionId, rating };
  if (outcome !== undefined) body.outcome = outcome;
  fetch('/control-center/api/rate-session', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  }).catch(() => {});
};

export const rateSkill = (skillName, rating) => {
  fetch('/control-center/api/rate-skill', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ skill_name: skillName, rating }),
  }).catch(() => {});
};

export const updateSessionStatus = async (sessionId, status) => {
  try {
    const res = await fetch('/control-center/api/session-status', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ session_id: sessionId, status }),
    });
    if (res.ok) applyStatusToCard(sessionId, status);
  } catch (_networkErr) { /* fire-and-forget status update */ }
};

export const analyseSession = async (sessionId) => {
  const card = document.querySelector('[data-session-id="' + sessionId + '"]');
  if (card) {
    const right = card.querySelector('.sf-session-right');
    if (right) {
      const loading = document.createElement('span');
      loading.className = 'sf-session-status-label sf-session-status-label--analysing';
      loading.textContent = 'analysing\u2026';
      right.append(loading);
    }
  }
  try {
    const res = await fetch('/control-center/api/analyse-session', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ session_id: sessionId }),
    });
    if (!res.ok) throw new Error('Analysis failed');
    const data = await res.json();
    if (card) {
      const right = card.querySelector('.sf-session-right');
      if (right) {
        const loading = right.querySelector('.sf-session-status-label--analysing');
        if (loading) loading.remove();
        const label = document.createElement('span');
        label.className = 'sf-session-status-label sf-session-status-label--analysed';
        label.textContent = 'analysed';
        right.append(label);
      }
      const menu = card.querySelector('.sf-action-menu');
      if (menu) {
        const analyseBtn = menu.querySelector('[data-action="analyse"]');
        if (analyseBtn) analyseBtn.remove();
      }
    }
    injectSuggestionRow(data);
  } catch (_err) {
    if (card) {
      const right = card.querySelector('.sf-session-right');
      if (right) {
        const loading = right.querySelector('.sf-session-status-label--analysing');
        if (loading) loading.textContent = 'analysis failed';
      }
    }
  }
};

const applyStatusToCard = (sessionId, status) => {
  const card = document.querySelector('[data-session-id="' + sessionId + '"]');
  if (card) {
    card.setAttribute('data-status', status);
    const header = card.querySelector('.sf-session-header');
    if (header) {
      for (const el of header.querySelectorAll('.sf-session-status-label')) el.remove();
      if (status === 'completed') {
        const label = document.createElement('span');
        label.className = 'sf-session-status-label sf-session-status-label--' + status;
        label.textContent = status;
        const actions = header.querySelector('.sf-session-actions');
        if (actions) header.insertBefore(label, actions);
      }
    }
    if (status === 'deleted') {
      card.classList.add('sf-session--leaving');
      setTimeout(() => { card.remove(); }, 350);
    }
  }
};
