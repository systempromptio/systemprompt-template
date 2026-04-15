import { escapeHtml } from '../utils/dom.js';

export const setStatValue = (key, val) => {
  const el = document.querySelector('[data-stat="' + key + '"] .cc-stat-value');
  if (el && val !== undefined) el.textContent = val;
};

export const updateSkillCards = (skills) => {
  if (skills?.length) {
    const table = document.querySelector('#tab-usage .cc-insight-table tbody');
    if (table) {
      table.innerHTML = '';
      for (const s of skills) {
        const tr = document.createElement('tr');
        const hasScore = s.scored_sessions > 0;
        const score = hasScore ? Number(s.avg_effectiveness).toFixed(1) : '\u2014';
        const goals = hasScore ? Math.round(s.goal_achievement_pct) + '%' : '\u2014';
        const rounded = Math.round(s.avg_effectiveness || 0);
        const stars = [1, 2, 3, 4, 5].map((v) =>
          '<span class="cc-star' + (hasScore && v <= rounded ? ' cc-star--filled' : '') + '">&#9733;</span>'
        ).join('');
        tr.innerHTML =
          '<td><span class="cc-entity-pill cc-entity-pill--skill">' + escapeHtml(s.skill_name) + '</span>' +
          '<div class="cc-rating cc-rating--readonly cc-rating--small">' + stars + '</div></td>' +
          '<td>' + s.total_uses + '</td>' +
          '<td>' + s.sessions_used_in + '</td>' +
          '<td>' + score + '</td>' +
          '<td>' + goals + '</td>';
        table.append(tr);
      }
    }
  }
};

export const updateHealthScore = (health) => {
  if (health) {
    const banner = document.querySelector('.cc-health-banner');
    if (banner) {
      const valueEl = banner.querySelector('.cc-health-value');
      if (valueEl) valueEl.textContent = health.score;
      const labelEl = banner.querySelector('.cc-health-label');
      if (labelEl) labelEl.textContent = health.label;
      const scoreEl = banner.querySelector('.cc-health-score');
      if (scoreEl) scoreEl.className = 'cc-health-score ' + health.color_class;
    }
  }
};

export const updateSessionRatings = (ratings) => {
  for (const r of ratings) {
    const ratingEl = document.querySelector('.cc-rating[data-session-id="' + r.session_id + '"]');
    if (ratingEl) {
      ratingEl.setAttribute('data-current', r.rating);
      fillStars(ratingEl, r.rating);
    }
  }
};

export const fillStars = (container, upTo) => {
  for (const s of container.querySelectorAll('.cc-star')) {
    const val = parseInt(s.getAttribute('data-value'), 10);
    s.classList.toggle('cc-star--filled', val <= upTo);
  }
};

export const updateSkillAdoption = (data) => {
  const el = document.getElementById('cc-adoption-summary');
  if (el) {
    const pctEl = el.querySelector('.cc-adoption-pct');
    if (pctEl) pctEl.textContent = (data.adoption_pct || 0) + '%';
    const ring = el.querySelector('.cc-adoption-ring');
    if (ring) ring.style.setProperty('--sp-adoption-pct', data.adoption_pct || 0);
    const label = el.querySelector('.cc-adoption-label');
    if (label) label.textContent = (data.total_used || 0) + '/' + (data.total_available || 0) + ' skills used';
  }
};

export const updateAchievementProgress = (progress) => {
  const el = document.getElementById('cc-achievement-progress');
  if (el && progress?.length) {
    const list = el.querySelector('.cc-achievement-progress-list');
    if (list) {
      list.innerHTML = '';
      for (const p of progress) {
        const item = document.createElement('div');
        item.className = 'cc-achievement-progress-item';
        item.innerHTML =
          '<div class="cc-achievement-progress-info">' +
            '<span class="cc-achievement-progress-name">' + escapeHtml(p.name) + '</span>' +
            '<span class="cc-achievement-progress-count">' + p.current + '/' + p.threshold + '</span>' +
          '</div>' +
          '<div class="cc-bar-track"><div class="cc-bar-fill" style="--sp-fill: ' + p.pct + '%"></div></div>';
        list.append(item);
      }
    }
  }
};
