(function(app) {
    'use strict';

    function renderChecklist(id, items, selected, label, opts) {
        const options = opts || {};
        const selectedSet = {};
        if (Array.isArray(selected)) {
            (selected || []).forEach((s) => {
                const key = typeof s === 'string' ? s : (s.name || s.id || s);
                selectedSet[key] = true;
            });
        } else if (selected && typeof selected === 'object') {
            Object.keys(selected).forEach((k) => {
                if (selected[k]) selectedSet[k] = true;
            });
        }

        const group = document.createElement('div');
        group.className = 'form-group';

        const labelEl = document.createElement('label');
        labelEl.className = 'field-label';
        labelEl.textContent = label;
        group.append(labelEl);

        const filterInput = document.createElement('input');
        filterInput.type = 'text';
        filterInput.className = 'field-input';
        filterInput.setAttribute('data-filter-list', id);

        if (options.hasSelectAll) {
            filterInput.placeholder = 'Search...';
            filterInput.style.flex = '1';
            const filterRow = document.createElement('div');
            filterRow.style.cssText = 'display:flex;gap:var(--sp-space-2);margin-bottom:var(--sp-space-2)';
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
            filterRow.append(filterInput, selectAllBtn, deselectAllBtn);
            group.append(filterRow);
        } else {
            filterInput.placeholder = 'Filter...';
            filterInput.style.marginBottom = 'var(--sp-space-2)';
            group.append(filterInput);
        }

        const maxHeight = options.hasSelectAll ? '300px' : '200px';
        const container = document.createElement('div');
        container.className = 'checklist-container';
        container.setAttribute('data-checklist', id);
        container.style.cssText = 'max-height:' + maxHeight + ';overflow-y:auto;border:1px solid var(--sp-border-subtle);border-radius:var(--sp-radius-md);padding:var(--sp-space-2)';

        const hasItems = items && items.length > 0;
        if (hasItems) {
            items.forEach(function(item) {
                const val = typeof item === 'string' ? item : (item.name || item.id || item);
                const displayName = typeof item === 'string' ? item : (item.name || item.id || String(item));
                const itemId = id + '-chk-' + val.replace(/[^a-zA-Z0-9_-]/g, '_');
                const itemDiv = document.createElement('div');
                itemDiv.className = 'checklist-item';
                itemDiv.setAttribute('data-item-name', val.toLowerCase());
                const checkbox = document.createElement('input');
                checkbox.type = 'checkbox';
                checkbox.name = id;
                checkbox.value = val;
                checkbox.id = itemId;
                if (selectedSet[val]) checkbox.checked = true;
                const itemLabel = document.createElement('label');
                itemLabel.setAttribute('for', itemId);
                itemLabel.textContent = displayName;
                itemDiv.append(checkbox, itemLabel);
                container.append(itemDiv);
            });
        } else {
            const emptyDiv = document.createElement('div');
            emptyDiv.className = 'empty-state';
            emptyDiv.style.padding = 'var(--sp-space-4)';
            const emptyP = document.createElement('p');
            emptyP.textContent = 'None available.';
            emptyDiv.append(emptyP);
            container.append(emptyDiv);
        }

        group.append(container);
        return group;
    }

    function attachFilterHandlers(root) {
        root.addEventListener('input', (e) => {
            const filterInput = e.target.closest('[data-filter-list]');
            if (!filterInput) return;
            const listId = filterInput.getAttribute('data-filter-list');
            const container = root.querySelector('[data-checklist="' + listId + '"]');
            if (!container) return;
            const q = filterInput.value.toLowerCase();
            const items = container.querySelectorAll('.checklist-item');
            items.forEach((item) => {
                const name = item.getAttribute('data-item-name') || '';
                item.style.display = (q && !name.includes(q)) ? 'none' : '';
            });
        });
    }

    function getCheckedValues(form, name) {
        const checked = form.querySelectorAll('input[name="' + name + '"]:checked');
        return Array.from(checked).map((cb) => cb.value);
    }
    app.formUtils = {
        renderChecklist,
        attachFilterHandlers,
        getCheckedValues
    };
})(window.AdminApp);
