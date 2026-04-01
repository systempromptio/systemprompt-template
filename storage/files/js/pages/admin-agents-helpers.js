export const getAgentDetail = (agentId) => {
  const el = document.querySelector('script[data-agent-detail="' + agentId + '"]');
  if (el) {
    try { return JSON.parse(el.textContent); } catch (e) { return null; }
  }
  return null;
};

export const renderAgentExpand = (agentId) => {
  const data = getAgentDetail(agentId);
  const container = document.createElement('div');
  container.className = 'agent-expand-content';

  if (!data) {
    const msg = document.createElement('p');
    msg.className = 'text-muted';
    msg.textContent = 'No detail available for this agent.';
    container.append(msg);
    return container;
  }

  const addField = (label, value) => {
    if (value === undefined || value === null || value === '') return;
    const row = document.createElement('div');
    row.className = 'detail-field sp-skill-detail-field';
    const labelEl = document.createElement('span');
    labelEl.className = 'detail-field-label sp-skill-detail-label';
    labelEl.textContent = label + ':';
    const valueEl = document.createElement('span');
    valueEl.className = 'detail-field-value sp-skill-detail-value';
    valueEl.textContent = typeof value === 'string' ? value : JSON.stringify(value);
    row.append(labelEl);
    row.append(valueEl);
    container.append(row);
  };

  addField('Name', data.name);
  addField('Agent ID', data.id || agentId);
  addField('Description', data.description);
  addField('Enabled', data.enabled !== undefined ? String(data.enabled) : undefined);

  if (data.assigned_plugin_ids && data.assigned_plugin_ids.length) {
    addField('Plugins', data.assigned_plugin_ids.join(', '));
  }

  if (data.system_prompt) {
    const promptSection = document.createElement('div');
    promptSection.className = 'sp-skill-prompt-section';
    const promptLabel = document.createElement('div');
    promptLabel.className = 'sp-skill-prompt-label';
    promptLabel.textContent = 'System Prompt:';
    const promptPre = document.createElement('pre');
    promptPre.className = 'code-block sp-skill-prompt-pre';
    promptPre.textContent = data.system_prompt;
    promptSection.append(promptLabel);
    promptSection.append(promptPre);
    container.append(promptSection);
  }

  return container;
};
