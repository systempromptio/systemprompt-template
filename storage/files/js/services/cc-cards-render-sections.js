export const buildConversationSection = (group) => {
  const section = document.createElement('details');
  section.className = 'cc-card-section';
  const title = document.createElement('summary');
  title.className = 'cc-card-section-title';
  title.textContent = 'Conversation (' + (group.turn_count || 0) + ' turns)';
  section.append(title);
  const body = document.createElement('div');
  body.className = 'cc-card-section-body';
  if (group.first_prompt) {
    const prompt = document.createElement('div');
    prompt.className = 'sf-prompt';
    const speaker = document.createElement('span');
    speaker.className = 'sf-speaker sf-speaker--you';
    speaker.textContent = 'You';
    const text = document.createElement('span');
    text.className = 'sf-prompt-text';
    text.textContent = group.first_prompt;
    prompt.append(speaker, text);
    body.append(prompt);
  }
  if (group.last_response) {
    const resp = document.createElement('div');
    resp.className = 'sf-response';
    const speaker = document.createElement('span');
    speaker.className = 'sf-speaker sf-speaker--ai';
    speaker.textContent = 'AI';
    const text = document.createElement('span');
    text.className = 'sf-response-text';
    text.textContent = group.last_response;
    resp.append(speaker, text);
    body.append(resp);
  }
  if (group.has_turns && group.turns && group.turn_count > 1) {
    const expand = document.createElement('details');
    expand.className = 'cc-turns-expand';
    const expandSummary = document.createElement('summary');
    expandSummary.textContent = 'View all ' + group.turn_count + ' turns';
    expand.append(expandSummary);
    const turnsDiv = document.createElement('div');
    turnsDiv.className = 'sf-turns';
    const { buildTurnElement } = await_import_turns();
    for (const turn of group.turns) turnsDiv.append(buildTurnElement(turn));
    expand.append(turnsDiv);
    body.append(expand);
  }
  section.append(body);
  return section;
};

let _buildTurnElement = null;
function await_import_turns() {
  if (!_buildTurnElement) {
    _buildTurnElement = (turn) => {
      const div = document.createElement('div');
      div.className = 'sf-turn';
      if (turn.has_prompt && turn.prompt_text) {
        const prompt = document.createElement('div');
        prompt.className = 'sf-prompt';
        const speaker = document.createElement('span');
        speaker.className = 'sf-speaker sf-speaker--you';
        speaker.textContent = 'You';
        const text = document.createElement('span');
        text.className = 'sf-prompt-text';
        text.textContent = turn.prompt_text;
        prompt.append(speaker, text);
        div.append(prompt);
      }
      if (turn.has_tools && turn.total_tools > 0) {
        const details = document.createElement('details');
        details.className = 'sf-tools-summary';
        const summary = document.createElement('summary');
        summary.className = 'sf-tools-toggle';
        summary.textContent = turn.total_tools + ' tools used';
        details.append(summary);
        div.append(details);
      }
      if (turn.errors) {
        for (const err of turn.errors) {
          const errEl = document.createElement('div');
          errEl.className = 'sf-error';
          errEl.textContent = (err.tool_name || 'Error') + ': ' + (err.description || '');
          div.append(errEl);
        }
      }
      if (turn.has_response && turn.response_text) {
        const resp = document.createElement('div');
        resp.className = 'sf-response';
        const speaker = document.createElement('span');
        speaker.className = 'sf-speaker sf-speaker--ai';
        speaker.textContent = 'AI';
        const text = document.createElement('span');
        text.className = 'sf-response-text';
        text.textContent = turn.response_text;
        resp.append(speaker, text);
        div.append(resp);
      }
      return div;
    };
  }
  return { buildTurnElement: _buildTurnElement };
}
export const buildEntitiesSection = (group) => {
  const section = document.createElement('details');
  section.className = 'cc-card-section';
  const title = document.createElement('summary');
  title.className = 'cc-card-section-title';
  title.textContent = 'Skills & Agents (' + (group.entities?.length || 0) + ')';
  section.append(title);
  const body = document.createElement('div');
  body.className = 'cc-card-section-body';
  const entities = group.entities || [];
  for (const entity of entities) {
    const pill = document.createElement('span');
    pill.className = 'cc-entity-pill cc-entity-pill--' + entity.entity_type;
    pill.textContent = entity.entity_name + ' (' + entity.usage_count + 'x)';
    body.append(pill);
  }
  section.append(body);
  return section;
};

export const buildErrorsSection = (group) => {
  const section = document.createElement('details');
  section.className = 'cc-card-section';
  const title = document.createElement('summary');
  title.className = 'cc-card-section-title cc-card-section-title--error';
  title.textContent = 'Errors (' + (group.total_errors || 0) + ')';
  section.append(title);
  const body = document.createElement('div');
  body.className = 'cc-card-section-body';
  const errors = group.all_errors || [];
  for (const err of errors) {
    const errEl = document.createElement('div');
    errEl.className = 'sf-error';
    const icon = document.createElement('span');
    icon.className = 'sf-error-icon';
    icon.textContent = '\u2715';
    const tool = document.createElement('span');
    tool.className = 'sf-error-tool';
    tool.textContent = (err.tool_name || 'Error') + ':';
    const desc = document.createElement('span');
    desc.className = 'sf-error-desc';
    desc.textContent = err.description || '';
    errEl.append(icon, tool, desc);
    body.append(errEl);
  }
  section.append(body);
  return section;
};
