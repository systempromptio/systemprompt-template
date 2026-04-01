const mk = (tag, props = {}, children = []) => {
  const node = Object.assign(document.createElement(tag), props);
  for (const c of children) node.append(typeof c === 'string' ? document.createTextNode(c) : c);
  return node;
};

export const renderDiffPanel = (userSkill, baseSkill) => {
  const wrapper = mk('div', { className: 'diff-panel', style: 'margin-top:var(--sp-space-4);border:1px solid var(--sp-border-default);border-radius:var(--sp-radius-md);overflow:hidden' });

  const header = mk('div', {
    className: 'diff-panel-header',
    style: 'padding:var(--sp-space-3) var(--sp-space-4);background:var(--sp-bg-surface-raised);border-bottom:1px solid var(--sp-border-default);font-weight:600;font-size:var(--sp-text-sm)'
  }, ['Diff: ' + (userSkill.skill_id || 'custom') + ' vs ' + (baseSkill.skill_id || 'core')]);
  wrapper.append(header);

  const body = mk('div', { style: 'display:grid;grid-template-columns:1fr 1fr;gap:0' });

  const renderSide = (label, skill) => {
    const side = mk('div', { style: 'padding:var(--sp-space-3) var(--sp-space-4);font-size:var(--sp-text-xs);overflow:auto;border-right:1px solid var(--sp-border-default)' });
    side.append(mk('div', { style: 'font-weight:600;margin-bottom:var(--sp-space-2);color:var(--sp-text-secondary)', textContent: label }));
    const fields = ['name', 'description', 'system_prompt', 'instructions'];
    for (const field of fields) {
      const val = skill[field];
      if (val !== undefined && val !== null) {
        side.append(mk('div', { style: 'margin-bottom:var(--sp-space-2)' }, [
          mk('span', { style: 'color:var(--sp-text-tertiary)', textContent: field + ': ' }),
          mk('span', { textContent: typeof val === 'string' ? val.substring(0, 500) : String(val) })
        ]));
      }
    }
    return side;
  };

  body.append(renderSide('Customized', userSkill));
  body.append(renderSide('Core', baseSkill));
  wrapper.append(body);
  return wrapper;
};

export const buildChangelogTable = (changelog) => {
  const table = mk('table', { className: 'data-table', style: 'width:100%' });

  const thead = mk('thead');
  const headerRow = mk('tr');
  for (const col of ['Date', 'Action', 'Description']) {
    headerRow.append(mk('th', { textContent: col }));
  }
  thead.append(headerRow);
  table.append(thead);

  const tbody = mk('tbody');
  for (const entry of changelog) {
    const row = mk('tr');
    const dateCell = mk('td', { style: 'white-space:nowrap' });
    const dateStr = entry.created_at || entry.date || '';
    dateCell.textContent = dateStr ? new Date(dateStr).toLocaleDateString() : '';
    row.append(dateCell);
    row.append(mk('td', { textContent: entry.action || entry.type || '' }));
    row.append(mk('td', { textContent: entry.description || entry.message || '' }));
    tbody.append(row);
  }
  table.append(tbody);
  return table;
};
