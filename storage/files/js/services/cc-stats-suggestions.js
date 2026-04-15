import { escapeHtml } from '../utils/dom.js';

const qualityClass = (score) => {
  if (score >= 4) return 'high';
  if (score === 3) return 'medium';
  return 'low';
};

export const injectSuggestionRow = (data) => {
  const panel = document.getElementById('tab-suggestions');
  if (!panel) return;

  const empty = panel.querySelector('.cc-empty-section');
  if (empty) {
    empty.remove();
    const section = document.createElement('div');
    section.className = 'cc-suggestions-section';
    section.innerHTML =
      '<h3 class="cc-suggestions-section-title">Session Analysis</h3>' +
      '<div class="sp-data-table-wrap"><table class="sp-data-table"><thead><tr>' +
      '<th>Session</th><th>Category</th><th>Quality</th><th>Goal</th><th>Tags</th><th>Summary</th><th></th>' +
      '</tr></thead><tbody></tbody></table></div>';
    panel.prepend(section);

    const overlay = document.createElement('div');
    overlay.className = 'cc-detail-overlay';
    overlay.id = 'cc-detail-overlay';
    overlay.hidden = true;
    overlay.innerHTML =
      '<div class="cc-detail-modal" role="dialog" aria-modal="true">' +
      '<button type="button" class="cc-detail-close" aria-label="Close">&times;</button>' +
      '<div class="cc-detail-content" id="cc-detail-content"></div></div>';
    panel.append(overlay);
  }

  const tbody = panel.querySelector('.sp-data-table tbody');
  if (!tbody) return;

  const qc = qualityClass(data.quality_score || 0);
  const tags = (data.tags || []).map((t) => '<span class="cc-tag-badge">' + escapeHtml(t) + '</span>').join('');
  const cat = data.category || 'other';

  const tr = document.createElement('tr');
  tr.className = 'cc-suggestion-row';
  tr.innerHTML =
    '<td><span class="cc-suggestions-session-title">' + escapeHtml(data.title || '') + '</span></td>' +
    '<td><span class="cc-category-badge cc-category-badge--' + escapeHtml(cat) + '">' + escapeHtml(cat) + '</span></td>' +
    '<td><span class="cc-quality-gauge cc-quality-gauge--' + qc + '">' + (data.quality_score || 0) + '/5</span></td>' +
    '<td><span class="cc-goal-badge cc-goal-badge--' + escapeHtml(data.goal_achieved || 'unknown') + '">' + escapeHtml(data.goal_achieved || 'unknown') + '</span></td>' +
    '<td><div class="cc-tag-badges">' + tags + '</div></td>' +
    '<td><span class="cc-suggestion-summary">' + escapeHtml(data.goal_summary || data.description || '') + '</span></td>' +
    '<td><button type="button" class="cc-detail-btn" data-detail="' + escapeHtml(data.session_id) + '">View</button></td>';
  tbody.prepend(tr);

  const detailDiv = document.createElement('div');
  detailDiv.className = 'cc-detail-data';
  detailDiv.id = 'detail-' + data.session_id;
  detailDiv.hidden = true;
  detailDiv.innerHTML = buildDetailHtml(data, qc);
  panel.append(detailDiv);
};

