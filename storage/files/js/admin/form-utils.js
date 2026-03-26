(function(app) {
    'use strict';

    const escapeHtml = app.escapeHtml;

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
        const hasItems = items && items.length > 0;
        const listItems = hasItems ? items.map((item) => {
            const val = typeof item === 'string' ? item : (item.name || item.id || item);
            const displayName = typeof item === 'string' ? item : (item.name || item.id || String(item));
            const checked = selectedSet[val] ? ' checked' : '';
            const itemId = id + '-chk-' + val.replace(/[^a-zA-Z0-9_-]/g, '_');
            return '<div class="checklist-item" data-item-name="' + escapeHtml(val.toLowerCase()) + '">' +
                '<input type="checkbox" name="' + escapeHtml(id) + '" value="' + escapeHtml(val) + '"' + checked + ' id="' + escapeHtml(itemId) + '">' +
                '<label for="' + escapeHtml(itemId) + '">' + escapeHtml(displayName) + '</label>' +
            '</div>';
        }).join('') : '<div class="empty-state" style="padding:var(--space-4)"><p>None available.</p></div>';
        let filterRow = '<input type="text" class="field-input" placeholder="Filter..." data-filter-list="' + escapeHtml(id) + '" style="margin-bottom:var(--space-2)">';
        if (options.hasSelectAll) {
            filterRow = '<div style="display:flex;gap:var(--space-2);margin-bottom:var(--space-2)">' +
                '<input type="text" class="field-input" placeholder="Search..." data-filter-list="' + escapeHtml(id) + '" style="flex:1">' +
                '<button type="button" class="btn btn-secondary btn-sm" data-select-all="' + escapeHtml(id) + '">Select All</button>' +
                '<button type="button" class="btn btn-secondary btn-sm" data-deselect-all="' + escapeHtml(id) + '">Deselect All</button>' +
            '</div>';
        }
        const maxHeight = options.hasSelectAll ? '300px' : '200px';
        return '<div class="form-group">' +
            '<label class="field-label">' + escapeHtml(label) + '</label>' +
            filterRow +
            '<div class="checklist-container" data-checklist="' + escapeHtml(id) + '" style="max-height:' + maxHeight + ';overflow-y:auto;border:1px solid var(--border-subtle);border-radius:var(--radius-md);padding:var(--space-2)">' +
                listItems +
            '</div>' +
        '</div>';
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
                item.style.display = (q && name.indexOf(q) < 0) ? 'none' : '';
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
