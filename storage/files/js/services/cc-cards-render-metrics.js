import { relativeTime } from './cc-stats-header.js';

export const formatContentBytes = (bytes) => {
  if (!bytes || bytes <= 0) return '0';
  if (bytes < 1000) return bytes + '';
  if (bytes < 1000000) return (bytes / 1000).toFixed(1).replace(/\.0$/, '') + 'k';
  return (bytes / 1000000).toFixed(1).replace(/\.0$/, '') + 'M';
};

const getTitle = (group) =>
  group.ai_title || group.session_title || group.project_name || 'Untitled';

const addSep = (strip) => {
  const sep = document.createElement('span');
  sep.className = 'sf-metric-sep';
  sep.textContent = '\u00B7';
  strip.append(sep);
};

export const buildMetricChips = (group) => {
  const strip = document.createElement('div');
  strip.className = 'sf-metric-chips';
  let hasItem = false;

  if (group.has_client_source || group.client_source_label) {
    const chip = document.createElement('span');
    chip.className = 'sf-metric-chip sf-chip--source sf-chip--source-' + (group.client_source_class || 'other');
    chip.title = 'Client: ' + (group.client_source || '');
    chip.textContent = group.client_source_label || group.client_source || '';
    strip.append(chip);
    hasItem = true;
  }

  if (group.is_plan_mode) {
    if (hasItem) addSep(strip);
    const chip = document.createElement('span');
    chip.className = 'sf-metric-chip sf-chip--mode';
    chip.title = 'Plan mode';
    chip.textContent = 'plan';
    strip.append(chip);
    hasItem = true;
  }

  if (group.has_model || group.model_short) {
    if (hasItem) addSep(strip);
    const chip = document.createElement('span');
    chip.className = 'sf-metric-chip sf-chip--model';
    chip.title = 'Model: ' + (group.model || '');
    chip.textContent = group.model_short || '';
    strip.append(chip);
    hasItem = true;
  }

  if (group.has_automated) {
    if (hasItem) addSep(strip);
    const chip = document.createElement('span');
    chip.className = 'sf-metric-chip';
    chip.title = (group.user_prompts || 0) + ' user prompts, ' + (group.automated_actions || 0) + ' automated actions';
    chip.textContent = (group.user_prompts || 0) + ' user / ' + (group.automated_actions || 0) + ' auto';
    strip.append(chip);
    hasItem = true;
  } else if (group.total_prompts) {
    if (hasItem) addSep(strip);
    const chip = document.createElement('span');
    chip.className = 'sf-metric-chip';
    chip.textContent = group.total_prompts + ' prompts';
    strip.append(chip);
    hasItem = true;
  }

  const skillCount = (group.entities || []).filter(
    (e) => e.entity_type === 'skill' || e.entity_type === 'agent',
  ).length;
  if (skillCount) {
    if (hasItem) addSep(strip);
    const chip = document.createElement('span');
    chip.className = 'sf-metric-chip';
    chip.textContent = skillCount + ' skills';
    strip.append(chip);
    hasItem = true;
  }

  if (group.total_tools) {
    if (hasItem) addSep(strip);
    const chip = document.createElement('span');
    chip.className = 'sf-metric-chip';
    chip.textContent = group.total_tools + ' tools';
    strip.append(chip);
    hasItem = true;
  }

  if (group.duration_display) {
    if (hasItem) addSep(strip);
    const chip = document.createElement('span');
    chip.className = 'sf-metric-chip';
    chip.textContent = group.duration_display;
    strip.append(chip);
  }

  return strip;
};

export const updateBadge = (card, type, count, className, text) => {
  const badges = card.querySelector('.sf-session-badges');
  if (badges) {
    let badge = badges.querySelector('[data-badge="' + type + '"]');
    if (count > 0) {
      if (badge) {
        if (badge.textContent !== text) {
          badge.textContent = text;
          badge.classList.add('sf-badge--updated');
          setTimeout(() => badge.classList.remove('sf-badge--updated'), 350);
        }
      } else {
        badge = document.createElement('span');
        badge.className = className;
        badge.setAttribute('data-badge', type);
        badge.textContent = text;
        badges.append(badge);
      }
    } else if (badge) {
      badge.remove();
    }
  }
};

export const populateSessionSummary = (container, group) => {
  container.textContent = '';

  const row1 = document.createElement('div');
  row1.className = 'sf-session-header-row1';

  const title = document.createElement('span');
  title.className = 'sf-session-title';
  title.textContent = getTitle(group);
  row1.append(title);

  const right = document.createElement('span');
  right.className = 'sf-session-right';
  const time = document.createElement('span');
  time.className = 'sf-session-time';
  time.textContent = relativeTime(group.last_activity_at || group.started_at);
  right.append(time);
  if (group.is_active) {
    const active = document.createElement('span');
    active.className = 'sf-session-active-label';
    active.textContent = 'active';
    right.append(active);
  }
  if (group.has_quality_score) {
    const qBadge = document.createElement('span');
    qBadge.className = 'sf-quality-badge sf-quality-badge--' + (group.quality_class || 'medium');
    qBadge.title = 'Quality: ' + group.quality_score + '/5';
    qBadge.textContent = 'Q' + group.quality_score;
    right.append(qBadge);
  }
  if (group.goal_achieved) {
    const gBadge = document.createElement('span');
    gBadge.className = 'sf-goal-badge sf-goal-badge--' + group.goal_achieved;
    gBadge.title = 'Goal: ' + group.goal_achieved;
    gBadge.textContent = group.goal_icon || '';
    right.append(gBadge);
  }
  if (group.is_analysed) {
    const analysed = document.createElement('span');
    analysed.className = 'sf-session-status-label sf-session-status-label--analysed';
    analysed.textContent = 'analysed';
    right.append(analysed);
  }
  const st = group.status || 'active';
  if (st === 'completed') {
    const label = document.createElement('span');
    label.className = 'sf-session-status-label sf-session-status-label--' + st;
    label.textContent = st;
    right.append(label);
  }
  row1.append(right);
  container.append(row1);

  const row2 = document.createElement('div');
  row2.className = 'sf-session-header-row2';
  const chips = buildMetricChips(group);
  row2.append(chips);
  const project = document.createElement('span');
  project.className = 'sf-session-project';
  project.textContent = group.project_name;
  row2.append(project);
  container.append(row2);
};