const buildDetailHtml = (d, qc) => {
  const cat = d.category || 'other';
  const tags = (d.tags || []).map((t) => '<span class="cc-tag-badge">' + escapeHtml(t) + '</span>').join('');
  let html =
    '<div class="cc-detail-header">' +
    '<h3 class="cc-detail-title">' + escapeHtml(d.title || '') + '</h3>' +
    '<div class="cc-detail-meta">' +
    '<span class="cc-category-badge cc-category-badge--' + escapeHtml(cat) + '">' + escapeHtml(cat) + '</span>' +
    '<span class="cc-quality-gauge cc-quality-gauge--' + qc + '">' + (d.quality_score || 0) + '/5</span>' +
    '<span class="cc-goal-badge cc-goal-badge--' + escapeHtml(d.goal_achieved || 'unknown') + '">' + escapeHtml(d.goal_achieved || 'unknown') + '</span>' +
    '<div class="cc-tag-badges">' + tags + '</div></div></div>';

  if (d.goal_summary) {
    html += '<div class="cc-detail-section"><h4 class="cc-detail-section-label">Goal Summary</h4>' +
      '<p class="cc-detail-text">' + escapeHtml(d.goal_summary) + '</p></div>';
  }
  if (d.goal_outcome_map && d.goal_outcome_map.length) {
    html += '<div class="cc-detail-section"><h4 class="cc-detail-section-label">Goal vs Outcome</h4><div class="cc-goal-outcome-map">';
    for (const g of d.goal_outcome_map) {
      const cls = g.achieved ? 'achieved' : 'missed';
      html += '<div class="cc-goal-outcome-row cc-goal-outcome-row--' + cls + '">' +
        '<span class="cc-goal-outcome-indicator"></span>' +
        '<div class="cc-goal-outcome-content">' +
        '<span class="cc-goal-text">' + escapeHtml(g.goal) + '</span>' +
        '<span class="cc-outcome-text">' + escapeHtml(g.outcome) + '</span>' +
        '</div></div>';
    }
    html += '</div></div>';
  }
  if (d.outcomes && d.outcomes.length) {
    html += '<div class="cc-detail-section"><h4 class="cc-detail-section-label">Outcomes</h4><ul class="cc-detail-outcomes">';
    for (const o of d.outcomes) html += '<li>' + escapeHtml(o) + '</li>';
    html += '</ul></div>';
  }
  if (d.efficiency_metrics) {
    const e = d.efficiency_metrics;
    html += '<div class="cc-detail-section"><h4 class="cc-detail-section-label">Efficiency</h4>' +
      '<div class="cc-efficiency-grid">' +
      '<div class="cc-eff-metric"><span class="cc-eff-value">' + (e.total_turns || 0) + '</span><span class="cc-eff-label">Turns</span></div>' +
      '<div class="cc-eff-metric"><span class="cc-eff-value">' + (e.duration_minutes || 0) + 'm</span><span class="cc-eff-label">Duration</span></div>' +
      '<div class="cc-eff-metric"><span class="cc-eff-value">' + (e.corrections_count || 0) + '</span><span class="cc-eff-label">Corrections</span></div>' +
      '<div class="cc-eff-metric"><span class="cc-eff-value">' + (e.unnecessary_loops || 0) + '</span><span class="cc-eff-label">Loops</span></div>' +
      '</div></div>';
  }
  if (d.best_practices_checklist && d.best_practices_checklist.length) {
    html += '<div class="cc-detail-section"><h4 class="cc-detail-section-label">Best Practices</h4><ul class="cc-best-practices-list">';
    for (const bp of d.best_practices_checklist) {
      html += '<li class="cc-bp-item cc-bp-item--' + escapeHtml(bp.score || 'n-a') + '">' +
        '<span class="cc-bp-indicator"></span>' +
        '<span class="cc-bp-practice">' + escapeHtml(bp.practice) + '</span>' +
        (bp.note ? '<span class="cc-bp-note">' + escapeHtml(bp.note) + '</span>' : '') +
        '</li>';
    }
    html += '</ul></div>';
  }
  if (d.improvement_hints) {
    html += '<div class="cc-detail-section cc-detail-section--hints">' +
      '<h4 class="cc-detail-section-label">Improvement Hints</h4>' +
      '<p class="cc-detail-text cc-detail-text--hint">' + escapeHtml(d.improvement_hints) + '</p></div>';
  }
  if (d.skill_assessment) {
    html += '<div class="cc-detail-section"><h4 class="cc-detail-section-label">Skill Assessment</h4>' +
      '<p class="cc-detail-text">' + escapeHtml(d.skill_assessment) + '</p></div>';
  }
  if (d.recommendations) {
    html += '<div class="cc-detail-section"><h4 class="cc-detail-section-label">Recommendations</h4>' +
      '<p class="cc-detail-text cc-detail-text--recommendation">' + escapeHtml(d.recommendations) + '</p></div>';
  }
  return html;
};
