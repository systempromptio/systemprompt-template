export const buildPluginBody = (state) => {
  const f = state.form;
  return {
    id: f.plugin_id.trim(),
    name: f.name.trim(),
    description: f.description || '',
    version: f.version || '0.1.0',
    category: f.category || '',
    enabled: true,
    keywords: (f.keywords || '').split(',').map((t) => t.trim()).filter(Boolean),
    author: { name: f.author_name || '' },
    roles: Object.keys(f.roles).filter((k) => f.roles[k]),
    skills: Object.keys(state.selectedSkills).filter((k) => state.selectedSkills[k]),
    agents: Object.keys(state.selectedAgents).filter((k) => state.selectedAgents[k]),
    mcp_servers: Object.keys(state.selectedMcpServers).filter((k) => state.selectedMcpServers[k]),
    hooks: state.hooks.filter((h) => h.command).map((h) => ({
      event: h.event || 'PostToolUse', matcher: h.matcher || '*', command: h.command, async: !!h.async
    }))
  };
};

const collectChecked = (root, name) => {
  const result = {};
  for (const cb of root.querySelectorAll('input[name="' + name + '"]:checked')) result[cb.value] = true;
  return result;
};

export const saveCurrentStepState = (state, root) => {
  if (root) {
    if (state.step === 1) {
      for (const n of ['plugin_id', 'name', 'description', 'version', 'category']) {
        const el = root.querySelector('[name="' + n + '"]');
        if (el) state.form[n] = el.value;
      }
    }
    if (state.step === 2) state.selectedSkills = collectChecked(root, 'wizard-skills');
    if (state.step === 3) state.selectedAgents = collectChecked(root, 'wizard-agents');
    if (state.step === 4) state.selectedMcpServers = collectChecked(root, 'wizard-mcp');
    if (state.step === 5) {
      state.hooks = [];
      for (const entry of root.querySelectorAll('.hook-entry')) {
        state.hooks.push({
          event: (entry.querySelector('[name="hook_event"]') || {}).value || 'PostToolUse',
          matcher: (entry.querySelector('[name="hook_matcher"]') || {}).value || '*',
          command: (entry.querySelector('[name="hook_command"]') || {}).value || '',
          async: !!(entry.querySelector('[name="hook_async"]') || {}).checked
        });
      }
    }
    if (state.step === 6) {
      state.form.roles = collectChecked(root, 'wizard-roles');
      const ai = root.querySelector('[name="author_name"]');
      if (ai) state.form.author_name = ai.value;
      const ki = root.querySelector('[name="keywords"]');
      if (ki) state.form.keywords = ki.value;
    }
  }
};

const restoreCheckboxes = (root, name, selected) => {
  for (const v of Object.keys(selected)) {
    if (selected[v]) {
      const cb = root.querySelector('input[name="' + name + '"][value="' + v + '"]');
      if (cb) cb.checked = true;
    }
  }
};

export const restoreStepState = (state, root) => {
  if (state.step === 1) {
    for (const n of ['plugin_id', 'name', 'description', 'version', 'category']) {
      const el = root.querySelector('[name="' + n + '"]');
      if (el && state.form[n]) el.value = state.form[n];
    }
  }
  if (state.step === 2) restoreCheckboxes(root, 'wizard-skills', state.selectedSkills);
  if (state.step === 3) restoreCheckboxes(root, 'wizard-agents', state.selectedAgents);
  if (state.step === 4) restoreCheckboxes(root, 'wizard-mcp', state.selectedMcpServers);
  if (state.step === 6) {
    restoreCheckboxes(root, 'wizard-roles', state.form.roles);
    const ai = root.querySelector('[name="author_name"]');
    if (ai && state.form.author_name) ai.value = state.form.author_name;
    const ki = root.querySelector('[name="keywords"]');
    if (ki && state.form.keywords) ki.value = state.form.keywords;
  }
};

const buildBadgeList = (items, msg) => {
  const frag = document.createDocumentFragment();
  if (!items.length) {
    const span = document.createElement('span');
    span.className = 'wizard-review-empty';
    span.textContent = msg;
    frag.append(span);
  } else {
    for (const i of items) {
      const badge = document.createElement('span');
      badge.className = 'badge badge-blue wizard-review-badge';
      badge.textContent = i;
      frag.append(badge);
    }
  }
  return frag;
};

const appendLabel = (el, text) => { const s = document.createElement('strong'); s.textContent = text; el.append(s); };

const addField = (el, labelText, value) => { appendLabel(el, labelText); const sp = document.createElement('span'); sp.textContent = value; el.append(sp); };

const addListField = (el, labelText, items, msg, wrap) => {
  appendLabel(el, labelText);
  const d = document.createElement('div');
  if (wrap) d.className = 'wizard-review-badges';
  d.append(buildBadgeList(items, msg));
  el.append(d);
};

export const renderReview = (state) => {
  const el = document.getElementById('wizard-review');
  if (el) {
    const f = state.form;
    const sk = Object.keys(state.selectedSkills).filter((k) => state.selectedSkills[k]);
    const ag = Object.keys(state.selectedAgents).filter((k) => state.selectedAgents[k]);
    const mc = Object.keys(state.selectedMcpServers).filter((k) => state.selectedMcpServers[k]);
    const ro = Object.keys(f.roles).filter((k) => f.roles[k]);
    el.textContent = '';
    addField(el, 'Plugin ID:', f.plugin_id || '-');
    addField(el, 'Name:', f.name || '-');
    addField(el, 'Description:', f.description || '-');
    addField(el, 'Version:', f.version || '0.1.0');
    addField(el, 'Category:', f.category || '-');
    addField(el, 'Author:', f.author_name || '-');
    addField(el, 'Keywords:', f.keywords || '-');
    addListField(el, 'Roles:', ro, 'None selected', false);
    addListField(el, 'Skills (' + sk.length + '):', sk, 'None selected', true);
    addListField(el, 'Agents (' + ag.length + '):', ag, 'None selected', true);
    addListField(el, 'MCP (' + mc.length + '):', mc, 'None selected', true);
    const hookStrong = document.createElement('strong');
    hookStrong.textContent = 'Hooks (' + state.hooks.length + '):';
    el.append(hookStrong);
    const hookSpan = document.createElement('span');
    hookSpan.textContent = state.hooks.length > 0
      ? state.hooks.map((h) => h.event + ': ' + (h.command || '?')).join(', ')
      : 'None';
    el.append(hookSpan);
  }
};
