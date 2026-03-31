export const populateToolsToggle = (container, turn) => {
  container.textContent = '';
  const arrow = document.createElement('span');
  arrow.className = 'sf-tools-arrow';
  arrow.textContent = '\u25B8';
  container.append(arrow);
  container.append(document.createTextNode(' ' + turn.total_tools + ' tools: '));
  if (turn.tool_groups) {
    for (const tg of turn.tool_groups) {
      const chip = document.createElement('span');
      chip.className = 'sf-tool-chip';
      chip.textContent = tg.name + '\u00D7' + tg.count;
      container.append(chip);
    }
  }
};

const buildErrorElement = (err) => {
  const errEl = document.createElement('div');
  errEl.className = 'sf-error';
  const icon = document.createElement('span');
  icon.className = 'sf-error-icon';
  icon.textContent = '\u2715';
  const tool = document.createElement('span');
  tool.className = 'sf-error-tool';
  tool.textContent = err.tool_name + ':';
  const desc = document.createElement('span');
  desc.className = 'sf-error-desc';
  desc.textContent = err.description;
  errEl.append(icon, tool, desc);
  return errEl;
};
const buildResponseElement = (turn) => {
  const resp = document.createElement('div');
  resp.className = 'sf-response';
  const speaker = document.createElement('span');
  speaker.className = 'sf-speaker sf-speaker--ai';
  speaker.textContent = 'AI';
  const text = document.createElement('span');
  text.className = 'sf-response-text';
  text.textContent = turn.response_text;
  resp.append(speaker, text);
  return resp;
};
export const buildTurnElement = (turn) => {
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
    populateToolsToggle(summary, turn);
    details.append(summary);
    div.append(details);
  }
  if (turn.errors) {
    for (const err of turn.errors) {
      div.append(buildErrorElement(err));
    }
  }
  if (turn.has_response && turn.response_text) {
    div.append(buildResponseElement(turn));
  }
  return div;
};

export const updateTurnElement = (el, turn) => {
  let responseEl = el.querySelector('.sf-response');
  if (turn.has_response && turn.response_text) {
    if (!responseEl) {
      el.append(buildResponseElement(turn));
    } else {
      const rText = responseEl.querySelector('.sf-response-text');
      if (rText) rText.textContent = turn.response_text;
    }
  }
  if (turn.has_tools) {
    let toolsDetail = el.querySelector('.sf-tools-summary');
    if (toolsDetail) {
      const toggle = toolsDetail.querySelector('.sf-tools-toggle');
      if (toggle) populateToolsToggle(toggle, turn);
    } else {
      toolsDetail = document.createElement('details');
      toolsDetail.className = 'sf-tools-summary';
      const summary = document.createElement('summary');
      summary.className = 'sf-tools-toggle';
      populateToolsToggle(summary, turn);
      toolsDetail.append(summary);
      const ins = el.querySelector('.sf-error') || el.querySelector('.sf-response');
      if (ins) el.insertBefore(toolsDetail, ins);
      else {
        const prompt = el.querySelector('.sf-prompt');
        if (prompt) prompt.after(toolsDetail);
        else el.append(toolsDetail);
      }
    }
  }
  const existingErrors = el.querySelectorAll('.sf-error');
  const turnErrors = turn.errors || [];
  for (let i = existingErrors.length; i < turnErrors.length; i++) {
    const resp = el.querySelector('.sf-response');
    const errEl = buildErrorElement(turnErrors[i]);
    if (resp) el.insertBefore(errEl, resp);
    else el.append(errEl);
  }
};

export const reconcileTurns = (card, turns) => {
  let tc = card.querySelector('.sf-turns');
  if (!tc) {
    tc = document.createElement('div');
    tc.className = 'sf-turns';
    const sectionBody = card.querySelector('.cc-card-section-body');
    if (sectionBody) {
      const expand = document.createElement('details');
      expand.className = 'cc-turns-expand';
      const expandSummary = document.createElement('summary');
      expandSummary.textContent = 'View all ' + turns.length + ' turns';
      expand.append(expandSummary);
      expand.append(tc);
      sectionBody.append(expand);
    } else {
      card.append(tc);
    }
  }
  const existing = tc.querySelectorAll('.sf-turn');
  if (existing.length > 0 && turns.length >= existing.length) {
    updateTurnElement(existing[existing.length - 1], turns[existing.length - 1]);
  }
  for (let i = existing.length; i < turns.length; i++) {
    const turnEl = buildTurnElement(turns[i]);
    turnEl.classList.add('sf-turn--entering');
    tc.append(turnEl);
    void turnEl.offsetHeight;
    requestAnimationFrame(() => turnEl.classList.remove('sf-turn--entering'));
  }
};
