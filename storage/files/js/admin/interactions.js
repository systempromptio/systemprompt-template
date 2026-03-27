(function(app) {
    'use strict';

    const shared = app.shared;

    function initTableSort() {
        const tables = document.querySelectorAll('.data-table');
        for (let t = 0; t < tables.length; t++) {
            setupTableSort(tables[t]);
        }
    }

    function setupTableSort(table) {
        if (table.getAttribute('data-sort-init')) return;
        table.setAttribute('data-sort-init', '1');
        const headers = table.querySelectorAll('thead th');
        if (!headers.length) return;

        for (let i = 0; i < headers.length; i++) {
            const th = headers[i];
            const text = th.textContent.trim();
            if (!text || th.classList.contains('col-actions') || th.classList.contains('col-chevron')) {
                th.style.cursor = 'default';
                continue;
            }
            if (!th.querySelector('.sort-icon')) {
                const icon = document.createElement('span');
                icon.className = 'sort-icon';
                icon.textContent = '\u25B4';
                th.append(icon);
            }
            th.setAttribute('data-sort-col', i);
            th.addEventListener('click', handleSortClick);
        }
    }

    function handleSortClick(e) {
        const th = e.currentTarget;
        const table = th.closest('.data-table');
        if (!table) return;
        const colIndex = parseInt(th.getAttribute('data-sort-col'), 10);
        const tbody = table.querySelector('tbody');
        if (!tbody) return;

        const wasAsc = th.classList.contains('sorted') && th.getAttribute('data-sort-dir') === 'asc';
        const dir = wasAsc ? 'desc' : 'asc';

        const allHeaders = table.querySelectorAll('thead th');
        for (let i = 0; i < allHeaders.length; i++) {
            allHeaders[i].classList.remove('sorted');
            allHeaders[i].removeAttribute('data-sort-dir');
            const icon = allHeaders[i].querySelector('.sort-icon');
            if (icon) icon.textContent = '\u25B4';
        }

        th.classList.add('sorted');
        th.setAttribute('data-sort-dir', dir);
        const activeIcon = th.querySelector('.sort-icon');
        if (activeIcon) activeIcon.textContent = dir === 'asc' ? '\u25B4' : '\u25BE';

        const allRows = Array.prototype.slice.call(tbody.querySelectorAll('tr'));
        const sortableRows = [];
        const detailMap = {};
        for (let r = 0; r < allRows.length; r++) {
            const detailFor = allRows[r].getAttribute('data-detail-for');
            if (detailFor) {
                detailMap[detailFor] = allRows[r];
            } else {
                sortableRows.push(allRows[r]);
            }
        }

        sortableRows.sort((a, b) => {
            const aCell = a.cells[colIndex];
            const bCell = b.cells[colIndex];
            if (!aCell || !bCell) return 0;
            const aVal = getSortValue(aCell);
            const bVal = getSortValue(bCell);

            const aNum = parseFloat(aVal);
            const bNum = parseFloat(bVal);
            if (!isNaN(aNum) && !isNaN(bNum)) {
                return dir === 'asc' ? aNum - bNum : bNum - aNum;
            }
            const cmp = aVal.localeCompare(bVal, undefined, { sensitivity: 'base' });
            return dir === 'asc' ? cmp : -cmp;
        });

        for (let j = 0; j < sortableRows.length; j++) {
            tbody.append(sortableRows[j]);
            const eventId = sortableRows[j].getAttribute('data-event-id');
            if (eventId && detailMap[eventId]) {
                tbody.append(detailMap[eventId]);
            }
        }
    }

    function getSortValue(cell) {
        if (cell.title) return cell.title.toLowerCase();
        const sv = cell.getAttribute('data-sort-value');
        if (sv) return sv.toLowerCase();
        return (cell.textContent || '').trim().toLowerCase();
    }

    function initListPage(entityType, searchInputId, opts) {
        opts = opts || {};
        const searchAttr = opts.searchAttr || 'data-name';

        initTableSort();

        app.events.on('input', '#' + searchInputId, (e, el) => {
            let debounceTimer = el._debounceTimer || null;
            clearTimeout(debounceTimer);
            el._debounceTimer = setTimeout(() => {
                const q = el.value.toLowerCase().trim();
                const rows = document.querySelectorAll('.data-table tbody tr');
                for (let i = 0; i < rows.length; i++) {
                    const searchVal = rows[i].getAttribute(searchAttr) || rows[i].textContent.toLowerCase();
                    rows[i].style.display = (!q || searchVal.includes(q)) ? '' : 'none';
                }
            }, 200);
        });

        app.events.on('click', '[data-action="delete"]', (e, deleteBtn) => {
            shared.closeAllMenus();
            const entityId = deleteBtn.getAttribute('data-entity-id');
            const deleteEntityType = deleteBtn.getAttribute('data-entity-type') || entityType;
            showDeleteDialog(deleteEntityType, entityId, opts);
        }, { exclusive: true });

        app.events.on('click', '[data-confirm-delete]', (e, confirmBtn) => {
            const deleteId = confirmBtn.getAttribute('data-confirm-delete');
            performDelete(entityType, deleteId, confirmBtn, opts);
        }, { exclusive: true });

        app.events.on('change', '[data-action="toggle"]', (e, toggle) => {
            const id = toggle.getAttribute('data-entity-id');
            const toggleType = toggle.getAttribute('data-entity-type') || entityType;
            const enabled = toggle.checked;
            const apiPath = opts.toggleApiPath
                ? opts.toggleApiPath(id)
                : '/' + toggleType + 's/' + encodeURIComponent(id);

            const row = toggle.closest('tr');
            if (row) {
                row.setAttribute('data-enabled', enabled ? 'enabled' : 'disabled');
                const statusCell = row.querySelector('.badge-green, .badge-gray');
                if (statusCell && (statusCell.textContent === 'Active' || statusCell.textContent === 'Disabled')) {
                    statusCell.className = 'badge ' + (enabled ? 'badge-green' : 'badge-gray');
                    statusCell.textContent = enabled ? 'Active' : 'Disabled';
                }
            }

            const statsMap = { 'plugin': 'stats-plugins', 'agent': 'stats-agents', 'mcp-server': 'stats-mcp', 'skill': 'stats-skills' };
            const statsEl = document.getElementById(statsMap[toggleType]);
            const statVal = statsEl ? statsEl.querySelector('.config-stat-value') : null;
            if (statVal) {
                const parts = statVal.textContent.split('/');
                if (parts.length === 2) {
                    const cur = parseInt(parts[0], 10) + (enabled ? 1 : -1);
                    statVal.textContent = cur + '/' + parts[1];
                }
            }

            app.api(apiPath, {
                method: 'PUT',
                body: JSON.stringify({ enabled: enabled })
            }).then(() => {
                app.Toast.show(toggleType + ' ' + (enabled ? 'enabled' : 'disabled'), 'success');
            }).catch((err) => {
                toggle.checked = !enabled;
                if (row) {
                    row.setAttribute('data-enabled', enabled ? 'disabled' : 'enabled');
                    const badge = row.querySelector('.badge-green, .badge-gray');
                    if (badge && (badge.textContent === 'Active' || badge.textContent === 'Disabled')) {
                        badge.className = 'badge ' + (enabled ? 'badge-gray' : 'badge-green');
                        badge.textContent = enabled ? 'Disabled' : 'Active';
                    }
                }
                if (statVal) {
                    const rparts = statVal.textContent.split('/');
                    if (rparts.length === 2) {
                        const rcur = parseInt(rparts[0], 10) + (enabled ? -1 : 1);
                        statVal.textContent = rcur + '/' + rparts[1];
                    }
                }
                app.Toast.show(err.message || 'Failed to update ' + toggleType, 'error');
            });
        });
    }

    function showDeleteDialog(entityType, entityId, opts) {
        const title = 'Delete ' + entityType.charAt(0).toUpperCase() + entityType.slice(1) + '?';
        shared.showDeleteConfirmDialog(title, entityId);
    }

    function performDelete(entityType, entityId, confirmBtn, opts) {
        confirmBtn.disabled = true;
        confirmBtn.textContent = 'Deleting...';
        const apiPath = opts.deleteApiPath
            ? opts.deleteApiPath(entityId)
            : '/' + entityType + 's/' + encodeURIComponent(entityId);
        app.api(apiPath, { method: 'DELETE' }).then(() => {
            app.Toast.show(entityType + ' deleted', 'success');
            shared.closeDeleteConfirm();
            window.location.reload();
        }).catch((err) => {
            app.Toast.show(err.message || 'Failed to delete ' + entityType, 'error');
            confirmBtn.disabled = false;
            confirmBtn.textContent = 'Delete';
        });
    }

    function initEditForm(formId, opts) {
        opts = opts || {};
        const form = document.getElementById(formId);
        if (!form) return;

        const apiPath = form.getAttribute('data-api-path') || '';
        const entity = form.getAttribute('data-entity') || 'item';
        const idField = form.getAttribute('data-id-field') || 'id';
        const listPath = form.getAttribute('data-list-path') || '/';
        const existingId = form.querySelector('[name="' + idField + '"]');
        const isEdit = existingId && existingId.readOnly && existingId.value;

        form.addEventListener('submit', (e) => {
            e.preventDefault();
            const formData = new FormData(form);
            const body = opts.buildBody ? opts.buildBody(form, formData) : formDataToObject(formData);

            let url, method;
            if (isEdit) {
                url = apiPath + encodeURIComponent(existingId.value);
                method = 'PUT';
            } else {
                url = apiPath.replace(/\/$/, '');
                method = 'POST';
            }

            const submitBtn = form.querySelector('[type="submit"]');
            if (submitBtn) { submitBtn.disabled = true; submitBtn.textContent = 'Saving...'; }

            app.api(url, { method: method, body: JSON.stringify(body) })
                .then(() => {
                    app.Toast.show(entity + ' saved!', 'success');
                    window.location.href = app.BASE + listPath;
                })
                .catch((err) => {
                    app.Toast.show(err.message || 'Failed to save ' + entity, 'error');
                    if (submitBtn) { submitBtn.disabled = false; submitBtn.textContent = isEdit ? 'Save Changes' : 'Create'; }
                });
        });
    }

    function formDataToObject(formData) {
        const obj = {};
        formData.forEach((value, key) => {
            if (key === 'tags') {
                obj[key] = value.split(',').map((t) => t.trim()).filter(Boolean);
            } else {
                obj[key] = value;
            }
        });
        return obj;
    }

    document.addEventListener('DOMContentLoaded', () => {
        app.events.init();
        initTableSort();
    });

    app.interactions = {
        initListPage: initListPage,
        initEditForm: initEditForm,
        initTableSort: initTableSort
    };
})(window.AdminApp);
