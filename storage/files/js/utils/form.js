export const renderChecklist = (id, items, selected, label, opts = {}) => {
  const selectedSet = {};
  if (Array.isArray(selected)) {
    for (const s of selected) {
      const key = typeof s === 'string' ? s : (s.name || s.id || s);
      selectedSet[key] = true;
    }
  } else if (selected && typeof selected === 'object') {
    for (const k of Object.keys(selected)) {
      if (selected[k]) selectedSet[k] = true;
    }
  }

  const hasItems = items?.length > 0;
  const container = document.createElement('div');
  container.className = opts.hasSelectAll ? 'checklist-container checklist-container--tall' : 'checklist-container';
  container.setAttribute('data-checklist', id);

  if (hasItems) {
    for (const item of items) {
      const val = typeof item === 'string' ? item : (item.name || item.id || item);
      const display = typeof item === 'string' ? item : (item.name || item.id || String(item));
      const itemId = id + '-chk-' + val.replace(/[^a-zA-Z0-9_-]/g, '_');

      const div = document.createElement('div');
      div.className = 'checklist-item';
      div.setAttribute('data-item-name', val.toLowerCase());

      const input = document.createElement('input');
      input.type = 'checkbox';
      input.name = id;
      input.value = val;
      input.id = itemId;
      if (selectedSet[val]) input.checked = true;

      const lbl = document.createElement('label');
      lbl.setAttribute('for', itemId);
      lbl.textContent = display;

      div.appendChild(input);
      div.appendChild(lbl);
      container.appendChild(div);
    }
  } else {
    const empty = document.createElement('div');
    empty.className = 'empty-state checklist-empty';
    const p = document.createElement('p');
    p.textContent = 'None available.';
    empty.appendChild(p);
    container.appendChild(empty);
  }

  const filterInput = document.createElement('input');
  filterInput.type = 'text';
  filterInput.className = 'field-input checklist-filter';
  filterInput.placeholder = opts.hasSelectAll ? 'Search...' : 'Filter...';
  filterInput.setAttribute('data-filter-list', id);

  let filterRow;
  if (opts.hasSelectAll) {
    filterRow = document.createElement('div');
    filterRow.className = 'checklist-filter-bar';

    const selectAllBtn = document.createElement('button');
    selectAllBtn.type = 'button';
    selectAllBtn.className = 'btn btn-secondary btn-sm';
    selectAllBtn.setAttribute('data-select-all', id);
    selectAllBtn.textContent = 'Select All';

    const deselectAllBtn = document.createElement('button');
    deselectAllBtn.type = 'button';
    deselectAllBtn.className = 'btn btn-secondary btn-sm';
    deselectAllBtn.setAttribute('data-deselect-all', id);
    deselectAllBtn.textContent = 'Deselect All';

    filterRow.appendChild(filterInput);
    filterRow.appendChild(selectAllBtn);
    filterRow.appendChild(deselectAllBtn);
  } else {
    filterRow = filterInput;
  }

  const group = document.createElement('div');
  group.className = 'form-group';

  const labelEl = document.createElement('label');
  labelEl.className = 'field-label';
  labelEl.textContent = label;

  group.appendChild(labelEl);
  group.appendChild(filterRow);
  group.appendChild(container);

  return group;
};

export const attachFilterHandlers = (root) => {
  root.addEventListener('input', (e) => {
    const filterInput = e.target.closest('[data-filter-list]');
    if (filterInput) {
      const listId = filterInput.getAttribute('data-filter-list');
      const container = root.querySelector('[data-checklist="' + listId + '"]');
      if (container) {
        const q = filterInput.value.toLowerCase();
        for (const item of container.querySelectorAll('.checklist-item')) {
          const name = item.getAttribute('data-item-name') || '';
          item.hidden = q && !name.includes(q);
        }
      }
    }
  });
};

export const getCheckedValues = (form, name) => {
  const checked = form.querySelectorAll('input[name="' + name + '"]:checked');
  return Array.from(checked).map((cb) => cb.value);
};

export const formDataToObject = (formData) => {
  const obj = {};
  for (const [key, value] of formData.entries()) {
    if (key === 'tags') {
      obj[key] = value.split(',').map((t) => t.trim()).filter(Boolean);
    } else {
      obj[key] = value;
    }
  }
  return obj;
};
