export const buildSessionActions = (sessionId, isAnalysed) => {
  const actions = document.createElement('span');
  actions.className = 'sf-session-actions';
  actions.setAttribute('data-session-id', sessionId);
  const actBtn = document.createElement('button');
  actBtn.className = 'sf-action-btn';
  actBtn.title = 'Actions';
  actBtn.textContent = '\u22EE';
  actions.append(actBtn);
  const menu = document.createElement('div');
  menu.className = 'sf-action-menu';
  const items = [];
  if (!isAnalysed) items.push(['analyse', 'Analyse', false, true]);
  items.push(['completed', 'Mark Complete'], ['deleted', 'Delete', true], ['active', 'Restore']);
  for (const [action, text, danger, accent] of items) {
    const btn = document.createElement('button');
    btn.className = 'sf-action-item' + (danger ? ' sf-action-item--danger' : '') + (accent ? ' sf-action-item--accent' : '');
    btn.setAttribute('data-action', action);
    btn.textContent = text;
    menu.append(btn);
  }
  actions.append(menu);
  return actions;
};

export const buildEntities = (group) => {
  const wrapper = document.createElement('div');
  wrapper.className = 'cc-card-entities-inline';
  const entities = group.entities || [];
  for (const entity of entities) {
    const pill = document.createElement('span');
    pill.className = 'cc-entity-pill cc-entity-pill--' + entity.entity_type;
    pill.title = entity.entity_name + ' (' + entity.usage_count + 'x)';
    pill.textContent = entity.entity_name;
    wrapper.append(pill);
  }
  return wrapper;
};

export const buildSummaryStrip = (group) => {
  const strip = document.createElement('div');
  strip.className = 'cc-session-summary';
  const parts = [];
  parts.push((group.turn_count || 0) + ' turns');
  parts.push((group.total_tools || 0) + ' tools');
  if (group.has_errors && group.total_errors > 0) {
    parts.push(group.total_errors + ' errors');
  }
  for (let i = 0; i < parts.length; i++) {
    if (i > 0) {
      const sep = document.createElement('span');
      sep.className = 'cc-summary-sep';
      sep.textContent = '\u00B7';
      strip.append(sep);
    }
    const span = document.createElement('span');
    span.className = 'cc-summary-stat' + (i === parts.length - 1 && group.has_errors && group.total_errors > 0 ? ' cc-summary-stat--error' : '');
    span.textContent = parts[i];
    strip.append(span);
  }
  return strip;
};
