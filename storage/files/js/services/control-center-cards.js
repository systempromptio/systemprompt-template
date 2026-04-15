import { populateSessionSummary } from './cc-cards-render-metrics.js';
import { buildEntities, buildSessionActions } from './cc-cards-render-session.js';
import {
  buildConversationSection, buildErrorsSection,
} from './cc-cards-render-sections.js';

export { updateSessionCard } from './cc-cards-helpers.js';

const shouldShowSummary = (group) => {
  if (!group.ai_summary) return false;
  if (!group.ai_title) return true;
  return !group.ai_summary.startsWith(group.ai_title);
};

export const buildSessionCard = (group, isFirst) => {
  const details = document.createElement('details');
  details.className = 'sf-session';
  details.setAttribute('data-session-id', group.session_id);
  details.setAttribute('data-is-active', group.is_active ? 'true' : 'false');
  details.setAttribute('data-status', group.status || 'active');
  details.setAttribute('data-is-analysed', group.is_analysed ? 'true' : 'false');
  if (isFirst) details.open = true;

  const checkWrap = document.createElement('span');
  checkWrap.className = 'cc-select-check';
  checkWrap.onclick = (e) => e.stopPropagation();
  const checkInput = document.createElement('input');
  checkInput.type = 'checkbox';
  checkInput.className = 'cc-session-checkbox';
  checkInput.setAttribute('data-session-id', group.session_id);
  checkInput.onclick = (e) => e.stopPropagation();
  checkWrap.append(checkInput);
  details.append(checkWrap);

  const summary = document.createElement('summary');
  summary.className = 'sf-session-header';
  populateSessionSummary(summary, group);
  details.append(summary);

  details.append(buildSessionActions(group.session_id, group.is_analysed));

  if (group.is_analysed) {
    const preview = document.createElement('div');
    preview.className = 'cc-analysis-preview';
    if (group.description) {
      const desc = document.createElement('span');
      desc.className = 'cc-analysis-desc';
      desc.textContent = group.description;
      preview.append(desc);
    }
    if (group.has_recommendations && group.recommendations) {
      const rec = document.createElement('span');
      rec.className = 'cc-analysis-rec';
      rec.textContent = group.recommendations;
      preview.append(rec);
    }
    details.append(preview);
  } else if (shouldShowSummary(group)) {
    const aiSummary = document.createElement('div');
    aiSummary.className = 'cc-ai-summary';
    aiSummary.textContent = group.ai_summary;
    details.append(aiSummary);
  }
  if (group.entities?.length > 0) details.append(buildEntities(group));
  details.append(buildConversationSection(group));
  if (group.has_errors && group.total_errors > 0) details.append(buildErrorsSection(group));

  return details;
};
