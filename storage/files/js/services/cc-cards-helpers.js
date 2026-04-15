import { reconcileTurns } from './control-center-turns.js';
import { buildMetricChips } from './cc-cards-render-metrics.js';
import { buildEntities } from './cc-cards-render-session.js';
import { relativeTime } from './cc-stats-header.js';

const getTitle = (g) => g.ai_title || g.session_title || g.project_name || 'Untitled';
const shouldShowSummary = (group) => {
  if (!group.ai_summary) return false;
  if (!group.ai_title) return true;
  return !group.ai_summary.startsWith(group.ai_title);
};
export const updateSessionCard = (card, group) => {
  card.setAttribute('data-is-active', group.is_active ? 'true' : 'false');
  card.setAttribute('data-status', group.status || 'active');
  card.setAttribute('data-is-analysed', group.is_analysed ? 'true' : 'false');

  const titleEl = card.querySelector('.sf-session-title');
  if (titleEl) {
    const newTitle = getTitle(group);
    if (titleEl.textContent !== newTitle) titleEl.textContent = newTitle;
  }
  const oldChips = card.querySelector('.sf-metric-chips');
  if (oldChips) oldChips.replaceWith(buildMetricChips(group));
  let analysisPreview = card.querySelector('.cc-analysis-preview');
  let aiSummary = card.querySelector('.cc-ai-summary');
  if (group.is_analysed) {
    if (aiSummary) aiSummary.remove();
    if (!analysisPreview) {
      analysisPreview = document.createElement('div');
      analysisPreview.className = 'cc-analysis-preview';
      const actions = card.querySelector('.sf-session-actions');
      if (actions?.nextSibling) actions.parentNode.insertBefore(analysisPreview, actions.nextSibling);
      else card.append(analysisPreview);
    }
    analysisPreview.textContent = '';
    if (group.description) {
      const desc = document.createElement('span');
      desc.className = 'cc-analysis-desc';
      desc.textContent = group.description;
      analysisPreview.append(desc);
    }
    if (group.has_recommendations && group.recommendations) {
      const rec = document.createElement('span');
      rec.className = 'cc-analysis-rec';
      rec.textContent = group.recommendations;
      analysisPreview.append(rec);
    }
  } else {
    if (analysisPreview) analysisPreview.remove();
    if (shouldShowSummary(group)) {
      if (aiSummary) {
        aiSummary.textContent = group.ai_summary;
      } else {
        aiSummary = document.createElement('div');
        aiSummary.className = 'cc-ai-summary';
        aiSummary.textContent = group.ai_summary;
        const actions = card.querySelector('.sf-session-actions');
        if (actions?.nextSibling) actions.parentNode.insertBefore(aiSummary, actions.nextSibling);
        else card.append(aiSummary);
      }
    } else if (aiSummary) {
      aiSummary.remove();
    }
  }
  const oldEntities = card.querySelector('.cc-card-entities-inline');
  if (group.entities?.length > 0) {
    const newEntities = buildEntities(group);
    if (oldEntities) oldEntities.replaceWith(newEntities);
    else {
      const summary = card.querySelector('.cc-ai-summary');
      if (summary?.nextSibling) summary.parentNode.insertBefore(newEntities, summary.nextSibling);
    }
  } else if (oldEntities) {
    oldEntities.remove();
  }
  const timeEl = card.querySelector('.sf-session-time');
  if (timeEl && group.last_activity_at) timeEl.textContent = relativeTime(group.last_activity_at);
  let activeLabel = card.querySelector('.sf-session-active-label');
  if (group.is_active && !activeLabel) {
    activeLabel = document.createElement('span');
    activeLabel.className = 'sf-session-active-label';
    activeLabel.textContent = 'active';
    card.querySelector('.sf-session-right')?.append(activeLabel);
  } else if (!group.is_active && activeLabel) {
    activeLabel.remove();
  }
  let qualityBadge = card.querySelector('.sf-quality-badge');
  if (group.has_quality_score) {
    if (qualityBadge) {
      qualityBadge.className = 'sf-quality-badge sf-quality-badge--' + (group.quality_class || 'medium');
      qualityBadge.textContent = 'Q' + group.quality_score;
    } else {
      qualityBadge = document.createElement('span');
      qualityBadge.className = 'sf-quality-badge sf-quality-badge--' + (group.quality_class || 'medium');
      qualityBadge.title = 'Quality: ' + group.quality_score + '/5';
      qualityBadge.textContent = 'Q' + group.quality_score;
      card.querySelector('.sf-session-right')?.prepend(qualityBadge);
    }
  } else if (qualityBadge) {
    qualityBadge.remove();
  }
  let goalBadge = card.querySelector('.sf-goal-badge');
  if (group.goal_achieved) {
    if (goalBadge) {
      goalBadge.className = 'sf-goal-badge sf-goal-badge--' + group.goal_achieved;
      goalBadge.textContent = group.goal_icon || '';
    } else {
      goalBadge = document.createElement('span');
      goalBadge.className = 'sf-goal-badge sf-goal-badge--' + group.goal_achieved;
      goalBadge.title = 'Goal: ' + group.goal_achieved;
      goalBadge.textContent = group.goal_icon || '';
      const rightEl = card.querySelector('.sf-session-right');
      const qBadge = rightEl?.querySelector('.sf-quality-badge');
      if (qBadge?.nextSibling) rightEl.insertBefore(goalBadge, qBadge.nextSibling);
      else rightEl?.prepend(goalBadge);
    }
  } else if (goalBadge) {
    goalBadge.remove();
  }
  let analysedLabel = card.querySelector('.sf-session-status-label--analysed');
  if (group.is_analysed && !analysedLabel) {
    analysedLabel = document.createElement('span');
    analysedLabel.className = 'sf-session-status-label sf-session-status-label--analysed';
    analysedLabel.textContent = 'analysed';
    card.querySelector('.sf-session-right')?.append(analysedLabel);
  } else if (!group.is_analysed && analysedLabel) {
    analysedLabel.remove();
  }
  const st = group.status || 'active';
  let statusLabel = card.querySelector('.sf-session-status-label--completed');
  if (st === 'completed') {
    if (statusLabel) {
      if (statusLabel.textContent !== st) {
        statusLabel.className = 'sf-session-status-label sf-session-status-label--' + st;
        statusLabel.textContent = st;
      }
    } else {
      statusLabel = document.createElement('span');
      statusLabel.className = 'sf-session-status-label sf-session-status-label--' + st;
      statusLabel.textContent = st;
      card.querySelector('.sf-session-right')?.append(statusLabel);
    }
  } else if (statusLabel) {
    statusLabel.remove();
  }
  if (group.is_analysed) {
    const analyseBtn = card.querySelector('.sf-action-item[data-action="analyse"]');
    if (analyseBtn) analyseBtn.remove();
  }
  const convTitle = card.querySelector('.cc-card-section-title:not(.cc-card-section-title--error)');
  if (convTitle && group.turn_count != null) {
    convTitle.textContent = 'Conversation (' + group.turn_count + ' turns)';
  }
  if (group.has_turns && group.turns) reconcileTurns(card, group.turns);
};
