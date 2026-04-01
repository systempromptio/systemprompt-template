const collectAllPlugins = () => {
  const allPlugins = [];
  for (const script of document.querySelectorAll('script[data-mcp-detail]')) {
    try {
      const data = JSON.parse(script.textContent);
      if (data.assigned_plugins) {
        for (const p of data.assigned_plugins) {
          if (!allPlugins.some((e) => e.id === p.id)) allPlugins.push(p);
        }
      }
    } catch (_e) {
      continue;
    }
  }
  return allPlugins;
};

const buildChecklist = (allPlugins, currentSet) => {
  const checklist = document.createElement('div');
  checklist.className = 'assign-panel-checklist';
  if (!allPlugins.length) {
    const p = document.createElement('p');
    p.className = 'checklist-empty';
    p.textContent = 'No plugins available.';
    checklist.append(p);
  } else {
    for (const pl of allPlugins) {
      const label = document.createElement('label');
      label.className = 'acl-checkbox-row';
      const input = document.createElement('input');
      input.type = 'checkbox';
      input.name = 'plugin_id';
      input.value = pl.id;
      if (currentSet[pl.id]) input.checked = true;
      const span = document.createElement('span');
      span.className = 'acl-checkbox-label';
      span.textContent = pl.name || pl.id;
      label.append(input);
      label.append(span);
      checklist.append(label);
    }
  }
  return checklist;
};

const buildPanelFooter = (footer, mcpId, panel, overlay) => {
  footer.textContent = '';
  const cancelBtn = document.createElement('button');
  cancelBtn.className = 'btn btn-secondary';
  cancelBtn.setAttribute('data-panel-close', '');
  cancelBtn.textContent = 'Cancel';
  const saveBtn = document.createElement('button');
  saveBtn.className = 'btn btn-primary';
  saveBtn.setAttribute('data-assign-save', '');
  saveBtn.setAttribute('data-entity-id', mcpId);
  saveBtn.textContent = 'Save';
  footer.append(cancelBtn);
  footer.append(document.createTextNode(' '));
  footer.append(saveBtn);
  cancelBtn.addEventListener('click', () => {
    panel.classList.remove('open');
    overlay?.classList.remove('active');
  });
};

export const openAssignPanel = (mcpId, mcpName, currentPluginIds) => {
  const allPlugins = collectAllPlugins();
  const panel = document.getElementById('assign-panel');
  if (panel) {
    const overlay = document.getElementById(panel.getAttribute('data-overlay') || 'assign-panel-overlay');
    const titleEl = panel.querySelector('[data-panel-title]');
    if (titleEl) titleEl.textContent = 'Assign ' + (mcpName || mcpId);
    const currentSet = {};
    for (const id of currentPluginIds) { currentSet[id] = true; }
    const body = panel.querySelector('[data-panel-body]');
    if (body) {
      body.textContent = '';
      body.append(buildChecklist(allPlugins, currentSet));
    }
    const footer = panel.querySelector('[data-panel-footer]');
    if (footer) buildPanelFooter(footer, mcpId, panel, overlay);
    panel.classList.add('open');
    overlay?.classList.add('active');
  }
};
